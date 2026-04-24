use std::borrow::Cow;

use anyhow::Result;

use cel::Program;
use log::LevelFilter;

use serde::de::Deserializer;

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
            s.parse::<LevelFilter>().map_err(serde::de::Error::custom)
        }
    }

    d.deserialize_str(LevelFilterVisitor)
}

pub(super) fn default_level() -> LevelFilter {
    LevelFilter::Off
}

pub(super) fn default_status() -> i32 {
    403
}
