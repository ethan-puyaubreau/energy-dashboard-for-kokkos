#!/bin/bash

find . -type f ! -name 'readme.md' ! -name 'clean.sh' ! -name 'generic_script.sh' -exec rm -f {} \;
find . -type d ! -name 'variorum' -exec rm -rf {} \;