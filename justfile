# Build by default
default: build

# Run cargo test
test *ARGS:
  cargo test {{ ARGS }}

# Basic clippy lints for this complete app
check:
  cargo clippy --all-targets

# Pedantic clippy lints for this complete app
pedantic:
  cargo clippy --workspace --all-targets -- -D warnings -D clippy::pedantic

# Build the app
build *ARGS:
  cargo build {{ ARGS }}

# Run the GUI app
run *ARGS:
  cargo run -p dkb {{ ARGS }}

# Build the Mac OS App bundle
pkg:
  ./scripts/bundle_macos.sh

# Build the mac OS App bundle
package: pkg
