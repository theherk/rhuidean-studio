.PHONY: build-web build-tui serve clean

build-web:
	wasm-pack build --target web web

build-tui:
	cargo build --release -p rhuidean-studio

serve: build-web
	@echo "Serving at http://localhost:8090/web/www/"
	python3 -m http.server 8090

clean:
	cargo clean
	rm -rf web/pkg
