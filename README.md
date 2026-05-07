# celguard

This traefik plugin filters incoming requests based on easy-to-configure rules using the Common Expression Language (CEL).
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
    action: myresponse                               # optional, default action is 403 without body
```

### Request Object

You can match on any part of the request:

```yaml
request:
    path: /.foobar
    method: GET
    version: HTTP/1.1
    header:
        host: [whoami.localhost:8080]
        user-agent: [curl/8.20.0]
        accept: ["*/*"]
```

### CEL Expression Example

```c
request.path.startsWith('/.')
```

You can experiment with CEL syntax at [playcel.undistro.io](https://playcel.undistro.io/?content=H4sIAAAAAAAAA1WQwW6DMAyGXyXKoUBVAt0J5T5tu%2B2AtEPpwSNugwQJc8xaadq7j5RObX38%2Fk%2FWb%2F%2FIFnupJeHXhIHVCGxVYCAOHx3bNClUkonVSjTOQkj%2FNYtgkK7BI9wl1gdO9grPXeCQ2o143HiyHoYuybLGyY00wPDmxolvJXTjxDyxihaFOnhPn0ALHJCtN1q8PNcL%2BEYKnXdavNb1e7FV2wUvXa6b4pzzg6cTkEGTj%2BTZa7GzzOP%2BpsTeM136qd630F9QVVblnTYFpByO6KLcTtQXlXoq1b0CbYtjjBu5LtaNnKPLsYM3ON8ZX%2F77B%2FWBVx16AQAA).

## Installation

The plugin is available on the [Traefik Plugin Catalog](https://plugins.traefik.io/plugins/69d60c0a4cda2b265225fa6a/celguard).
