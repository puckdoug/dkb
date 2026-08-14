default: build

test:
  cargo test

check:
  cargo clippy

build:
  cargo build

run:
  cargo run

pkg:
  ./scripts/bundle_macos.sh
