// cargo run --example breakout


// cargo build --target wasm32-unknown-unknown --release


// wasm-tools component new ./target/wasm32-unknown-unknown/release/wgpu_examples_wasi.wasm -o ./target/examples_component.wasm


// cargo build --release --example breakout --target wasm32-unknown-unknown


// cargo build --profile wasm-release --example breakout --target wasm32-unknown-unknown



cargo build --example breakout --target wasm32-unknown-unknown --release
// wasm-tools component new ./target/wasm32-unknown-unknown/release/examples/breakout.wasm -o ./target/breakout_component.wasm


cargo run --manifest-path ../replace-imports/Cargo.toml -- -i ./target/wasm32-unknown-unknown/release/examples/breakout.wasm -o ./target/breakout_removed_imports.wasm


wasm-tools component new ./target/breakout_removed_imports.wasm -o ./target/breakout_component.wasm








look at rand crate


// compiler crashing on:
// solution? https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip2.html#building-the-target
// export WASI_SDK_PATH="/opt/wasi-sdk"
// cargo +nightly build --example breakout --target wasm32-wasip2 --release

WASI_SDK_PATH=`pwd` -${WASI_ARCH}-${WASI_OS}

















cargo build --example breakout --target wasm32-unknown-unknown
cargo run --manifest-path ../replace-imports/Cargo.toml -- -i ./target/wasm32-unknown-unknown/debug/examples/breakout.wasm -o ./target/breakout_removed_imports.wasm
wasm-tools component new ./target/breakout_removed_imports.wasm -o ./target/breakout_component.wasm
