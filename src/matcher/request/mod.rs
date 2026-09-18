use anyhow::{Error, Result};
use cel::objects::{Key, KeyRef, Map, Value};
use http_wasm_guest::host;
use std::{collections::HashMap, fmt::Display, net::SocketAddr, sync::Arc};

#[cfg(test)]
pub mod tests;

#[derive(PartialEq, Debug)]
pub(crate) struct Request {
    path: Arc<String>,
    method: Arc<String>,
    version: Arc<String>,
    headers: Map, //a Value::Map
    pub source_ip: Arc<String>,
}
static AGENT: KeyRef = KeyRef::String("user-agent");
// e.g. "127.0.0.1 \"GET /apache_pb.gif HTTP/1.0\" \"curl/8.20.0\""
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} \"{} {} {}\"", self.source_ip, self.method, self.path, self.version)?;
        match &self.headers.get(&AGENT) {
            Some(Value::String(ua)) if !ua.is_empty() => write!(f, " \"{}\"", ua),
            _ => write!(f, " -"),
        }
    }
}

impl TryFrom<&host::Request> for Request {
    type Error = anyhow::Error;
    fn try_from(request: &host::Request) -> Result<Self, Self::Error> {
        Ok(Request {
            path: to_string(&request.uri()).into(),
            method: to_string(&request.method()).into(),
            version: to_string(&request.version()).into(),
            source_ip: parse_socket_addr(&request.source_addr()).map(|a| a.to_string().into())?,
            headers: header_value(&request.header),
        })
    }
}

/// Builds the CEL value for the request headers.
fn header_value(header: &host::Header) -> Map {
    Map {
        map: Arc::new(
            header
                .names_iter()
                .map(|name| {
                    let values = header.values(&name);
                    let mut key = to_string(&name);
                    key.make_ascii_lowercase();
                    (
                        Key::String(key.into()),
                        match values.len() {
                            0 => Value::Null,
                            1 => Value::String(to_string(&values[0]).into()),
                            _ => Value::List(Arc::new(
                                values.into_iter().map(|b| to_string(&b).into()).collect(),
                            )),
                        },
                    )
                })
                .collect(),
        ),
    }
}

impl Request {
    pub(super) fn value(&self) -> Value {
        let field = |name: &str, value: Value| (Key::String(String::from(name).into()), value);
        Value::Map(Map {
            map: Arc::new(HashMap::from([
                field("path", Value::String(self.path.clone())),
                field("method", Value::String(self.method.clone())),
                field("version", Value::String(self.version.clone())),
                field("source_ip", Value::String(self.source_ip.clone())),
                field("headers", Value::Map(self.headers.clone())),
            ])),
        })
    }
}
fn to_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).into_owned()
}

/// Parses a socket address from the request source address.
/// valid formats: `ipv4:port`, `[ipv6]:port`, `[ipv6%zone]:port`, `[ipv6]`
/// returns the addr-part as a string
fn parse_socket_addr(input: &[u8]) -> Result<String> {
    let s = str::from_utf8(input)?;

    // Check if it looks like an IPv6 address with a scope zone separator '%...]'
    let addr = match s
        .split_once('%')
        .and_then(|(before, after)| after.split_once(']').map(|(_, tail)| (before, tail)))
    {
        // Reconstruct the string omitting the "%scope" part
        Some((before, tail)) => format!("{}{}{}", before, ']', tail).parse::<SocketAddr>(),
        None => s.parse::<SocketAddr>(),
    };
    addr.map(|a| a.ip()).map(|ip| ip.to_string()).map_err(Error::from)
}
