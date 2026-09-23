#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

find . -mindepth 1 -type f ! -name 'readme.md' ! -name 'clean.sh' ! -name 'generic_script.sh' -exec rm -f {} +
find . -mindepth 1 -type d ! -name 'variorum' -exec rm -rf {} +