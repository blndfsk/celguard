use cel::Program;
use log::LevelFilter;
use serde::de::Deserializer;
use std::{borrow::Cow, str::FromStr};

pub(super) fn deserialize_vec_program<'de, D>(deserializer: D) -> Result<Vec<Program>, D::Error>
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
            let mut programs = Vec::with_capacity(seq.size_hint().unwrap_or_default());
            while let Some(s) = seq.next_element::<Cow<str>>()? {
                programs.push(Program::compile(&s).map_err(serde::de::Error::custom)?);
            }
            Ok(programs)
        }
    }

    deserializer.deserialize_seq(ProgramVisitor)
}

pub(super) fn deserialize_opt_program<'de, D>(deserializer: D) -> Result<Option<Program>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ProgramVisitor;
    impl<'de> serde::de::Visitor<'de> for ProgramVisitor {
        type Value = Program;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a CEL expression string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Program::compile(v).map_err(serde::de::Error::custom)
        }

        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Program::compile(v).map_err(serde::de::Error::custom)
        }
    }
    deserializer.deserialize_str(ProgramVisitor).map(Some)
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
