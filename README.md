# traefik-filter

Traefik plugin written in rust. 

This plugin filters incoming requests dependent on the configuration.


## Building

if not already installed, add the wasm-target

```shell
rustup target add wasm32-wasip1
```

Build the plugin with

```shell
make
```

The artifacts are found in target/plugin/

## Installation

Traefik supports a manual installation.

```shell
mkdir -p <traefik>/plugins-local/src/pluginfilter/
cp target/plugin/plugin.wasm <traefik>/plugins-local/src/pluginfilter/

```
Configure the static configuration (and restart traefik)
```yaml
# Static configuration

experimental:
  localPlugins:
    plugindemowasm:
      moduleName: pluginfilter
```
Call the middleware from one of your routers
```yaml
# Dynamic configuration

http:
  routers:
    my-router:
    [...]
      middlewares:
        - pluginfilter
[...]
  middlewares:
    pluginfilter:
      plugin:
        config:
          actions:
            - jail
            - log
          jails:
            - name: short
              duration: 5h
          rules:
            - action: jail.short
              trigger: request.source_ip=='1.1.1.1'
```

```
{
  "actions" : [
    "jail",
    "log"
    ],
  "jails" :[
      { "name":"short",
        "duration":"5h"
      }
    ],
  "rules": [
    {"action":"jail.short", "trigger":"request.source_ip=='1.1.1.1'"}
  ]
}
```