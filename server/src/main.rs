mod app;
mod core;

use core::Core;
use app::App;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
}