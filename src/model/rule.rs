use crate::model::Action;

use cel::Program;
use log::LevelFilter;
use serde::{Deserialize, Deserializer};
use serde_saphyr::RcAnchor;
use std::borrow::Cow;
use std::str::FromStr;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rule {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) disabled: bool,
    #[serde(deserialize_with = "deserialize_level", default = "default_level")]
    pub(crate) log: LevelFilter,
    #[serde(default, deserialize_with = "deserialize_program")]
    pub(crate) tests: Vec<Program>,
    pub(crate) action: Option<RcAnchor<Action>>,
}

impl Rule {
    #[cfg(test)]
    pub(crate) fn from_parts(
        name: &str,
        disabled: bool,
        log: LevelFilter,
        tests: Vec<&str>,
        action: Option<RcAnchor<Action>>,
    ) -> Rule {
        Rule {
            name: name.to_string(),
            disabled,
            log,
            tests: tests.iter().flat_map(|s| Program::compile(s)).collect::<Vec<_>>(),
            action,
        }
    }
}

pub(super) fn deserialize_program<'de, D>(deserializer: D) -> Result<Vec<Program>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ProgramVisitor;
    impl<'de> serde::de::Visitor<'de> for ProgramVisitor {
        type Value = Vec<Program>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a sequence of CEL expression strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut programs = Vec::with_capacity(seq.size_hint().unwrap_or(32));
            while let Some(s) = seq.next_element::<Cow<str>>()? {
                programs.push(Program::compile(&s).map_err(serde::de::Error::custom)?);
            }
            Ok(programs)
        }
    }

    deserializer.deserialize_seq(ProgramVisitor)
}

pub(super) fn deserialize_level<'de, D>(d: D) -> Result<LevelFilter, D::Error>
where
    D: Deserializer<'de>,
{
    struct LevelFilterVisitor;

    impl<'de> serde::de::Visitor<'de> for LevelFilterVisitor {
        type Value = LevelFilter;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a string representing a log level")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            LevelFilter::from_str(s).map_err(serde::de::Error::custom)
        }
    }

    d.deserialize_str(LevelFilterVisitor)
}

pub(super) fn default_level() -> LevelFilter {
    LevelFilter::Off
}
