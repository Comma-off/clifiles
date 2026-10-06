mod app;
mod config;
mod exit;
mod fs;
mod input;
mod picker_setup;
mod prefetcher;
mod preview;
mod preview_worker;
mod ui;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use input::Signal;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "clifiles", about = "A TUI file manager")]
struct Cli {
    /// Directory to start in
    dir: Option<PathBuf>,

    /// Path to write the final directory to on exit, for shell cd integration
    #[arg(long)]
    cwd_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = config::load();

    let start_dir = cli
        .dir
        .unwrap_or_else(|| resolve_default_start_dir(config.start_dir))
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."));

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, start_dir.clone(), config);
    restore_terminal(&mut terminal)?;

    let final_cwd = result?;
    exit::write_cwd(&cli.cwd_file, &final_cwd)?;
    Ok(())
}

fn resolve_default_start_dir(start_dir: config::StartDir) -> PathBuf {
    let current = || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match start_dir {
        config::StartDir::Current => current(),
        config::StartDir::Home => directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .unwrap_or_else(current),
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    start_dir: PathBuf,
    config: config::Config,
) -> Result<PathBuf> {
    let picker = picker_setup::build_picker(&config.image);
    let mut app = App::new(start_dir, config)?;
    app.request_preview(picker);

    loop {
        app.poll_preview();

        let layouts = {
            let mut captured = None;
            terminal.draw(|f| captured = Some(ui::draw(f, &mut app)))?;
            captured.expect("draw always runs the closure")
        };
        app.last_preview_area = layouts.preview_inner;

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        let signal = match event::read()? {
            Event::Key(key) => input::handle_key(&mut app, key)?,
            Event::Mouse(mouse) => input::handle_mouse(
                &mut app,
                mouse,
                layouts.list_inner_top,
                layouts.list_inner_rows,
            ),
            Event::Resize(_, _) => Signal::Redraw,
            _ => Signal::Continue,
        };

        match signal {
            Signal::Quit => break,
            Signal::PreviewDirty => app.request_preview(picker),
            Signal::Redraw | Signal::Continue => {}
        }
    }

    Ok(app.cwd)
}
