use anyhow::Result;
use cel::Program;
use log::LevelFilter;
use serde::Deserialize;
use serde::de::Deserializer;

pub(super) fn deserialize_filters<'de, D>(d: D) -> Result<Vec<Program>, D::Error>
where
    D: Deserializer<'de>,
{
    let filters: Vec<String> = Vec::deserialize(d)?;
    filters
        .iter()
        .map(|s| Program::compile(s).map_err(serde::de::Error::custom))
        .collect()
}

pub(super) fn deserialize_level<'de, D>(d: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub(super) fn default_level() -> LevelFilter {
    LevelFilter::Off
}

pub(super) fn default_status() -> i32 {
    403
}
