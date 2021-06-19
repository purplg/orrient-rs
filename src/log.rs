use crate::config::Config;

pub fn setup_logger(config: &Config) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("/tmp/output.log")?)
        .level(if config.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .apply()?;

    Ok(())
}
