all: ../target/mipsel-unknown-linux-musl/release/doory-keypad

../target/mipsel-unknown-linux-musl/release/doory-keypad: src/main.rs
	cargo build --release --target=mipsel-unknown-linux-musl
	mipsel-openwrt-linux-strip ../target/mipsel-unknown-linux-musl/release/doory-keypad

clean:
	cargo clean
