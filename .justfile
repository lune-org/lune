# Run an individual test using the Lune CLI
run-test TEST_NAME:
	cargo run -- "tests/{{TEST_NAME}}"

# Run an individual file using the Lune CLI
run-file FILE_NAME:
	cargo run -- "{{FILE_NAME}}"

# Run tests for the Lune library
test:
	cargo test --package lune -- --test-threads 1

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
publish-gitbook: generate-gitbook
	npx push-dir --dir=gitbook --branch=gitbook
