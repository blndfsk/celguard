# celguard

This traefik plugin filters incoming requests based on easy-to-configure rules using the Common Expression Language (CEL).
The intended use is for small sites that want to block or log certain requests without needing a full WAF solution.

## Features

- **Easy configuration:** Write rules using the Common Expression Language (CEL), a familiar and powerful syntax.
- **Request filtering:** Match requests based on any HTTP property (method, path, headers, etc.).
- **Client IP resolution:** Resolve the real client IP from a header (e.g. `X-Real-IP`) so rules work correctly behind proxies and load balancers.
- **Per-rule control:** Disable individual rules and attach a log level to each one without touching the others.
- **Reusable actions:** Define an action once and reference it from multiple rules with YAML anchors.
- **Logging:** Rules can specify log levels for matched requests.
- **Custom responses:** Return custom HTTP status, header and body for matched requests.
- **Traefik integration:** Deploy as a WASM plugin for Traefik.

## Configuration

Rules are written in YAML and use CEL expressions for matching. The configuration has two top-level sections:

- **`plugin`** — general plugin settings (optional).
- **`matcher`** — the client IP expression and the list of rules.

### Plugin Settings

```yaml
plugin:
  default_status: 400
```

- **`default_status`** (default `400`) — the HTTP status used by the default action and by any response that omits a status.

### Client IP

By default the client address is the socket address of the connection. Behind a proxy or load balancer that value is not the real client IP, so you can point the plugin at a header instead with a CEL expression. The expression is evaluated against the request, must return a string, and its result is exposed to your rules as `request.source_ip`.

```yaml
matcher:
  source_ip: request.header['x-real-ip'].get_first()
```

### Defaults

The default action is supplied on all rules without an action: it returns `plugin.default_status` with no body and no headers. If `plugin.default_status` is not set, the default is `400`.

### Rules

A rule is a name, a list of CEL `tests`, and an optional `action`:

```yaml
matcher:
  rules:
    - name: useragent
      log: warn
      tests:
        - request.header.contains('user-agent') == false
```

This would test if the header-map does not contain an `user-agent`.

| Field | Default | Description |
| --- | --- | --- |
| `name` | required | Human readable name, used in log output. |
| `disabled` | `false` | Skip this rule entirely when set. |
| `log` | `off` | Log level to use when the rule matches. |
| `tests` | `[]` | CEL expressions to evaluate. The rule matches when **any** test is true; an empty list matches every request. |
| `action` | optional | The response to produce when the rule matches. |

Rules are evaluated in order and the **first matching rule** determines the outcome.

### Actions

An action describes the response to produce:

```yaml
action:
  continue: false
  response:
    status: 403
    header: { allow: "GET, HEAD, OPTIONS" }
    body: forbidden
```

- **`continue`** (default `false`) — when `false`, Traefik stops processing and this is the final response. When `true`, request handling continues after the middleware.
- **`response.status`** — HTTP status, defaults to `plugin.default_status`.
- **`response.header`** — additional response headers.
- **`response.body`** — the response body.

Actions can be defined once and shared between rules with YAML anchors:

```yaml
matcher:
  rules:
    - name: block-admin
      tests:
        - request.path.startsWith('/admin')
      action: &block
        response: { status: 403 }
    - name: block-api
      tests:
        - request.path.startsWith('/api')
      action: *block
```

### Request Object

You can match on these parts of the request:

```yaml
request:
  path: /.foobar
  method: GET
  version: HTTP/1.1
  source_ip: 192.0.2.1
  header:
    host: [whoami.localhost:8080]
    user-agent: [curl/8.20.0]
    accept: ["*/*"]
```

`source_ip` is the resolved client IP, see [Client IP](#client-ip).

## CEL Expressions
The heavy lifting is done with the [CEL crate](https://crates.io/crates/cel) which implements the [Cel-Spec](https://github.com/cel-expr/cel-spec).

Header values are lists and a few helper functions are available to work with them:

| Function | Description |
| --- | --- |
| `contains(name)` | `true` if the header map has an entry for `name`. |
| `get_first()` | The first value of a header, or `null` if it is not present. |
| `lower()` | Lower-case a string. |
| `trim()` | Trim surrounding whitespace from a string. |

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
