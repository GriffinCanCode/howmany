use crate::core::types::{CodeStats, FileStats};
use crate::ui::interactive::app::InteractiveApp;
use crate::ui::interactive::rendering::{render_footer, render_header, render_main_content, render_help, render_welcome};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::{
    io::{self, stdout},
    time::Duration,
};
use tokio::time::{interval, timeout};

pub struct ModernInteractiveDisplay {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    app: InteractiveApp,
}

impl ModernInteractiveDisplay {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            app: InteractiveApp::new(),
        })
    }

    pub fn show_welcome(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let area = f.area();
            render_welcome(f, area);
        })?;

        Ok(())
    }

    pub fn show_scanning_progress(&mut self, path: &str) -> Result<ProgressBar> {
        println!("{}", format!("📁 Analyzing directory: {}", path).bright_yellow());
        println!("{}", "🔍 Scanning for user-created code files...".bright_blue());
        println!();
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        pb.set_message("Scanning files...");
        pb.enable_steady_tick(Duration::from_millis(100));
        Ok(pb)
    }

    pub fn run_interactive_mode(&mut self, stats: CodeStats, individual_files: Vec<(String, FileStats)>) -> Result<()> {
        self.app.set_data(stats, individual_files);

        // Use tokio runtime for async event handling
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(self.run_interactive_async())
    }

    async fn run_interactive_async(&mut self) -> Result<()> {
        let mut animation_interval = interval(Duration::from_millis(100));
        let mut redraw_needed = true;
        let mut _frame_count = 0u64;
        let mut last_fps_check = std::time::Instant::now();

        loop {
            tokio::select! {
                // Handle keyboard events with highest priority
                event_result = self.handle_events_async() => {
                    match event_result {
                        Ok(true) => {
                            redraw_needed = true;
                        }
                        Ok(false) => {
                            // No event or no redraw needed
                        }
                        Err(e) => {
                            eprintln!("Event handling error: {}", e);
                        }
                    }
                }
                
                // Update animation at regular intervals
                _ = animation_interval.tick() => {
                    self.app.update_animation();
                    redraw_needed = true;
                }
                
                // Background task: Process any heavy computations
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Background processing completed
                }
            }

            // Redraw if needed with frame rate limiting
            if redraw_needed {
                self.render_frame()?;
                redraw_needed = false;
                _frame_count += 1;
                
                // Optional: FPS monitoring (can be removed in production)
                if last_fps_check.elapsed() >= Duration::from_secs(1) {
                    // Reset counters for next second
                    _frame_count = 0;
                    last_fps_check = std::time::Instant::now();
                }
            }

            // Check if we should quit
            if self.app.should_quit {
                break;
            }

            // Yield control to prevent busy waiting
            tokio::task::yield_now().await;
        }

        Ok(())
    }

    async fn handle_events_async(&mut self) -> Result<bool> {
        // Use timeout to make event polling non-blocking
        let event_timeout = timeout(Duration::from_millis(1), async {
            // Check if events are available
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.app.handle_key_event(key.code);
                        return Ok(true); // Redraw needed
                    }
                }
            }
            Ok(false) // No redraw needed
        }).await;

        match event_timeout {
            Ok(result) => result,
            Err(_) => Ok(false), // Timeout, no events
        }
    }



    fn render_frame(&mut self) -> Result<()> {
        let app = &mut self.app;
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            render_header(f, chunks[0], app);
            
            if app.show_help {
                render_help(f, chunks[1]);
            } else {
                render_main_content(f, chunks[1], app);
            }
            
            render_footer(f, chunks[2], app);
        })?;

        Ok(())
    }
}

impl Drop for ModernInteractiveDisplay {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
} 