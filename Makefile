PREFIX=/usr/local

.PHONY: install clean

clean:
	cargo clean

install: target/release/bright target/release/tyla
	install -m 6711 target/release/bright $(PREFIX)/bin/bright
	install -m 755 target/release/tyla $(PREFIX)/bin/tyla

target/release/bright:
	cargo build --release --package bright

target/release/tyla:
	cargo build --release --package tyla
