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

Rules are written in YAML and use CEL expressions for matching. 

### Defaults
The default action is supplied on all rules without an action.
```yaml
actions:
  - &default
    continue: false
    response: { status: 403, body: "", header: {} }

rules:
  - disabled: false 
    log: off        
    tests: []
    action: *default
```


### Rule Example

```yaml
rules:
  - name: useragent
    tests:
      - request.header.has('user-agent') == false
      - request.header['user-agent'] == []
```
This would test if the header-map does not contain an `user-agent` or if the header value is empty.

### Request Object

You can match on these parts of the request:

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

## CEL Expressions
The heavy lifting is done with the [CEL crate](https://crates.io/crates/cel) which implements the [Cel-Spec](https://github.com/cel-expr/cel-spec).

You can experiment with CEL syntax at [playcel.undistro.io](https://playcel.undistro.io/?content=H4sIAAAAAAAAA1WQwW6DMAyGXyXKoUBVAt0J5T5tu%2B2AtEPpwSNugwQJc8xaadq7j5RObX38%2Fk%2FWb%2F%2FIFnupJeHXhIHVCGxVYCAOHx3bNClUkonVSjTOQkj%2FNYtgkK7BI9wl1gdO9grPXeCQ2o143HiyHoYuybLGyY00wPDmxolvJXTjxDyxihaFOnhPn0ALHJCtN1q8PNcL%2BEYKnXdavNb1e7FV2wUvXa6b4pzzg6cTkEGTj%2BTZa7GzzOP%2BpsTeM136qd630F9QVVblnTYFpByO6KLcTtQXlXoq1b0CbYtjjBu5LtaNnKPLsYM3ON8ZX%2F77B%2FWBVx16AQAA).

## Testing

You can test the plugin via the provided `run.sh` script. This creates a running container for the traefik-server with the plugin configured and the whois-service wired into the router.

```shell
$ ./run.sh whitelist
[lots of logging output]
```

#### Interpreting Example Output

After running the container, you can test the plugin by sending a request to the local server:

```shell
$ curl http://whoami.localhost:8080
Hostname: pensive_curran
IP: 127.0.0.1
IP: ::1
RemoteAddr: [::1]:53364
GET / HTTP/1.1
Host: whoami.localhost:8080
User-Agent: curl/8.18.0
[more output]
```

## Installation

The plugin is available on the [Traefik Plugin Catalog](https://plugins.traefik.io/plugins/69d60c0a4cda2b265225fa6a/celguard).
