use anyhow::{Error, Result};
use cel::objects::{Key, Map, Value};
use http_wasm_guest::host;
use std::{collections::HashMap, fmt::Display, net::IpAddr, str::FromStr, sync::Arc};

#[derive(Eq, PartialEq, Debug)]
pub(super) struct Request {
    path: Arc<String>,
    method: Arc<String>,
    version: Arc<String>,
    headers: HashMap<Arc<String>, Vec<Arc<String>>>,
    pub source_ip: Arc<String>,
}

// e.g. "127.0.0.1 \"GET /apache_pb.gif HTTP/1.0\" \"curl/8.20.0\""
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} \"{} {} {}\"", self.source_ip, self.method, self.path, self.version)?;
        match self
            .headers
            .iter()
            .find(|(k, _)| k.as_str() == "user-agent")
            .and_then(|(_, v)| v.first())
            .filter(|v| !v.is_empty())
        {
            Some(ua) => write!(f, " \"{}\"", ua),
            None => write!(f, " -"),
        }
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
            headers: map_header(&request.header),
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
    /// Builds the CEL value for this request. Only `Arc` reference counts are
    /// bumped — no string data is copied.
    pub(super) fn value(&self) -> Value {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| {
                (
                    Key::String(k.clone()),
                    match v.len() {
                        0 => Value::Null,
                        1 => Value::String(v[0].clone()),
                        _ => Value::List(Arc::new(v.iter().cloned().map(Value::String).collect())),
                    },
                )
            })
            .collect();
        let field = |name: &str, value: Value| (Key::String(String::from(name).into()), value);
        Value::Map(Map {
            map: Arc::new(HashMap::from([
                field("path", Value::String(self.path.clone())),
                field("method", Value::String(self.method.clone())),
                field("version", Value::String(self.version.clone())),
                field("source_ip", Value::String(self.source_ip.clone())),
                field("headers", Value::Map(Map { map: Arc::new(headers) })),
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
            headers: HashMap::from([
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
            headers: HashMap::from([
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
            headers: HashMap::from([(
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
            headers: HashMap::new(),
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
            headers: HashMap::from([("user-agent".to_string().into(), vec![])]),
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
