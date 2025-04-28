EXT := if os() == "windows" { ".exe" } else { "" }
CWD := invocation_directory()
BIN_NAME := "lune"

# Default hidden recipe for listing other recipes + cwd
[no-cd]
[no-exit-message]
[private]
default:
	#!/usr/bin/env bash
	set -euo pipefail
	printf "Current directory:\n    {{CWD}}\n"
	just --list

# Builds the Lune CLI binary
[no-exit-message]
build *ARGS:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo build --bin {{BIN_NAME}} {{ARGS}}

# Run an individual file using the Lune CLI
[no-exit-message]
run FILE_PATH:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo run --bin {{BIN_NAME}} -- run "{{FILE_PATH}}"

# Run tests for the Lune library
[no-exit-message]
test *ARGS:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo test --lib -- {{ARGS}}

# Run tests for the Lune binary
[no-exit-message]
test-bin *ARGS:
	#!/usr/bin/env bash
	set -euo pipefail
	cargo test --bin {{BIN_NAME}} -- {{ARGS}}

# Apply formatting for all Rust & Luau files
[no-exit-message]
fmt:
	#!/usr/bin/env bash
	set -euo pipefail
	stylua .lune crates scripts tests \
		--glob "tests/**/*.luau" \
		--glob "!tests/roblox/rbx-test-files/**"
	cargo fmt

# Check formatting for all Rust & Luau files
[no-exit-message]
fmt-check:
	#!/usr/bin/env bash
	set -euo pipefail
	stylua .lune crates scripts tests \
		--glob "tests/**/*.luau" \
		--glob "!tests/roblox/rbx-test-files/**"
	cargo fmt --check

# Analyze and lint Luau files using luau-lsp
[no-exit-message]
analyze:
	#!/usr/bin/env bash
	set -euo pipefail
	lune run scripts/analyze_copy_typedefs
	luau-lsp analyze \
		--settings=".vscode/settings.json" \
		--ignore="tests/roblox/rbx-test-files/**" \
		.lune crates scripts tests

# Zips up the built binary into a single zip file
[no-exit-message]
zip-release TARGET_TRIPLE:
	#!/usr/bin/env bash
	set -euo pipefail
	rm -rf staging
	rm -rf release.zip
	mkdir -p staging
	cp "target/{{TARGET_TRIPLE}}/release/{{BIN_NAME}}{{EXT}}" staging/
	cd staging
	if [ "{{os_family()}}" = "windows" ]; then
		7z a ../release.zip *
	else
		chmod +x {{BIN_NAME}}
		zip ../release.zip *
	fi
	cd "{{CWD}}"
	rm -rf staging

# Used in GitHub workflow to move per-matrix release zips
[no-exit-message]
[private]
unpack-releases RELEASES_DIR:
	#!/usr/bin/env bash
	set -euo pipefail
	#
	if [ ! -d "{{RELEASES_DIR}}" ]; then
		echo "Releases directory is missing"
		exit 1
	fi
	#
	cd "{{RELEASES_DIR}}"
	echo ""
	echo "Releases dir:"
	ls -lhrt
	echo ""
	echo "Searching for zipped releases..."
	#
	for DIR in * ; do
		if [ -d "$DIR" ]; then
			cd "$DIR"
			for FILE in * ; do
				if [ ! -d "$FILE" ]; then
					if [ "$FILE" = "release.zip" ]; then
						echo "Found zipped release '$DIR'"
						mv "$FILE" "../$DIR.zip"
						rm -rf "../$DIR/"
					fi
				fi
			done
			cd ..
		fi
	done
	#
	echo ""
	echo "Releases dir:"
	ls -lhrt
