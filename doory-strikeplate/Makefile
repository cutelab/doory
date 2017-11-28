all: ../target/mipsel-unknown-linux-musl/release/doory-strikeplate

../target/mipsel-unknown-linux-musl/release/doory-strikeplate: src/main.rs
	cargo build --release --target=mipsel-unknown-linux-musl
	mipsel-openwrt-linux-strip ../target/mipsel-unknown-linux-musl/release/doory-strikeplate

clean:
	cargo clean
