# doom-fire-rust

Implementing Fabian Sanglards Doom Fire implementation in rust

## Running the Code

### Running in a native window

Build and run:

```bash
cargo run
```

### Running on the web

This is mostly based on the pixels [minimal-web example](https://github.com/parasyte/pixels/tree/main/examples/minimal-web).

The [getrandom](https://crates.io/crates/getrandom) crate is used instead of the usual [rand](https://crates.io/crates/rand), because it can be run in wasm.

Install the wasm32 target:

```bash
rustup target add wasm32-unknown-unknown
```

Build and start a local server:

```bash
cargo run-wasm --bin doom-fire-rust
```
