# By default, this makefile will run everything inside a Docker container.
# If you're a developer and want to run everything with your local `cargo`:
# - Creates a `.env` file in the same folder as this `makefile`
# - Add a line in `.env` with `LOCAL_DEV=true`
-include .env

ifeq ($(LOCAL_DEV),true)
	SH=sh -c
else
	SH=docker run --rm --user "$(id -u)":"$(id -g)" --volume "${PWD}":/tmp/workspace --workdir /tmp/workspace rust:latest sh -c
endif

fmt: format ## Check formatting of the code (alias for 'lint')

format: ## Check formatting of the code
	${SH} "rustup component add rustfmt && cargo fmt --all -- --check"

clippy: lint ## Check quality of the code (alias for 'lint')

lint: ## Check quality of the code
	${SH} "rustup component add clippy && cargo clippy --workspace --all-features --all-targets -- --warn clippy::cargo --allow clippy::multiple_crate_versions --deny warnings"

test: ## Launch all tests
	${SH} "cargo test --workspace --all-features --all-targets"

help: ## Print this help message
	@grep -E '^[a-zA-Z_-]+:.*## .*$$' $(CURDIR)/$(firstword $(MAKEFILE_LIST)) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt format clippy lint test help
.DEFAULT_GOAL := help
