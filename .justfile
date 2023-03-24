# Run tests for the Lune library
test:
	cargo test --package lune -- --test-threads 1

# Run tests for the Lune CLI
test-cli:
	cargo test --package lune-cli

# Publish gitbook directory to gitbook branch
publish-gitbook:
	npx push-dir --dir=docs --branch=gitbook
