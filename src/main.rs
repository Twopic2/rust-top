mod data;
mod draw;
mod app;
mod collection;
mod event;
mod processes;
mod tools;

use app::App;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal).await;
    ratatui::restore();
    result
}
