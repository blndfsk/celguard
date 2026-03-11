# celguard

A Traefik plugin written in Rust.

celguard filters incoming requests based on easy-to-configure rules using the Common Expression Language (CEL).

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
    log: warn
    response: { status: 403, body: "forbidden" }

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

## Example Actions

- Log a warning and block requests from certain user agents.
- Return a custom response for requests missing a header.
- Allow or deny requests based on IP, method, or path.

## Limitations

- The "jail" feature is unfinished and not documented here.

## Todo

- Complete jail/ban functionality
- Add more examples and documentation
