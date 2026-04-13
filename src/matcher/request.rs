use std::{collections::HashMap, fmt::Display};

use anyhow::{Error, Result};
use cel::objects::Opaque;
use http_wasm_guest::host;
use serde::Serialize;

#[derive(Eq, PartialEq, Serialize, Debug)]
pub struct Request {
    path: String,
    method: String,
    version: String,
    header: HashMap<String, String>,
}
impl Opaque for Request {
    fn runtime_type_name(&self) -> &str {
        "request"
    }
}

///"GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\"{} {} {}\" {}",
            self.method,
            self.path,
            self.version,
            self.header
                .get("user-agent")
                .map(|s| s.as_str())
                .unwrap_or_else(|| "-")
        )
    }
}

impl Request {
    pub fn try_from_host(request: &host::Request) -> Result<Self> {
        Self::try_from(request)
    }
    #[cfg(test)]
    pub fn from_parts(
        path: &str,
        method: &str,
        version: &str,
        header: HashMap<String, String>,
    ) -> Self {
        Request {
            path: path.to_string(),
            method: method.to_string(),
            version: version.to_string(),
            header,
        }
    }
}

impl TryFrom<&host::Request> for Request {
    type Error = Error;

    fn try_from(request: &host::Request) -> std::result::Result<Self, Self::Error> {
        Ok(Request {
            path: request.uri().to_string(),
            method: request.method().to_string(),
            version: request.version().to_string(),
            header: map_header(request.header()),
        })
    }
}

fn map_header(header: &host::Header) -> HashMap<String, String> {
    header
        .values()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string().to_lowercase(),
                value
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })
        .collect()
}
