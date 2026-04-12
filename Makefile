.PHONY: build serve clean

build:
	wasm-pack build --target web

serve: build
	@echo "Serving at http://localhost:8090/www/"
	python3 -m http.server 8090

clean:
	cargo clean
	rm -rf pkg
