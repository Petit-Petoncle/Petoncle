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
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::grpc_client::AgentClient;

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Local>,
}

pub struct ChatState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub cursor_position: usize,
    pub extracted_commands: Vec<String>, // Commands extracted from last AI response
    pub scroll_offset: u16, // Scroll position (line-based)
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
            }],
            input: String::new(),
            cursor_position: 0,
            extracted_commands: Vec::new(),
            scroll_offset: 0,
            grpc_client,
            runtime,
        }
    }

    /// Scroll to the latest message (bottom of chat)
    pub fn scroll_to_bottom(&mut self) {
        // Set to a very large value to scroll to bottom
        self.scroll_offset = u16::MAX;
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content,
            timestamp: Local::now(),
        });
        // Don't auto-scroll, let user scroll manually with ↑↓
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content,
            timestamp: Local::now(),
        });
        // Don't auto-scroll, let user scroll manually with ↑↓
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    /// Generate AI response using gRPC service
    pub fn generate_response(&mut self, user_input: &str) -> String {
        eprintln!("[CHAT] Sending message to gRPC service...");

        // Try to call gRPC service
        let result = self.runtime.block_on(async {
            self.grpc_client
                .send_message(user_input.to_string(), vec![])
                .await
        });

        match result {
            Ok(response) => {
                eprintln!("[CHAT] Received response: {} bytes, {} commands",
                         response.message.len(), response.commands.len());

                // Update extracted commands from response
                self.extracted_commands = response.commands;
                response.message
            }
            Err(e) => {
                // Fallback to placeholder if service unavailable
                eprintln!("[CHAT] gRPC error: {}", e);
                format!(
                    "⚠️ Service IA non disponible\n\n\
                     Erreur: {}\n\n\
                     💡 Assurez-vous que le service Python est démarré:\n\
                     cd python && python agent_service.py",
                    e
                )
            }
        }
    }
}

/// Extract shell commands from AI response text
/// Looks for lines that appear to be shell commands
fn extract_commands(text: &str) -> Vec<String> {
    let mut commands = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip empty lines, comments, and explanatory text
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with('•')
            || trimmed.starts_with('-')
            || trimmed.starts_with("⚠️")
            || trimmed.starts_with("💡")
            || trimmed.starts_with("🔍")
            || trimmed.starts_with("🛡️")
            || trimmed.starts_with("🔎")
            || trimmed.starts_with("🔌")
            || trimmed.ends_with(':')
            || trimmed.chars().all(|c| !c.is_ascii_alphanumeric())
        {
            continue;
        }

        // Check if it looks like a command (starts with common command names or contains shell syntax)
        let looks_like_command = trimmed.starts_with("nmap ")
            || trimmed.starts_with("sqlmap ")
            || trimmed.starts_with("nc ")
            || trimmed.starts_with("netcat ")
            || trimmed.starts_with("curl ")
            || trimmed.starts_with("wget ")
            || trimmed.starts_with("grep ")
            || trimmed.starts_with("find ")
            || trimmed.starts_with("awk ")
            || trimmed.starts_with("sed ")
            || trimmed.starts_with("python ")
            || trimmed.starts_with("ruby ")
            || trimmed.starts_with("perl ")
            || trimmed.contains(" | ")
            || trimmed.contains(" && ")
            || trimmed.contains(" || ")
            || (trimmed.contains('<') && trimmed.contains('>'));

        if looks_like_command {
            commands.push(trimmed.to_string());
        }
    }

    // Limit to 9 commands max (for numeric keybindings 1-9)
    commands.truncate(9);
    commands
}

/// Render the chat overlay UI
pub fn render_chat_ui(
    frame: &mut Frame,
    state: &mut ChatState,
    area: Rect,
) {
    // Create a centered popup area (80% width, 70% height)
    let popup_area = centered_rect(80, 70, area);

    // Calculate constraints based on whether we have commands to show
    let constraints = if state.extracted_commands.is_empty() {
        vec![
            Constraint::Min(0),      // Messages
            Constraint::Length(3),   // Input box
        ]
    } else {
        vec![
            Constraint::Min(0),      // Messages
            Constraint::Length(state.extracted_commands.len() as u16 + 2), // Commands box
            Constraint::Length(3),   // Input box
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(popup_area);

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

        // Add header
        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(format!(" • {}", time)),
        ]));
        lines.push(Line::from(""));

        // Add content (no truncation, full message)
        for line in msg.content.lines() {
            lines.push(Line::from(line.to_string()));
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
                .title("💬 Petoncle Chat (↑↓ pour scroller, ESC pour quitter)")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(messages_paragraph, chunks[0]);

    let mut next_chunk = 1;

    // Render commands box if there are any
    if !state.extracted_commands.is_empty() {
        let mut command_lines = vec![];
        for (i, cmd) in state.extracted_commands.iter().enumerate() {
            command_lines.push(Line::from(vec![
                Span::styled(
                    format!("[{}]", i + 1),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(cmd, Style::default().fg(Color::White)),
            ]));
        }

        let commands_widget = Paragraph::new(command_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title("⚡ Commandes Détectées (Appuyez 1-9 pour envoyer au terminal)"),
            )
            .style(Style::default().bg(Color::Black));

        frame.render_widget(commands_widget, chunks[next_chunk]);
        next_chunk += 1;
    }

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

    frame.render_widget(input, chunks[next_chunk]);
}

/// Result of the chat loop - either None (just closed) or Some(command) to execute
pub enum ChatLoopResult {
    Closed,
    ExecuteCommand(String),
}

/// Run the chat overlay loop
pub fn run_chat_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut ChatState,
) -> Result<ChatLoopResult> {
    loop {
        // Render the UI
        terminal.draw(|frame| {
            let area = frame.area();

            // Fill background (simulate the terminal still being visible)
            let bg = Block::default().style(Style::default().bg(Color::Black));
            frame.render_widget(bg, area);

            render_chat_ui(frame, state, area);
        })?;

        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc => {
                        // Exit chat mode
                        return Ok(ChatLoopResult::Closed);
                    }
                    KeyCode::Up => {
                        // Scroll up
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        // Scroll down
                        state.scroll_offset = state.scroll_offset.saturating_add(1);
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        // Check if it's a command number (1-9)
                        let num = c.to_digit(10).unwrap() as usize;
                        if num > 0 && num <= state.extracted_commands.len() {
                            // Return the command to execute
                            let cmd = state.extracted_commands[num - 1].clone();
                            return Ok(ChatLoopResult::ExecuteCommand(cmd));
                        } else {
                            // Not a valid command number, add to input
                            state.input.push(c);
                            state.cursor_position += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // Send message
                        if !state.input.is_empty() {
                            let user_message = state.input.clone();
                            state.add_user_message(user_message.clone());
                            state.clear_input();

                            // Generate AI response via gRPC
                            let response = state.generate_response(&user_message);
                            state.add_assistant_message(response);
                        }
                    }
                    KeyCode::Char(c) => {
                        // Add character to input
                        state.input.push(c);
                        state.cursor_position += 1;
                    }
                    KeyCode::Backspace => {
                        // Remove character
                        if !state.input.is_empty() {
                            state.input.pop();
                            if state.cursor_position > 0 {
                                state.cursor_position -= 1;
                            }
                        }
                    }
                    _ => {}
                }
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
