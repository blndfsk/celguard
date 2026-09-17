use anyhow::{Error, Result};
use cel::objects::{Key, KeyRef, Map, Value};
use http_wasm_guest::host;
use std::{
    collections::HashMap,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

#[derive(PartialEq, Debug)]
pub(super) struct Request {
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
fn parse_socket_addr(input: &[u8]) -> Result<IpAddr> {
    let s = str::from_utf8(input)?;

    // Check if it looks like an IPv6 address with a scope zone separator '%'
    let addr = if let (Some(p), Some(b)) = (s.find('%'), s.find(']')) {
        // Reconstruct the string omitting the "%scope" part
        let clean = format!("{}{}", &s[..p], &s[b..]);
        clean.parse::<SocketAddr>()
    } else {
        s.parse::<SocketAddr>()
    };
    addr.map(|a| a.ip()).map_err(Error::from)
}

#[cfg(test)]
impl Request {
    pub(super) fn get_request() -> Request {
        Request {
            path: "/".to_string().into(),
            method: "GET".to_string().into(),
            version: "HTTP/1.1".to_string().into(),
            headers: test_headers(&[("user-agent", &["curl/8.0"]), ("x-real-ip", &["1.1.1.1"])]),
            source_ip: "127.0.0.1".to_string().into(),
        }
    }

    pub(super) fn post_request() -> Request {
        Request {
            path: "/".to_string().into(),
            method: "POST".to_string().into(),
            version: "HTTP/1.1".to_string().into(),
            headers: test_headers(&[("user-agent", &["curl/8.0"]), ("x-real-ip", &["1.1.1.1"])]),
            source_ip: "127.0.0.1".to_string().into(),
        }
    }
}

/// Builds a CEL header value from (name, values) test pairs.
#[cfg(test)]
fn test_headers(pairs: &[(&str, &[&str])]) -> Map {
    let map = pairs
        .iter()
        .map(|(name, values)| {
            (
                Key::String(name.to_string().into()),
                match values.len() {
                    0 => Value::Null,
                    1 => Value::String(values[0].to_string().into()),
                    _ => Value::List(Arc::new(
                        values.iter().map(|v| Value::String(v.to_string().into())).collect(),
                    )),
                },
            )
        })
        .collect();
    Map { map: Arc::new(map) }
}

#[cfg(test)]
mod tests {

    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_request() {
        let req = Request {
            path: "/foo/bar".to_string().into(),
            method: "GET".to_string().into(),
            version: "HTTP/1.1".to_string().into(),
            headers: test_headers(&[("user-agent", &["curl/8.0"])]),
            source_ip: "127.0.0.1".to_string().into(),
        };
        assert_eq!(format!("{}", req), "127.0.0.1 \"GET /foo/bar HTTP/1.1\" \"curl/8.0\"");
    }

    #[test]
    fn test_display_without_user_agent() {
        let req = Request {
            path: "/foo/bar".to_string().into(),
            method: "POST".to_string().into(),
            version: "HTTP/2.0".to_string().into(),
            headers: test_headers(&[]),
            source_ip: "127.0.0.1".to_string().into(),
        };
        assert_eq!(format!("{}", req), "127.0.0.1 \"POST /foo/bar HTTP/2.0\" -");
    }

    #[test]
    fn test_display_with_empty_user_agent() {
        let req = Request {
            path: "/".to_string().into(),
            method: "GET".to_string().into(),
            version: "HTTP/1.0".to_string().into(),
            headers: test_headers(&[("user-agent", &[])]),
            source_ip: "127.0.0.1:123".to_string().into(),
        };
        assert_eq!(format!("{}", req), "127.0.0.1:123 \"GET / HTTP/1.0\" -");
    }

    #[test]
    fn test_parse_socket_addr() -> TestResult {
        assert!(parse_socket_addr(b"127.0.0.1:80").map(|a| a.to_string() == "127.0.0.1")?);
        assert!(parse_socket_addr(b"203.0.113.7:443").map(|a| a.to_string() == "203.0.113.7")?);
        assert!(parse_socket_addr(b"[::1]:443").map(|a| a.to_string() == "::1")?);
        assert!(parse_socket_addr(b"[2001:db8::1]:8080").map(|a| a.to_string() == "2001:db8::1")?);
        assert!(parse_socket_addr(b"[fe80::1%eth0]:8080").map(|a| a.to_string() == "fe80::1")?);
        Ok(())
    }

    #[test]
    fn test_parse_socket_addr_invalid() {
        assert!(parse_socket_addr(b"").is_err());
        assert!(parse_socket_addr(b"1.2.3:80").is_err());
        assert!(parse_socket_addr(b"1.2.3.4:").is_err());
        assert!(parse_socket_addr(b"1.2.3.4:1:2").is_err());
        assert!(parse_socket_addr(b"::1:80").is_err()); // unbracketed ipv6
        assert!(parse_socket_addr(b"[::1").is_err()); // missing ']'
        assert!(parse_socket_addr(b"[]:80").is_err());
        assert!(parse_socket_addr(b"[not-ipv6]:80").is_err());
    }
}
