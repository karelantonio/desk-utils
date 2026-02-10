PREFIX=/usr/local

.PHONY: install clean

clean:
	cargo clean

install: target/release/bright
	install -m 6711 target/release/bright $(PREFIX)/bin/bright

target/release/bright:
	cargo build --release --package bright
