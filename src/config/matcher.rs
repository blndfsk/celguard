use crate::config::{
    deserialize,
    rule::{Action, Response, Rule},
};
use anyhow::Result;
use cel::Program;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(deserialize_with = "deserialize::deserialize_opt_program")]
    pub(crate) source_ip: Option<Program>,
    /// used for matching rules without action
    pub(crate) default_action: Action,
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
impl Default for Config {
    fn default() -> Self {
        Self { source_ip: None, default_action: DEFAULT_ACTION, rules: Vec::new() }
    }
}
/// Default action used when a rule matches without an explicit action.
const DEFAULT_ACTION: Action = Action {
    response: Some(Response { status: 400, body: None, header: None }),
    r#continue: false,
};
