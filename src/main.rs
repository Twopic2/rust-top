#[cfg(not(feature = "dhat-heap"))]
use rust_top::app::App;
use std::io;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(feature = "dhat-heap")]
fn main() -> io::Result<()> {
    let _profiler = dhat::Profiler::builder()
        .trim_backtraces(Some(24))
        .build();

    let iters: usize = std::env::var("PROFILE_ITERS")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(100);

    let mut collection = rust_top::collection::TopCollection::new();
    for _ in 0..iters {
        collection.refresh_timer();
        collection.update_data();
    }
    Ok(())
}

#[cfg(not(feature = "dhat-heap"))]
fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new().run(&mut terminal);
    ratatui::restore();
    result
}
