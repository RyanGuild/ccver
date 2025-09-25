# Makefile for ccver development

.PHONY: help setup fmt clippy test build clean pre-commit install-hooks

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

setup: ## Set up development environment with pre-commit hooks
	./setup-precommit.sh

fmt: ## Format code with cargo fmt
	cargo fmt --all

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

test: ## Run all tests
	cargo test --all-features

build: ## Build the project
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

clean: ## Clean build artifacts
	cargo clean

pre-commit: ## Run all pre-commit hooks manually
	pre-commit run --all-files

install-hooks: ## Install pre-commit hooks (requires pre-commit to be installed)
	pre-commit install

check: fmt clippy test ## Run format, clippy, and tests (same as pre-commit)

update-version: ## Update version in Cargo.toml using ccver
	@./scripts/update-version.sh
