all: ../target/mipsel-unknown-linux-musl/release/doory-reader
.PHONY: debug clean

../target/mipsel-unknown-linux-musl/release/doory-reader: src/main.rs
	cargo build --release --target=mipsel-unknown-linux-musl
	mipsel-openwrt-linux-strip ../target/mipsel-unknown-linux-musl/release/doory-reader

debug: ../target/mipsel-unknown-linux-musl/debug/doory-reader

../target/mipsel-unknown-linux-musl/debug/doory-reader: src/main.rs
	cargo build --target=mipsel-unknown-linux-musl
	mipsel-openwrt-linux-strip ../target/mipsel-unknown-linux-musl/debug/doory-reader

clean:
	cargo clean
