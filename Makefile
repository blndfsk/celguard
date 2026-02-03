.PHONY: build clean

plugin: clean build
	mkdir target/plugin
	cp .traefik.yml target/plugin/
	cp LICENSE target/plugin/
	cp target/wasm32-wasip1/release/traefik-filter.wasm target/plugin/plugin.wasm
	zip -rj target/plugin.zip target/plugin/
	
build:
	cargo build --target wasm32-wasip1 --release

clean:
	cargo clean

