use std::sync::OnceLock;

static DEFAULT_LOGGER: OnceLock<bool> = OnceLock::new();

pub fn setup_stdout_logger() {
    DEFAULT_LOGGER.get_or_init(|| {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();
        true
    });
}
