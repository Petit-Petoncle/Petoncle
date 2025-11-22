mod chat;

use anyhow::{Context, Result};
use chat::{ChatLoopResult, ChatState};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Main entry point for Petoncle terminal wrapper
fn main() -> Result<()> {
    println!("🐚 Petoncle - AI-Powered Terminal Wrapper");
    println!("💡 Appuyez sur '!' pour ouvrir le chat AI");
    println!("Starting zsh session...\n");

    // Small delay to let message display before raw mode
    thread::sleep(Duration::from_millis(100));

    // Get terminal size
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    // Get PTY system
    let pty_system = native_pty_system();

    // Create a new PTY with actual terminal size
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .context("Failed to create PTY")?;

    // Spawn zsh shell in current working directory
    let mut cmd = CommandBuilder::new("zsh");
    cmd.env("TERM", "xterm-256color");

    // Start in the same directory where Petoncle was launched
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn zsh")?;

    // Get reader and writer from master PTY
    let mut reader = pair.master.try_clone_reader()?;
    let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
    let writer_clone = writer.clone();

    // Shared buffer for shell output
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let output_buffer_clone = output_buffer.clone();

    // Shared flag to signal shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone1 = running.clone();
    let running_clone2 = running.clone();

    // Shared flag to pause output during chat
    let output_paused = Arc::new(AtomicBool::new(false));
    let output_paused_clone = output_paused.clone();

    // Create persistent chat state
    let chat_state = Arc::new(Mutex::new(ChatState::new()));
    let chat_state_clone = chat_state.clone();

    // Enable raw mode for proper terminal handling
    enable_raw_mode().context("Failed to enable raw mode")?;

    // Thread to read from PTY and print to stdout
    let output_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            if !running_clone1.load(Ordering::Relaxed) {
                break;
            }

            match reader.read(&mut buf) {
                Ok(0) => {
                    // EOF - shell has exited
                    running_clone1.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(n) => {
                    let data = &buf[..n];

                    // Store in buffer for RAG (will be used later)
                    if let Ok(mut buffer) = output_buffer_clone.lock() {
                        buffer.extend_from_slice(data);

                        // Keep last 100KB to avoid unbounded growth
                        if buffer.len() > 100_000 {
                            buffer.drain(..50_000);
                        }
                    }

                    // Print to stdout only if not in chat mode
                    if !output_paused_clone.load(Ordering::Relaxed) {
                        std::io::stdout().write_all(data).ok();
                        std::io::stdout().flush().ok();
                    }
                }
                Err(_) => {
                    running_clone1.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    // Main input loop (handles both terminal and chat mode)
    let input_loop_result = input_loop(writer_clone, running_clone2, output_paused, chat_state_clone);

    // Cleanup
    running.store(false, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(100));

    disable_raw_mode().context("Failed to disable raw mode")?;

    output_thread.join().ok();

    let exit_status = child.wait()?;
    println!("\n🐚 Shell exited with status: {:?}", exit_status);

    input_loop_result
}

/// Main input loop that handles terminal mode and chat mode
fn input_loop(
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    running: Arc<AtomicBool>,
    output_paused: Arc<AtomicBool>,
    chat_state: Arc<Mutex<ChatState>>,
) -> Result<()> {
    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Check for '!' to trigger chat mode
                    if key_event.code == KeyCode::Char('!')
                        && !key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        // Enter chat mode
                        match enter_chat_mode(&output_paused, &chat_state) {
                            Ok(ChatLoopResult::ExecuteCommand(cmd)) => {
                                // Send the command to PTY without executing (no Enter)
                                if let Ok(mut w) = writer.lock() {
                                    w.write_all(cmd.as_bytes()).ok();
                                    w.flush().ok();
                                }
                            }
                            Ok(ChatLoopResult::Closed) => {
                                // Just closed, do nothing
                            }
                            Err(e) => {
                                eprintln!("Chat error: {}", e);
                            }
                        }
                        continue;
                    }

                    // Handle Ctrl+D as a special case to exit gracefully
                    if key_event.code == KeyCode::Char('d')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if let Ok(mut w) = writer.lock() {
                            w.write_all(&[4]).ok();
                            w.flush().ok();
                        }
                        continue;
                    }

                    // Convert crossterm key event to bytes and send to PTY
                    let bytes = key_event_to_bytes(key_event);
                    if !bytes.is_empty() {
                        if let Ok(mut w) = writer.lock() {
                            if w.write_all(&bytes).is_err() {
                                break;
                            }
                            w.flush().ok();
                        }
                    }
                }
                Event::Resize(_w, _h) => {
                    // Handle terminal resize
                    // We'll implement this later when we add proper PTY resize support
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Enter chat mode with ratatui overlay
fn enter_chat_mode(
    output_paused: &Arc<AtomicBool>,
    chat_state: &Arc<Mutex<ChatState>>,
) -> Result<ChatLoopResult> {
    // Pause shell output
    output_paused.store(true, Ordering::Relaxed);

    // Setup terminal for ratatui
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    // Run chat loop with persistent state
    let result = {
        let mut state = chat_state.lock().unwrap();
        chat::run_chat_loop(&mut terminal, &mut state)
    };

    // Cleanup and return to normal mode
    execute!(std::io::stdout(), LeaveAlternateScreen)?;

    // Resume shell output
    output_paused.store(false, Ordering::Relaxed);

    result
}

/// Convert crossterm KeyEvent to bytes to send to PTY
fn key_event_to_bytes(key_event: event::KeyEvent) -> Vec<u8> {
    match key_event.code {
        KeyCode::Char(c) => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Handle Ctrl+ combinations
                match c {
                    'a'..='z' => vec![c as u8 - b'a' + 1],
                    '@' => vec![0],
                    '[' => vec![27],
                    '\\' => vec![28],
                    ']' => vec![29],
                    '^' => vec![30],
                    '_' => vec![31],
                    _ => c.to_string().into_bytes(),
                }
            } else {
                c.to_string().into_bytes()
            }
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![27],
        KeyCode::Up => vec![27, 91, 65],
        KeyCode::Down => vec![27, 91, 66],
        KeyCode::Right => vec![27, 91, 67],
        KeyCode::Left => vec![27, 91, 68],
        KeyCode::Home => vec![27, 91, 72],
        KeyCode::End => vec![27, 91, 70],
        KeyCode::PageUp => vec![27, 91, 53, 126],
        KeyCode::PageDown => vec![27, 91, 54, 126],
        KeyCode::Delete => vec![27, 91, 51, 126],
        KeyCode::Insert => vec![27, 91, 50, 126],
        KeyCode::F(n) => match n {
            1 => vec![27, 79, 80],
            2 => vec![27, 79, 81],
            3 => vec![27, 79, 82],
            4 => vec![27, 79, 83],
            5..=12 => vec![27, 91, b'0' + (n - 5) as u8, 126],
            _ => vec![],
        },
        _ => vec![],
    }
}
