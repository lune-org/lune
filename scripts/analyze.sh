#!/usr/bin/env bash

set -euo pipefail

lune run scripts/analyze_copy_typedefs

luau-lsp analyze \
	--platform=standard \
	--settings=".vscode/settings.json" \
	--ignore="tests/roblox/rbx-test-files/**" \
	.lune crates scripts tests
