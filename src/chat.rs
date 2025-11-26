use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::Stdout;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

use crate::grpc_client::AgentClient;

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub enum MessageState {
    Loading,
    Ready,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub state: MessageState,
    pub agent: Option<String>, // Which agent handled this message (toolsmith, researcher, scribe, general)
}

// Spinner frames for loading animation
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub scroll_offset: u16, // Scroll position (line-based)
    pub auto_scroll: bool, // Auto-scroll to bottom on new message
    pub last_visible_height: u16, // Last known visible height of messages area
    pub spinner_frame: usize, // Current spinner frame index
    pub last_spinner_update: Instant, // Last time spinner was updated
    pub response_receiver: Option<Receiver<Result<(String, String)>>>, // Channel to receive async responses (message, agent)
    grpc_client: AgentClient,
    runtime: Runtime,
}

impl ChatState {
    pub fn new() -> Self {
        // Initialize gRPC client and tokio runtime
        let grpc_client = AgentClient::new("127.0.0.1:50051");
        let runtime = Runtime::new().expect("Failed to create tokio runtime");

        Self {
            messages: vec![ChatMessage {
                role: MessageRole::Assistant,
                content: "👋 Bienvenue dans Petoncle!\n\n🤖 Connexion au service IA en cours...\n💡 Appuyez sur ESC pour fermer".to_string(),
                timestamp: Local::now(),
                state: MessageState::Ready,
                agent: None,
            }],
            input: String::new(),
            scroll_offset: 0,
            auto_scroll: true,
            last_visible_height: 20, // Default fallback
            spinner_frame: 0,
            last_spinner_update: Instant::now(),
            response_receiver: None,
            grpc_client,
            runtime,
        }
    }

    /// Calculate total number of lines in all messages
    fn count_total_lines(&self) -> usize {
        let mut count = 0;
        for msg in &self.messages {
            count += 1; // Header line
            count += 1; // Empty line after header
            count += msg.content.lines().count();
            count += 1; // Empty line
            count += 1; // Separator
            count += 1; // Empty line after separator
        }
        count
    }

    /// Scroll to the latest message (bottom of chat)
    pub fn scroll_to_bottom(&mut self, visible_height: u16) {
        let total_lines = self.count_total_lines();
        let visible = visible_height as usize;

        // Scroll to show the last messages
        if total_lines > visible {
            self.scroll_offset = total_lines.saturating_sub(visible) as u16;
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Get maximum scroll offset based on visible height
    pub fn max_scroll_offset(&self, visible_height: u16) -> u16 {
        let total_lines = self.count_total_lines();
        let visible = visible_height as usize;

        if total_lines > visible {
            total_lines.saturating_sub(visible) as u16
        } else {
            0
        }
    }

    /// Scroll down by n lines, respecting bounds
    pub fn scroll_down(&mut self, n: u16, visible_height: u16) {
        let max_offset = self.max_scroll_offset(visible_height);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
        self.auto_scroll = false;
    }

    /// Scroll up by n lines
    pub fn scroll_up(&mut self, n: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        self.auto_scroll = false;
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
            timestamp: Local::now(),
            state: MessageState::Ready,
            agent: None, // User messages don't have an agent
        });
        self.auto_scroll = true; // Request auto-scroll on next render
    }

