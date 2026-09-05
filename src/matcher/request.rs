use std::{collections::HashMap, fmt::Display, net::IpAddr, str::FromStr, sync::Arc};

use anyhow::{Error, Result};
use cel::objects::{Key, Map, Value};
use http_wasm_guest::host;

#[derive(Eq, PartialEq, Debug)]
pub(super) struct Request {
    path: Arc<String>,
    method: Arc<String>,
    version: Arc<String>,
    header: HashMap<Arc<String>, Vec<Arc<String>>>,
    pub source_ip: Arc<String>,
}

///"GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} \"{} {} {}\" ", self.source_ip, self.method, self.path, self.version)?;
        match self.header("user-agent") {
            Some(ua) if !ua.is_empty() => {
                let mut sep = std::iter::once("");
                ua.iter().for_each(|elem| {
                    write!(f, "{}\"{}\"", sep.next().unwrap_or(", "), elem).unwrap_or_default();
                });
            }
            _ => write!(f, "-")?,
        }
        Ok(())
    }
}

impl From<&host::Request> for Request {
    fn from(request: &host::Request) -> Self {
        Request {
            path: to_string(&request.uri()).into(),
            method: to_string(&request.method()).into(),
            version: to_string(&request.version()).into(),
            source_ip: parse_socket_addr(&request.source_addr())
                .map(|a| a.to_string().into())
                .unwrap_or_default(),
            header: map_header(&request.header),
        }
    }
}

fn map_header(header: &host::Header) -> HashMap<Arc<String>, Vec<Arc<String>>> {
    header
        .names_iter()
        .map(|name| {
            let val = header.values_iter(&name).map(|i| to_string(&i).into()).collect::<Vec<_>>();
            let mut key = to_string(&name);
            key.make_ascii_lowercase();
            (key.into(), val)
        })
        .collect()
}

impl Request {
    pub(super) fn header(&self, name: &str) -> Option<&Vec<Arc<String>>> {
        self.header.iter().find_map(|(k, v)| (k.as_str() == name).then_some(v))
    }

    /// Builds the CEL value for this request. Only `Arc` reference counts are
    /// bumped — no string data is copied.
    pub(super) fn value(&self) -> Value {
        let header = self
            .header
            .iter()
            .map(|(k, v)| {
                (
                    Key::String(k.clone()),
                    Value::List(Arc::new(v.iter().cloned().map(Value::String).collect())),
                )
            })
            .collect();
        let field = |name: &str, value: &Arc<String>| {
            (Key::String(String::from(name).into()), Value::String(value.clone()))
        };
        Value::Map(Map {
            map: Arc::new(HashMap::from([
                field("path", &self.path),
                field("method", &self.method),
                field("version", &self.version),
                field("source_addr", &self.source_ip),
                (
                    Key::String(String::from("header").into()),
                    Value::Map(Map { map: Arc::new(header) }),
                ),
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
    // bracketed form: `[ipv6]:port`, `[ipv6%zone]:port`, or `[ipv6]`
    let addr = if input.first() == Some(&b'[') {
        let (inner, _) =
            byte_split(&input[1..], b']').ok_or_else(|| Error::msg("right bracket missing"))?;
        match byte_split(inner, b'%') {
            Some((ip, zone)) if !zone.is_empty() => Ok(ip),
            Some((_, _)) => Err(Error::msg("zone missing")),
            None => Ok(inner),
        }
    } else {
        match byte_split(input, b':') {
            Some((ip, port)) if !port.is_empty() && port.iter().all(u8::is_ascii_digit) => Ok(ip),
            Some((_, _)) => Err(Error::msg("port missing")),
            None => Ok(input),
        }
    };
    IpAddr::from_str(str::from_utf8(addr?)?)
        .map_err(|e| Error::msg(format!("invalid ip address: {}", e)))
}

fn byte_split(slice: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    slice.iter().position(|&b| b == delim).map(|i| (&slice[..i], &slice[i + 1..]))
}

#[cfg(test)]
impl Request {
    pub(super) fn get_request() -> Request {
        Request {
            path: "/".to_string().into(),
            method: "GET".to_string().into(),
            version: "HTTP/1.1".to_string().into(),
            header: HashMap::from([
                ("user-agent".to_string().into(), vec!["curl/8.0".to_string().into()]),
                ("x-real-ip".to_string().into(), vec!["1.1.1.1".to_string().into()]),
            ]),
            source_ip: "127.0.0.1".to_string().into(),
        }
    }

    pub(super) fn post_request() -> Request {
        Request {
            path: "/".to_string().into(),
            method: "POST".to_string().into(),
            version: "HTTP/1.1".to_string().into(),
            header: HashMap::from([
                ("user-agent".to_string().into(), vec!["curl/8.0".to_string().into()]),
                ("x-real-ip".to_string().into(), vec!["1.1.1.1".to_string().into()]),
            ]),
            source_ip: "127.0.0.1".to_string().into(),
        }
    }
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
            header: HashMap::from([(
                "user-agent".to_string().into(),
                vec!["curl/8.0".to_string().into()],
            )]),
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
            header: HashMap::new(),
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
            header: HashMap::from([("user-agent".to_string().into(), vec![])]),
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
        assert!(parse_socket_addr(b"[::1]").map(|a| a.to_string() == "::1")?);
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
        assert!(parse_socket_addr(b"[::1%]:80").is_err()); // empty zone
    }
}
