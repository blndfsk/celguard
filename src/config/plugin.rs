use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) default_status: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self { default_status: 400 }
    }
}