    pub fn add_assistant_message(&mut self, content: String, agent: Option<String>) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content,
            timestamp: Local::now(),
            state: MessageState::Ready,
            agent,
        });
        self.auto_scroll = true; // Request auto-scroll on next render
    }

    pub fn add_loading_message(&mut self) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: "Réflexion en cours".to_string(),
            timestamp: Local::now(),
            state: MessageState::Loading,
            agent: None, // Will be set when response is received
        });
        self.auto_scroll = true;
    }

    pub fn update_last_message(&mut self, content: String, agent: Option<String>) {
        if let Some(last) = self.messages.last_mut() {
            last.content = content;
            last.state = MessageState::Ready;
            last.agent = agent;
            self.auto_scroll = true;
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    /// Start generating AI response asynchronously (non-blocking)
    pub fn start_generate_response(&mut self, user_input: String) {
        // Create channel for async communication
        let (tx, rx): (Sender<Result<(String, String)>>, Receiver<Result<(String, String)>>) = mpsc::channel();

        // Take ownership of grpc_client temporarily
        let mut client = AgentClient::new("127.0.0.1:50051");
        std::mem::swap(&mut client, &mut self.grpc_client);

        // Spawn thread to handle gRPC call
        thread::spawn(move || {
            // Create runtime for this thread
            let runtime = Runtime::new().unwrap();

            let result = runtime.block_on(async {
                client.send_message(user_input, vec![]).await
            });

            let response = match result {
                Ok(resp) => Ok((resp.message, resp.agent)),
                Err(e) => Ok((format!(
                    "⚠️ Service IA non disponible\n\n\
                     Erreur: {}\n\n\
                     💡 Assurez-vous que le service Python est démarré:\n\
                     cd python && python agent_service.py",
                    e
                ), "error".to_string())),
            };

            // Send result back
            tx.send(response).ok();
        });

        // Store receiver
        self.response_receiver = Some(rx);

        // Add loading message
        self.add_loading_message();
    }

    /// Check if response is ready and update message
    pub fn check_response(&mut self) -> bool {
        if let Some(ref receiver) = self.response_receiver {
            if let Ok(result) = receiver.try_recv() {
                // Response received!
                match result {
                    Ok((content, agent)) => {
                        self.update_last_message(content, Some(agent));
                    }
                    Err(e) => {
                        self.update_last_message(format!("❌ Error: {}", e), Some("error".to_string()));
                    }
                }
                self.response_receiver = None;
                return true;
            }
        }
        false
    }

    /// Update spinner animation
    pub fn update_spinner(&mut self) {
        if self.last_spinner_update.elapsed() > Duration::from_millis(80) {
            self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
            self.last_spinner_update = Instant::now();
        }
    }
}

/// Render the chat overlay UI
pub fn render_chat_ui(
    frame: &mut Frame,
    state: &mut ChatState,
    area: Rect,
) {
    // Create a centered popup area (80% width, 70% height)
    let popup_area = centered_rect(80, 70, area);

    // Split into messages and input sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Min(0),      // Messages
            Constraint::Length(3),   // Input box
        ])
        .split(popup_area);

    // Store the actual visible height of the messages area
    let visible_height = chunks[0].height.saturating_sub(2); // Subtract borders
    state.last_visible_height = visible_height;

    // Apply auto-scroll if requested (before building lines)
    if state.auto_scroll {
        state.scroll_to_bottom(visible_height);
        state.auto_scroll = false;
    }

    // Get current spinner frame
    let current_spinner_frame = state.spinner_frame;

    // Build a single text with all messages (line by line)
    let mut lines: Vec<Line> = Vec::new();

    for msg in &state.messages {
        let time = msg.timestamp.format("%H:%M:%S");
        let (prefix, style) = match msg.role {
            MessageRole::User => (
                "🧑 You",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            MessageRole::Assistant => (
                "🤖 Petoncle",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        };

        // Build agent badge if available
        let agent_badge = if let Some(ref agent) = msg.agent {
            let (emoji, _) = match agent.as_str() {
                "toolsmith" => ("🛠️", Color::Yellow),
                "researcher" => ("🔍", Color::Blue),
                "scribe" => ("📝", Color::Magenta),
                "general" => ("🧠", Color::Cyan),
                "error" => ("⚠️", Color::Red),
                _ => ("❓", Color::White),
            };
            format!(" {} {}", emoji, agent)
        } else {
            String::new()
        };

        // Add header with agent badge
        let mut header_spans = vec![
            Span::styled(prefix, style),
            Span::raw(format!(" • {}", time)),
        ];
        if !agent_badge.is_empty() {
            if let Some(ref agent) = msg.agent {
                let (_, color) = match agent.as_str() {
                    "toolsmith" => ("🛠️", Color::Yellow),
                    "researcher" => ("🔍", Color::Blue),
                    "scribe" => ("📝", Color::Magenta),
                    "general" => ("🧠", Color::Cyan),
                    "error" => ("⚠️", Color::Red),
                    _ => ("❓", Color::White),
                };
                header_spans.push(Span::styled(
                    agent_badge,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
            }
        }
        lines.push(Line::from(header_spans));
        lines.push(Line::from(""));

        // Add content with spinner animation if loading
        match msg.state {
            MessageState::Loading => {
                let spinner = SPINNER_FRAMES[current_spinner_frame];
                lines.push(Line::from(vec![
                    Span::styled(
                        spinner,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        &msg.content,
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw("..."),
                ]));
            }
            MessageState::Ready => {
                // Add content (no truncation, full message)
                for line in msg.content.lines() {
                    lines.push(Line::from(line.to_string()));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from("───────────────────────────────────"));
        lines.push(Line::from(""));
    }

    // Create Paragraph with scroll
    let messages_paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("💬 Petoncle Chat (↑↓ scroller | Home/End haut/bas | ESC quitter)")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(messages_paragraph, chunks[0]);

    // Render input box
    let input_text = format!("➤ {}", state.input);
    let input = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title("Votre message (Enter pour envoyer)"),
        )
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(input, chunks[1]);
}

/// Result of the chat loop
pub enum ChatLoopResult {
    Closed,
}

/// Run the chat overlay loop
pub fn run_chat_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut ChatState,
) -> Result<ChatLoopResult> {
    loop {
        // Check if response is ready
        state.check_response();

        // Update spinner animation if waiting for response
        if state.response_receiver.is_some() {
            state.update_spinner();
        }

        // Render the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Fill background (simulate the terminal still being visible)
            let bg = Block::default().style(Style::default().bg(Color::Black));
            frame.render_widget(bg, area);

            render_chat_ui(frame, state, area);
        })?;

        // Use shorter poll timeout when waiting for response (for smoother animation)
        let poll_timeout = if state.response_receiver.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(100)
        };

        // Handle input events
        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Paste(text) => {
                    // Handle pasted text
                    state.input.push_str(&text);
                }
                Event::Key(key_event) => {
                    // Use the last known visible height from render
                    let visible_height = state.last_visible_height;

                    match key_event.code {
                        KeyCode::Esc => {
                            // Exit chat mode
                            return Ok(ChatLoopResult::Closed);
                        }
                        KeyCode::Up => {
                            // Scroll up
                            state.scroll_up(1);
                        }
                        KeyCode::Down => {
                            // Scroll down with bounds check
                            state.scroll_down(1, visible_height);
                        }
                        KeyCode::PageUp => {
                            // Scroll up by page (10 lines)
                            state.scroll_up(10);
                        }
                        KeyCode::PageDown => {
                            // Scroll down by page (10 lines)
                            state.scroll_down(10, visible_height);
                        }
                        KeyCode::Home => {
                            // Jump to top
                            state.scroll_offset = 0;
                            state.auto_scroll = false;
                        }
                        KeyCode::End => {
                            // Jump to bottom
                            state.auto_scroll = true; // Trigger auto-scroll on next render
                        }
                        KeyCode::Enter => {
                            // Send message
                            if !state.input.is_empty() && state.response_receiver.is_none() {
                                let user_message = state.input.clone();
                                state.add_user_message(user_message.clone());
                                state.clear_input();

                                // Start generating AI response asynchronously (non-blocking)
                                state.start_generate_response(user_message);
                            }
                        }
                        KeyCode::Char(c) => {
                            // Add character to input
                            state.input.push(c);
                        }
                        KeyCode::Backspace => {
                            // Remove character
                            state.input.pop();
                        }
                        _ => {}
                    }
                }
                _ => {} // Ignore other events (Mouse, Resize, etc.)
            }
        }
    }
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
