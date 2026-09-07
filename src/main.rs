mod app;
mod input;
mod windowpanes;
mod widgets;
mod theme;
mod log;

use app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let result = App::new().run_loop(terminal);

    ratatui::restore();
    return result;
}
