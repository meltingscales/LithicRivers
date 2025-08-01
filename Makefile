.PHONY: help install test test-coverage run build clean lint format setup-dev migrate-to-uv

# Default target
help: ## Show this help message
	@echo "LithicRivers - Available Makefile Targets"
	@echo "=========================================="
	@echo ""
	@echo "📦 SETUP & INSTALLATION"
	@echo "------------------------"
	@echo "  setup-dev            Install development dependencies"
	@echo "  install              Install dependencies"
	@echo "  migrate-to-uv        Migrate from Poetry to uv"
	@echo ""
	@echo "🧪 TESTING"
	@echo "-----------"
	@echo "  test                 Run all tests"
	@echo "  test-tui             Run TUI tests"
	@echo "  test-tui-quick       Run quick TUI tests (simplified)"
	@echo "  test-tui-headless    Run headless TUI tests"
	@echo "  test-tui-visual      Run visual regression tests"
	@echo "  test-coverage        Run tests with coverage report"
	@echo "  test-lcov            Generate LCOV coverage report"
	@echo ""
	@echo "🎮 GAME OPERATIONS"
	@echo "------------------"
	@echo "  run                  Run the game"
	@echo "  docker-run           Run game in Docker"
	@echo ""
	@echo "🔨 BUILDING & DEPLOYMENT"
	@echo "-------------------------"
	@echo "  build                Build the executable"
	@echo "  build-docker         Build Docker image"
	@echo ""
	@echo "🐳 DOCKER OPERATIONS"
	@echo "-------------------"
	@echo "  docker-push          Push Docker image"
	@echo ""
	@echo "🔍 CODE QUALITY"
	@echo "--------------"
	@echo "  lint                 Run linting"
	@echo "  format               Format code"
	@echo ""
	@echo "🧹 CLEANUP"
	@echo "----------"
	@echo "  clean                Clean build artifacts"
	@echo ""
	@echo "🔄 WORKFLOWS"
	@echo "------------"
	@echo "  dev                  Install, test, and run (development workflow)"
	@echo "  release              Clean, test with coverage, and build (release workflow)"
	@echo ""
	@echo "💡 Usage Examples:"
	@echo "  make dev          # Full development workflow"
	@echo "  make test         # Run all tests"
	@echo "  make run          # Run the game"
	@echo "  make build        # Build executable"
	@echo "  make clean        # Clean build artifacts"

# Development setup
setup-dev: ## Install development dependencies
	uv sync --extra dev
	uv run pre-commit install

install: ## Install dependencies
	uv sync --extra dev

# Testing
test: ## Run all tests
	@if [ -z "$$TERM" ]; then \
		echo "⚠️  TERM environment variable not set - some TUI tests will be skipped"; \
	fi
	TESTING=1 uv run coverage run -m unittest discover lithicrivers

# Helper function to check if terminal is available
check_term = @if [ -z "$$TERM" ]; then \
	echo "⚠️  TERM environment variable not set - skipping terminal-dependent tests"; \
	echo "   Available tests: simplified TUI tests (mock-based)"; \
	echo "   Skipped tests: headless TUI tests, visual regression tests"; \
else \
	echo "✅ TERM environment variable detected - running all TUI tests"; \
fi

test-tui: ## Run TUI tests
	@echo "Running TUI tests..."
	@echo "1. Running simplified TUI tests..."
	uv run python -m unittest lithicrivers.test.test_tui_simple
	@echo "2. Running advanced mock-based tests..."
	uv run python -m unittest lithicrivers.test.test_tui_advanced
	$(check_term)
	@if [ -n "$$TERM" ]; then \
		echo "3. Running headless TUI tests..."; \
		uv run python -m unittest lithicrivers.test.test_tui_headless; \
	else \
		echo "3. Skipping headless TUI tests (no TERM)"; \
	fi
	@if [ -n "$$TERM" ]; then \
		echo "4. Running visual regression tests..."; \
		uv run python -m unittest lithicrivers.test.test_tui_visual; \
	else \
		echo "4. Skipping visual regression tests (no TERM)"; \
	fi
	@echo "TUI tests completed!"

test-tui-quick: ## Run quick TUI tests (simplified)
	@echo "Running simplified TUI tests..."
	uv run python -m unittest lithicrivers.test.test_tui_simple

test-tui-headless: ## Run headless TUI tests
	@if [ -z "$$TERM" ]; then \
		echo "⚠️  TERM environment variable not set - skipping headless TUI tests"; \
		echo "   These tests require a terminal environment"; \
	else \
		echo "Running headless TUI tests..."; \
		uv run python -m unittest lithicrivers.test.test_tui_headless; \
	fi

test-tui-visual: ## Run visual regression tests
	@if [ -z "$$TERM" ]; then \
		echo "⚠️  TERM environment variable not set - skipping visual regression tests"; \
		echo "   These tests require a terminal environment"; \
	else \
		echo "Running visual regression tests..."; \
		uv run python -m unittest lithicrivers.test.test_tui_visual; \
	fi



test-coverage: ## Run tests with coverage report
	TESTING=1 uv run coverage run -m unittest discover lithicrivers
	uv run coverage report
	uv run coverage html

test-lcov: ## Generate LCOV coverage report
	TESTING=1 uv run coverage run -m unittest discover lithicrivers
	uv run coverage lcov -o coverage/lcov.info

# Running the game
run: ## Run the game
	uv run python -m lithicrivers

# Building
build: ## Build the executable
	uv run pyinstaller lithicrivers.spec

build-docker: ## Build Docker image
	docker build ./ --tag henryfbp/lithicrivers:latest

# Code quality
lint: ## Run linting
	uv run ruff check lithicrivers/
	uv run mypy lithicrivers/

lint-stats: ## Run linting with statistics
	uv run ruff check lithicrivers/ --statistics

format: ## Format code
	uv run ruff format lithicrivers/
	uv run ruff check --fix lithicrivers/

# Cleanup
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf .coverage
	rm -rf htmlcov/
	rm -rf coverage/
	@echo "Removing log files..."
	rm -f *.log
	@echo "Removing Python cache files..."
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	find . -type f -name "*.pyo" -delete 2>/dev/null || true
	find . -type f -name "*.pyd" -delete 2>/dev/null || true
	@echo "Removing temporary files..."
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	rm -rf .ruff_cache/
	@echo "Clean complete!"

# Migration from Poetry to uv
migrate-to-uv: ## Migrate from Poetry to uv
	@echo "Installing uv..."
	pip install uv
	@echo "Installing dependencies with uv..."
	uv sync
	@echo "Migration complete! Use 'make run' to run the game"

# Docker operations
docker-run: ## Run game in Docker
	docker run --interactive --tty henryfbp/lithicrivers:latest

docker-push: ## Push Docker image
	docker push henryfbp/lithicrivers

# Development workflow
dev: install test run ## Install, test, and run (development workflow)

# Release workflow
release: clean test-coverage build ## Clean, test with coverage, and build (release workflow)

 