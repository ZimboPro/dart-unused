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
    println!("Hello, world!");
    Ok(())
}
