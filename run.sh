#!/usr/bin/env bash
set -eu -o pipefail

plugin="celguard"
pod=$(podman pod create -p 8080:8080)

function cleanup()
{
    podman pod rm -f $pod
    podman image rm localhost/$plugin
}

trap 'cleanup' EXIT HUP INT TERM

cargo build --release --target wasm32-wasip1

TRAEFIK_ROOT=/opt/traefik
ROOT_DIR=$TRAEFIK_ROOT/plugins-local/src/$plugin

container=$(buildah from traefik:v3.6)
buildah copy $container target/wasm32-wasip1/release/$plugin.wasm $ROOT_DIR/plugin.wasm
buildah copy $container .traefik.yml $ROOT_DIR/.traefik.yml
buildah copy $container config/ $ROOT_DIR/config/
buildah config --workingdir $TRAEFIK_ROOT $container
buildah commit $container localhost/$plugin
buildah rm $container

traefik_container=localhost/$plugin
traefik_parameter="--experimental.localplugins.$plugin.modulename=$plugin --experimental.localplugins.$plugin.settings.mounts=$ROOT_DIR/config/"

podman run -d --pod $pod --replace --name whoami \
    --label 'traefik.http.routers.whoami.rule=Host(`whoami.localhost`)' \
    --label "traefik.http.routers.whoami.middlewares=$plugin" \
    --label "traefik.http.routers.whoami.service=whoami" \
    --label 'traefik.http.routers.echo.rule=Host(`echo.localhost`)' \
    --label "traefik.http.routers.echo.service=whoami" \
    --label "traefik.http.middlewares.$plugin.plugin.$plugin.paths[0]=$ROOT_DIR/config/rules.yaml" \
    --label "traefik.http.services.whoami.loadbalancer.server.url=http://localhost:8081" \
    traefik/whoami -port 8081

    podman run -it --rm --pod $pod \
        --volume /run/user/${UID}/podman/podman.sock:/var/run/docker.sock \
        $traefik_container --entrypoints.web.address=:8080 --providers.docker=true --log.level=INFO $traefik_parameter
