set dotenv-load := true

cargo_run := "cargo run --package server"
pg_ctl_opts := "-o \"-c unix_socket_directories=$PGHOST -c listen_addresses=\""

run:
    {{ cargo_run }}

check:
    cargo check --workspace

test:
    cargo test --workspace --features domain/test-utils

fmt:
    cargo +nightly fmt --all

test-all:
    cargo test --workspace --features application/test-utils

build-cross:
    nix --extra-experimental-features 'nix-command flakes' develop .#cross --command \
        env SQLX_OFFLINE=true \
        cargo build --release --target aarch64-unknown-linux-musl

flamegraph:
    cargo flamegraph --package server --profile release-debug

console:
    RUSTFLAGS="--cfg tokio_unstable" {{ cargo_run }} --features server/tokio-console --profile release-debug

console-release:
    RUSTFLAGS="--cfg tokio_unstable" {{ cargo_run }} --features server/tokio-console --release

migrate:
    sqlx migrate run

prepare:
    cargo sqlx prepare --workspace

pg-start:
    pg_ctl start -l "$PGDATA/logfile" {{ pg_ctl_opts }}

pg-stop:
    pg_ctl stop

pg-status:
    pg_ctl status

pg-log:
    tail -f "$PGDATA/logfile"

db-reset:
    pg_ctl status || just pg-start
    dropdb --if-exists mydb
    createdb mydb
    just migrate
