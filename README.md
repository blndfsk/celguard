# celguard

A Traefik plugin written in Rust.

This plugin filters incoming requests based on easy-to-configure rules using the Common Expression Language (CEL).
The intended use is for small sites that want to block or log certain requests without needing a full WAF solution.

## Features

- **Easy configuration:** Write rules using the Common Expression Language (CEL), a familiar and powerful syntax.
- **Request filtering:** Match requests based on any HTTP property (method, path, headers, etc.).
- **Logging:** Actions can specify log levels for matched requests.
- **Custom responses:** Return custom HTTP status, header and body for matched requests.
- **Traefik integration:** Deploy as a WASM plugin for Traefik.

## Configuration

### Rule Example

Rules are written in YAML and use CEL expressions for matching:

```yaml
actions:
  myresponse:
    log: off                                         # off(default), debug, info, warn, error
    response: { status: 403, body: "", header: {} }  # default is status:403, no body, no extra header
    continue: false                                  # true, false(default) - do no continue 

rules:
  - name: useragent
    disabled: false
    tests:
      - request.header.contains('user-agent') == false
      - request.header['user-agent'].matches('(?i)gpt')
    action: myresponse                        # optional
```

### Request Object

You can match on any part of the request:

```yaml
request:
  path: "/foobar"
  method: GET
  version: HTTP/1.1
  header:
    user-agent: curl/123
    host: whoami.localhost:8080
    accept: "*/*"
```

### CEL Expression Example

```c
request.header['user-agent'].matches('(?i)curl')
```

You can experiment with CEL syntax at [playcel.undistro.io](https://playcel.undistro.io/?content=H4sIAAAAAAAAA0WQwU7EMAxEfyXKZWEFScOipfIXwImVqMSlEjKJQyq1SUkcOCD%2BnWb3wNHPI8%2BMf6SlWYLM9FmpsFqRgyqMmcvrxOFqp9XueozyRjpkfIpr5X81jFGIkmq29DatIDz1HQDeGwvWeQ%2BerIGHwxGbrl0GMUqtfErvmEfZ6EIckgNxen4Z2vxFuUwpgngchpM2yjQYCB3ls5sQtVC%2BxQ%2BKDMLWPGtzd7hsQtoiie%2BQcJnUnCzOZ9J3fXcRoLW0cgux1%2Fvmv9VakqOtUfvC7x%2F24TWjDQEAAA%3D%3D).

## Installation

The plugin is available on the [Traefik Plugin Catalog](https://plugins.traefik.io/plugins/69d60c0a4cda2b265225fa6a/celguard).
