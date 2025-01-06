# Setup development environment
setup:
    cp .env.sample .env

# Run socket mode example
run-socket-mode:
    cargo run --example socket_mode_example --features socket_mode

# Check code
check:
    cargo check --all-features

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --all-features
