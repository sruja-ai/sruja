## Compatibility shim.
##
## Sruja uses `justfile` as the primary task runner.
## This Makefile exists only so `make <target>` continues to work.
##
## Usage:
##   just --list
##   just check
##   just build-extension

.DEFAULT_GOAL := help

.PHONY: help
help:
	@command -v just >/dev/null 2>&1 || (echo "❌ 'just' is required. Install: https://github.com/casey/just"; exit 1)
	@just --list

.PHONY: %
%:
	@command -v just >/dev/null 2>&1 || (echo "❌ 'just' is required. Install: https://github.com/casey/just"; exit 1)
	@just $@
