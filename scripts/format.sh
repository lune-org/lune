#!/usr/bin/env bash

set -euo pipefail

stylua .lune crates scripts tests \
	--glob "tests/**/*.luau" \
	--glob "!tests/roblox/rbx-test-files/**"

cargo fmt
