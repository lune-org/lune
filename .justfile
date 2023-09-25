# Run an individual test using the Lune CLI
run-test TEST_NAME:
	cargo run -- "tests/{{TEST_NAME}}"

# Run an individual file using the Lune CLI
run-file FILE_NAME:
	cargo run -- "{{FILE_NAME}}"

# Run tests for the Lune library
test:
	cargo test --lib

# Check formatting for all Rust & Luau files
fmt-check:
	#!/usr/bin/env bash
	set -euo pipefail
	stylua scripts --check
	stylua types --check
	stylua tests --check \
		--glob "tests/**/*.luau" \
		--glob "!tests/roblox/rbx-test-files/**"
	cargo fmt --check
