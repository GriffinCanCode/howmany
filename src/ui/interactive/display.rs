use crate::core::types::{CodeStats, FileStats};
use crate::ui::interactive::app::InteractiveApp;
use crate::ui::interactive::rendering::{
    render_footer, render_header, render_help, render_main_content, render_welcome,
};
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

/// How long to wait for a key before redrawing the animation.
///
/// This is the frame budget *and* the idle cost: the loop blocks in
/// `event::poll` for this long, so an open but untouched view uses no CPU.
const FRAME_INTERVAL: Duration = Duration::from_millis(100);

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
        println!(
            "{}",
            format!("📁 Analyzing directory: {}", path).bright_yellow()
        );
        println!(
            "{}",
            "🔍 Scanning for user-created code files...".bright_blue()
        );
        println!();

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Scanning files...");
        pb.enable_steady_tick(Duration::from_millis(100));
        Ok(pb)
    }

    /// Draw, then wait for a key or the next animation frame, until asked to quit.
    ///
    /// The loop blocks in `event::poll`, so it is idle between keystrokes. The
    /// previous implementation drove the same three cases through a tokio
    /// `select!` whose event branch returned immediately, which span the loop at
    /// full speed and held a core busy for as long as the view was open.
    pub fn run_interactive_mode(
        &mut self,
        stats: CodeStats,
        individual_files: Vec<(String, FileStats)>,
    ) -> Result<()> {
        self.app.set_data(stats, individual_files);
        self.render_frame()?;

        while !self.app.should_quit {
            if event::poll(FRAME_INTERVAL)? {
                // Drain the queue before drawing: held keys and paste-like
                // bursts must not each cost a frame.
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            self.app.handle_key_event(key.code);
                        }
                    }
                }
            } else {
                self.app.update_animation();
            }

            self.render_frame()?;
        }

        Ok(())
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
