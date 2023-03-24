# Run tests for the Lune library
test:
	cargo test --package lune -- --test-threads 1

# Run tests for the Lune CLI
test-cli:
	cargo test --package lune-cli

# Generate gitbook directory
generate-gitbook:
	rm -rf ./gitbook

	mkdir gitbook
	mkdir gitbook/docs

	cp -R docs gitbook
	cp README.md gitbook/docs/README.md
	cp .gitbook.yaml gitbook/.gitbook.yaml

	rm -rf gitbook/docs/typedefs

	cargo run -- --generate-gitbook-dir

# Publish gitbook directory to gitbook branch
publish-gitbook:
	npx push-dir --dir=gitbook --branch=gitbook
