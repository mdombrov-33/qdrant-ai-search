
# Python format/lint
format-python:
	poetry run --directory backend black .

# Lint Python code inside backend/
lint-python:
	poetry run --directory backend ruff check .

# Rust format/lint
format-rust:
	cargo fmt --manifest-path rust_accelerator/Cargo.toml

lint-rust:
	cargo clippy --manifest-path rust_accelerator/Cargo.toml --all-targets --all-features -- -D warnings