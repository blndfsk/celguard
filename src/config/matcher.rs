use crate::config::{deserialize, rule::Rule};
use anyhow::Result;
use cel::Program;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default, deserialize_with = "deserialize::deserialize_opt_program")]
    pub(crate) source_ip: Option<Program>,
    pub(crate) rules: Vec<Rule>,
}

impl Config {
    /// Rule names are rule identities; duplicates would make matching ambiguous.
    pub(crate) fn validate(&self) -> Result<()> {
        let mut seen: Vec<&str> = Vec::with_capacity(self.rules.len());
        for rule in &self.rules {
            if seen.iter().any(|name| *name == rule.name) {
                anyhow::bail!("duplicate rule name: {}", rule.name);
            }
            seen.push(&rule.name);
        }
        Ok(())
    }
}
