use crate::config::generate_config;

pub fn init() -> anyhow::Result<()> {
    generate_config()
}
