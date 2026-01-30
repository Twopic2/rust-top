mod data;
mod draw;
mod app;
mod event;
mod processes;

use app::App;
use std::io;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}