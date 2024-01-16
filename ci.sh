set -euxo pipefail

cd macros
cargo build
cargo clean

cd ../tiny-serde

cargo build --no-default-features
cargo build --release
cargo test --no-default-features
cargo test --features=derive
cargo test

cargo clean
