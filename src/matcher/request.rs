use std::{collections::HashMap, fmt::Display};

use anyhow::{Error, Result};
use cel::objects::Opaque;
use http_wasm_guest::host;
use serde::Serialize;

#[derive(Eq, PartialEq, Serialize, Debug)]
pub(super) struct Request {
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
                .filter(|s| !s.is_empty())
                .map_or("-", |s| s.as_str())
        )
    }
}

impl Request {
    pub(super) fn try_from_host(request: &host::Request) -> Result<Self> {
        Self::try_from(request)
    }
    #[cfg(test)]
    pub(super) fn from_parts(
        path: &str,
        method: &str,
        version: &str,
        header: HashMap<String, String>,
    ) -> Self {
        Request {
            path: path.to_string(),
            method: method.to_string(),
            version: version.to_string(),
            header: header,
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
            header: map_header(&request.header),
        })
    }
}

fn map_header(header: &host::Header) -> HashMap<String, String> {
    header
        .names_iter()
        .map(|key| {
            (
                key.to_string().to_lowercase(),
                header
                    .values_iter(&key)
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_user_agent() {
        let req = Request::from_parts(
            "/foo/bar",
            "GET",
            "HTTP/1.1",
            HashMap::from([("user-agent".to_string(), "curl/8.0".to_string())]),
        );
        assert_eq!(format!("{}", req), "\"GET /foo/bar HTTP/1.1\" curl/8.0");
    }

    #[test]
    fn test_display_without_user_agent() {
        let req = Request::from_parts("/foo/bar", "POST", "HTTP/2.0", HashMap::new());
        assert_eq!(format!("{}", req), "\"POST /foo/bar HTTP/2.0\" -");
    }

    #[test]
    fn test_display_with_empty_user_agent() {
        let req = Request::from_parts(
            "/",
            "GET",
            "HTTP/1.0",
            HashMap::from([("user-agent".to_string(), "".to_string())]),
        );
        assert_eq!(format!("{}", req), "\"GET / HTTP/1.0\" -");
    }

    #[test]
    fn test_runtime_type_name() {
        let req = Request::from_parts("/", "GET", "HTTP/1.1", HashMap::new());
        assert_eq!(req.runtime_type_name(), "request");
    }
}
