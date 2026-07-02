#!/usr/bin/env bash

# Since we are using cargo workspaces, reading the actual version
# of the CLI is slightly more complicated - which is why this exists

set -euo pipefail

CLI_MANIFEST=$(cargo read-manifest --manifest-path crates/lune/Cargo.toml)
CLI_VERSION=$(echo $CLI_MANIFEST | jq -r .version)

echo $CLI_VERSION
