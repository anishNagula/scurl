use env_logger::{Builder, Env};

// initialize logger with INFO level as default
pub fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("info")).init();
}