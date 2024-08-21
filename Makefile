# Variables
PROJECT_NAME := wiki_meilisearch
DOCKER_COMPOSE := docker-compose.yml
PKG_DIR := pkg
SOURCE_FILES := engine.js definition.yml

# Default target (help)
.DEFAULT_GOAL := help

# List of targets with descriptions
help: ## Show this help
	@echo "$(PROJECT_NAME)"
	@echo "Usage: make <target>"
	@echo
	@awk 'BEGIN {FS = ":.*##"; printf "\033[1m\033[36m%-20s\033[0m %s\n", "Target", "Help"} \
	{ if (NF == 2) printf "\033[36m%-20s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST) | grep -v 'awk'

setup: ## Install Rust, wasm32 target, and wasm-pack
	@echo "Checking and installing necessary tools..."
	@if ! command -v rustup &> /dev/null; then \
		echo "rustup not found. Installing rustup..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
	fi
	@if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then \
		echo "wasm32-unknown-unknown target not found. Adding target..."; \
		rustup target add wasm32-unknown-unknown; \
	fi
	@if ! command -v wasm-pack &> /dev/null; then \
		echo "wasm-pack not found. Installing wasm-pack..."; \
		cargo install wasm-pack --force; \
	fi

build: clean ## Build the project
	@echo "Copying necessary files to pkg..."
	@mkdir -p $(PKG_DIR)
	@cp $(SOURCE_FILES) $(PKG_DIR)
	@echo "Building wasm-pack..."
	@wasm-pack build --target nodejs


dev: build ## Build the project and start development environment
	@echo "Starting Docker Compose..."
	@docker compose -f $(DOCKER_COMPOSE) up --remove-orphans

lint: ## Run linters on the Rust code
	@echo "Running linters..."
	@cargo fmt -- --check
	@cargo clippy -- -D warnings
	@npx -y prettier --write ./{*,.github/**/*}.{js,md,yaml,yml}

clean: ## Clean up the project
	@echo "Cleaning up..."
	@rm -rf $(PKG_DIR)
	@docker compose -f $(DOCKER_COMPOSE) down --remove-orphans --volumes

stop: ## Stop running Docker containers
	@echo "Stopping Docker Compose..."
	@docker compose -f $(DOCKER_COMPOSE) down

.PHONY: help setup build dev lint clean stop
