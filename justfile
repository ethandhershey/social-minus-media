set dotenv-load := true

cargo_run := "cargo run --package server"

run:
    {{cargo_run}}

check:
    cargo check --workspace

test:
    cargo test --workspace --features domain/test-utils

fmt:
    cargo +nightly fmt --all

test-all:
    cargo test --workspace --features application/test-utils

flamegraph:
    cargo flamegraph --package server --profile release-debug

console:
    RUSTFLAGS="--cfg tokio_unstable" {{cargo_run}} --features server/tokio-console --profile release-debug

console-release:
    RUSTFLAGS="--cfg tokio_unstable" {{cargo_run}} --features server/tokio-console --release

migrate:
    sqlx migrate run

prepare:
    cargo sqlx prepare --workspace
