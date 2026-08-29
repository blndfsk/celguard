use std::{collections::HashMap, fmt::Display};

use anyhow::Error;
use cel::objects::Opaque;
use http_wasm_guest::host;
use serde::Serialize;

#[derive(Eq, Default, PartialEq, Serialize, Debug)]
pub(super) struct Request {
    path: String,
    method: String,
    version: String,
    header: HashMap<String, Vec<String>>,
}
impl Opaque for Request {
    fn runtime_type_name(&self) -> &str {
        "request"
    }
}

///"GET /apache_pb.gif HTTP/1.0" curl/
impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{} {} {}\" ", self.method, self.path, self.version)?;
        match self.header.get("user-agent") {
            Some(ua) if !ua.is_empty() => {
                let mut sep = std::iter::once("");
                ua.iter().for_each(|elem| {
                    write!(f, "{}{}", sep.next().unwrap_or(", "), elem).unwrap_or_default();
                });
            }
            _ => write!(f, "-")?,
        }
        Ok(())
    }
}

impl Request {
    #[cfg(test)]
    pub(super) fn from_parts(
        path: &str,
        method: &str,
        version: &str,
        header: HashMap<String, Vec<String>>,
    ) -> Self {
        Request { path: path.into(), method: method.into(), version: version.into(), header }
    }
}

impl TryFrom<&host::Request> for Request {
    type Error = Error;

    fn try_from(request: &host::Request) -> std::result::Result<Self, Self::Error> {
        Ok(Request {
            path: request.uri().into(),
            method: request.method().into(),
            version: request.version().into(),
            header: map_header(&request.header),
        })
    }
}

fn map_header(header: &host::Header) -> HashMap<String, Vec<String>> {
    header
        .names_iter()
        .map(|name| {
            let val = header.values_iter(&name).map(|i| i.into()).collect::<Vec<_>>();
            let mut key: String = name.into();
            key.make_ascii_lowercase();
            (key, val)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_user_agent() {
        let req = Request {
            path: "/foo/bar".to_string(),
            method: "GET".to_string(),
            version: "HTTP/1.1".to_string(),
            header: HashMap::from([(
                "user-agent".to_string(),
                vec!["curl/8.0".to_string(), "test".to_string()],
            )]),
        };
        assert_eq!(format!("{}", req), "\"GET /foo/bar HTTP/1.1\" curl/8.0, test");
    }

    #[test]
    fn test_display_without_user_agent() {
        let req = Request {
            path: "/foo/bar".to_string(),
            method: "POST".to_string(),
            version: "HTTP/2.0".to_string(),
            ..Request::default()
        };
        assert_eq!(format!("{}", req), "\"POST /foo/bar HTTP/2.0\" -");
    }

    #[test]
    fn test_display_with_empty_user_agent() {
        let req = Request {
            path: "/".to_string(),
            method: "GET".to_string(),
            version: "HTTP/1.0".to_string(),
            header: HashMap::from([("user-agent".to_string(), vec![])]),
        };
        assert_eq!(format!("{}", req), "\"GET / HTTP/1.0\" -");
    }

    #[test]
    fn test_runtime_type_name() {
        let req = Request { ..Default::default() };
        assert_eq!(req.runtime_type_name(), "request");
    }
}
