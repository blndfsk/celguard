# celguard

A Traefik plugin written in Rust.

This plugin filters incoming requests based on easy-to-configure rules using the Common Expression Language (CEL).
The intended use is for small sites that want to block or log certain requests without needing a full WAF solution.

## Features

- **Easy configuration:** Write rules using the Common Expression Language (CEL), a familiar and powerful syntax.
- **Request filtering:** Match requests based on any HTTP property (method, path, headers, IP, etc.).
- **Logging:** Actions can specify log levels for matched requests.
- **Custom responses:** Return custom HTTP status and body for matched requests.
- **Traefik integration:** Deploy as a WASM plugin for Traefik.

## Configuration

### Rule Example

Rules are written in YAML and use CEL expressions for matching:

```yaml
actions:
  myresponse:
    log: warn                                            # default: off
    response: { status: 400, body: "bad request" }       # default: status 200, no body
    continue: false                                      # default: false

rules:
  - name: useragent
    tests:
      - request.header.contains('user-agent') == false
      - request.header['user-agent'].matches('(?i)gpt')
    action: myresponse
```

### Request Object

You can match on any part of the request:

```yaml
request:
  source_ip: fe80::a41c:cdff:fec1:736a
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

You can experiment with CEL syntax at [playcel.undistro.io](https://playcel.undistro.io/).

## Installation

Copy the plugin to the `plugins-local/src` directory of your Traefik installation:

plugins-local
└── src
    └── celguard
        ├── .traefik.yml
        ├── LICENSE
        └── plugin.wasm

Add the plugin to your static Traefik configuration:

```yaml
experimental:
  localplugins:
    celguard:
      moduleName: celguard
```

Then you can use the plugin in your dynamic configuration:

```yaml
http:
  middlewares:
    mycelguard:
      plugin:
        celguard:
          rules:
            - name: useragent
              tests:
                - request.header.contains('user-agent') == false
              action: myresponse
          actions:
            myresponse:
              log: warn
              response: { status: 400, body: "bad request" }
```

You need to add the plugin to your Traefik router:
```yaml
http:
  routers:
    myrouter:
      rule: "Host(`whoami.localhost`)"
      service: myservice
      middlewares:
        - mycelguard
```
