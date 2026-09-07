use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub(crate) struct Config {
    #[serde(default = "default_status")]
    pub(crate) default_status: i32,
}

/// Returns the default status code if none is specified in the config.
fn default_status() -> i32 {
    400
}
