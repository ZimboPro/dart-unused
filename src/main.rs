use dart_unused::get_unreferenced_files;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

fn main() -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .build();
    TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    get_unreferenced_files()
}
