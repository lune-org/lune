# Run an individual test using the Lune CLI
run-test TEST_NAME:
	cargo run --features reqwest/rustls-tls -- "tests/{{TEST_NAME}}"

# Run an individual file using the Lune CLI
run-file FILE_NAME:
	cargo run --features reqwest/rustls-tls -- "{{FILE_NAME}}"

# Run tests for the Lune library
test:
	cargo test --features reqwest/rustls-tls --package lune -- --test-threads 1

# Run tests for the Lune CLI
test-cli:
	cargo test --features reqwest/rustls-tls --package lune-cli

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

list:
	#!/usr/bin/env bash
	for DIR in */ ; do
		cd "$DIR"
		for FILE in * ; do
			if [ ! -d "$FILE" ]; then
				echo "$DIR$FILE"
			fi
		done
		cd ..
	done
