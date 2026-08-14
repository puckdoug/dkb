default: build

test:
  cargo test

check:
  cargo clippy --all-targets

pedantic:
  cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic

build:
  cargo build

run:
  cargo run

pkg:
  ./scripts/bundle_macos.sh
