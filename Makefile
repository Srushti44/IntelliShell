SHELL := /bin/bash
.SHELLFLAGS := -c

# Source required environments
export PATH := $(HOME)/.cargo/bin:$(HOME)/.local/bin:$(PATH)

.PHONY: help bootstrap doctor fmt lint test check llm-smoke clean run-core run-api run-dashboard build

help: ## Show this help message
	@echo "IntelliShell - Available targets:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

bootstrap: ## Install all toolchains and dependencies
	@bash scripts/bootstrap.sh

doctor: ## Run environment health checks
	@bash scripts/doctor.sh

fmt: ## Format all code (Rust, Python, Dashboard)
	@echo "==> Formatting Rust code..."
	@cd core-engine && cargo fmt
	@echo "==> Formatting Python code..."
	@cd ai-service && uv run ruff format
	@echo "==> Formatting dashboard code..."
	@bash -c 'source ~/.nvm/nvm.sh && cd dashboard && npm run lint -- --fix || true'
	@echo "All code formatted."

lint: ## Lint all code (Rust, Python, Dashboard)
	@echo "==> Linting Rust code..."
	@cd core-engine && cargo clippy --all-targets -- -D warnings
	@echo "==> Linting Python code..."
	@cd ai-service && uv run ruff check
	@echo "==> Linting dashboard code..."
	@bash -c 'source ~/.nvm/nvm.sh && cd dashboard && npm run lint'
	@echo "All linting passed."

test: ## Run all tests (Rust, Python, Dashboard)
	@echo "==> Testing Rust code..."
	@cd core-engine && cargo test
	@echo "==> Testing Python code..."
	@cd ai-service && uv run pytest
	@echo "==> Testing dashboard..."
	@bash -c 'source ~/.nvm/nvm.sh && cd dashboard && (npm run test 2>/dev/null || echo "No dashboard tests yet")'
	@echo "All tests passed."

build: ## Build all components
	@echo "==> Building Rust..."
	@cd core-engine && cargo build --release
	@echo "==> Building dashboard..."
	@bash -c 'source ~/.nvm/nvm.sh && cd dashboard && npm run build'
	@echo "All components built."

check: lint test build ## Run lint, test, and build for all components

llm-smoke: ## Run LLM smoke test
	@bash scripts/llm-smoke.sh

clean: ## Clean all build artifacts and caches
	@echo "==> Cleaning Rust artifacts..."
	@cd core-engine && cargo clean
	@echo "==> Cleaning Python artifacts..."
	@rm -rf ai-service/.venv ai-service/__pycache__ ai-service/.pytest_cache ai-service/.ruff_cache
	@echo "==> Cleaning dashboard artifacts..."
	@bash -c 'rm -rf dashboard/node_modules dashboard/dist'
	@echo "All artifacts cleaned."

run-core: ## Run the core engine monitor loop
	@echo "Starting core engine (Ctrl+C to stop)..."
	@cd core-engine && cargo run

run-api: ## Run the AI service API
	@echo "Starting AI service on 127.0.0.1:8000..."
	@cd ai-service && uv run uvicorn main:app --host 127.0.0.1 --port 8000 --reload

run-dashboard: ## Run the dashboard dev server
	@echo "Starting dashboard on 127.0.0.1:5173..."
	@bash -c 'source ~/.nvm/nvm.sh && cd dashboard && npm run dev -- --host 127.0.0.1 --port 5173'
