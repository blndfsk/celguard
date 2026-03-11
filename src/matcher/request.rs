use std::{collections::HashMap, fmt::Display, sync::LazyLock};

use anyhow::{Error, Result};
use cel::objects::Opaque;
use http_wasm_guest::host;
use regex::Regex;
use serde::Serialize;

#[derive(Eq, PartialEq, Serialize, Debug)]
pub struct Request {
    source_ip: String,
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

///127.0.0.1 "GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} \"{} {} {}\" {}",
            self.source_ip,
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
        source_ip: &str,
        path: &str,
        method: &str,
        version: &str,
        header: HashMap<String, String>,
    ) -> Self {
        Request {
            source_ip: source_ip.to_string(),
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
            source_ip: map_source_ip(request).unwrap_or_default(),
            path: request.uri().to_string(),
            method: request.method().to_string(),
            version: request.version().to_string(),
            header: map_header(request.header()),
        })
    }
}

///matches
/// - `[fe80::680c:59ff:fe17:ffce%enp5s0]:42026`
static IP_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[?([a-f\d\.:]+)%?(\w*)\]?:(\d+)").unwrap());

fn map_source_ip(request: &host::Request) -> Option<String> {
    let addr = Some(request.source_addr())
        .or_else(|| request.header().get(b"x-real-ip"))
        .or_else(|| request.header().get(b"x-forwarded-for"))
        .unwrap_or_default();

    IP_REGEX
        .captures(addr.to_str().ok()?)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn map_header(header: &host::Header) -> HashMap<String, String> {
    let header_map: HashMap<String, String> = header
        .values()
        .iter()
        .map(|(key, value)| {
            (
                key.to_string().to_lowercase(),
                value.iter().map(|i| i.to_string()).collect(),
            )
        })
        .collect();
    header_map
}
