# Run an individual test using the Lune CLI
run-test TEST_NAME:
	cargo run -- "tests/{{TEST_NAME}}"

# Run an individual file using the Lune CLI
run-file FILE_NAME:
	cargo run -- "{{FILE_NAME}}"

# Run tests for the Lune library
test:
	cargo test --lib -- --test-threads 1
