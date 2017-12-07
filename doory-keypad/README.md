# doory-keypad

This contains code that runs on a "blue pill" dev board which reads
a keypad and sends the resulting data via usbcdc. `doory-reader`
is the consumer of this output.

## Install this version of Rust
curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly

## Notes for installing xargo

```
cargo install xargo --vers 0.3.8 -f
```
