# Prefer podman if available, fall back to docker
container_runtime := `command -v podman >/dev/null 2>&1 && echo podman || echo docker`
compose := container_runtime + " compose"

# Run fmt check, clippy, build, and unit tests
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace
    cargo test --workspace

# Auto-fix formatting
fmt:
    cargo fmt --all

# Run the mock server locally
run port="3000":
    cargo run --bin openresponses-server -- --port {{port}}

# Build container image
build:
    {{container_runtime}} build -t openresponses-server .

# Run compliance tests in containers (requires ../openresponses checkout)
compliance:
    {{compose}} -f docker-compose.test.yml up \
        --build --abort-on-container-exit \
        --exit-code-from compliance
    {{compose}} -f docker-compose.test.yml down

# Full CI pipeline: check + compliance
ci: check compliance

# Clean build artifacts
clean:
    cargo clean
