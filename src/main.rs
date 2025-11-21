use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Main entry point for Petoncle terminal wrapper
fn main() -> Result<()> {
    println!("🐚 Petoncle - AI-Powered Terminal Wrapper");
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

    // Spawn zsh shell
    let mut cmd = CommandBuilder::new("zsh");
    cmd.env("TERM", "xterm-256color");

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn zsh")?;

    // Get reader and writer from master PTY
    let mut reader = pair.master.try_clone_reader()?;
    let mut writer = pair.master.take_writer()?;

    // Shared buffer for shell output
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let output_buffer_clone = output_buffer.clone();

    // Shared flag to signal shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone1 = running.clone();
    let running_clone2 = running.clone();

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

                    // Print to stdout
                    std::io::stdout().write_all(data).ok();
                    std::io::stdout().flush().ok();
                }
                Err(_) => {
                    running_clone1.store(false, Ordering::Relaxed);
                    break;
                }
            }
        }
    });

    // Thread to read from stdin and write to PTY
    let input_thread = thread::spawn(move || {
        loop {
            if !running_clone2.load(Ordering::Relaxed) {
                break;
            }

            // Poll for events with timeout
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(key_event)) => {
                        // Handle Ctrl+D as a special case to exit gracefully
                        if key_event.code == KeyCode::Char('d')
                            && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            writer.write_all(&[4]).ok(); // Send EOT
                            writer.flush().ok();
                            continue;
                        }

                        // Convert crossterm key event to bytes
                        let bytes = key_event_to_bytes(key_event);
                        if !bytes.is_empty() {
                            // TODO: Here we'll add autocompletion detection later
                            // For now, just pass through
                            if writer.write_all(&bytes).is_err() {
                                break;
                            }
                            writer.flush().ok();
                        }
                    }
                    Ok(Event::Resize(_w, _h)) => {
                        // Handle terminal resize
                        // We'll implement this later when we add proper PTY resize support
                    }
                    _ => {}
                }
            }
        }
    });

    // Wait for shell to exit
    let exit_status = child.wait()?;
    running.store(false, Ordering::Relaxed);

    // Wait a bit for threads to finish
    thread::sleep(Duration::from_millis(100));

    // Disable raw mode before exiting
    disable_raw_mode().context("Failed to disable raw mode")?;

    // Wait for I/O threads
    output_thread.join().ok();
    input_thread.join().ok();

    println!("\n🐚 Shell exited with status: {:?}", exit_status);

    Ok(())
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
        KeyCode::F(n) => {
            match n {
                1 => vec![27, 79, 80],
                2 => vec![27, 79, 81],
                3 => vec![27, 79, 82],
                4 => vec![27, 79, 83],
                5..=12 => vec![27, 91, b'0' + (n - 5) as u8, 126],
                _ => vec![],
            }
        }
        _ => vec![],
    }
}
