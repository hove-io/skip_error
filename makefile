# By default, this makefile will run everything inside a Docker container.
# If you're a developer and want to run everything with your local `cargo`:
# - Creates a `.env` file in the same folder as this `makefile`
# - Add a line in `.env` with `LOCAL_DEV=true`
-include .env

define run_cmd
	if [ "$(LOCAL_DEV)" = true ] ; \
	then \
		$(1); \
	else \
		docker run --rm --user "$(id -u)":"$(id -g)" --volume "${PWD}":/tmp/workspace --workdir /tmp/workspace kisiodigital/rust-ci:latest sh -c "$(1)"; \
	fi
endef

CARGO_FMT=cargo fmt --all -- --check
fmt: format ## Check formatting of the code (alias for 'format')
format: ## Check formatting of the code
	@echo "--> Running '$(CARGO_FMT)'"
	@$(call run_cmd, rustup component add rustfmt && $(CARGO_FMT))

CARGO_CLIPPY=cargo clippy --workspace --all-features --all-targets -- --warn clippy::cargo --allow clippy::multiple_crate_versions --deny warnings
clippy: lint ## Check quality of the code (alias for 'lint')
lint: ## Check quality of the code
	@echo "--> Running '$(CARGO_CLIPPY)'"
	@$(call run_cmd, rustup component add clippy && $(CARGO_CLIPPY))

CARGO_TEST=cargo test --workspace --all-features --all-targets
test: ## Launch all tests
	@echo "--> Running '$(CARGO_TEST)'"
	@$(call run_cmd, $(CARGO_TEST))

help: ## Print this help message
	@grep -E '^[a-zA-Z_-]+:.*## .*$$' $(CURDIR)/$(firstword $(MAKEFILE_LIST)) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt format clippy lint test help
.DEFAULT_GOAL := help
