pub use app::App;

pub mod app;
pub mod chezmoi;
pub mod utils;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    chezmoi::check_installed()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
