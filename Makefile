.PHONY: help install test build run clean lint format

# Default target
help: ## Show this help message
	@echo "LithicRivers - Simplified Makefile"
	@echo "=================================="
	@echo ""
	@echo "📦 SETUP"
	@echo "--------"
	@echo "  install              Install dependencies"
	@echo ""
	@echo "🧪 TESTING"
	@echo "---------"
	@echo "  test                 Run all tests (with TERM detection)"
	@echo "  test-quick           Run quick tests only"
	@echo ""
	@echo "🎮 GAME"
	@echo "------"
	@echo "  run                  Run the game"
	@echo ""
	@echo "🔨 BUILDING"
	@echo "----------"
	@echo "  build                Build executable"
	@echo ""
	@echo "🔍 CODE QUALITY"
	@echo "---------------"
	@echo "  lint                 Run linting"
	@echo "  format               Format code"
	@echo ""
	@echo "🧹 CLEANUP"
	@echo "---------"
	@echo "  clean                Clean build artifacts"

# Setup
install: ## Install dependencies
	uv sync --extra dev

# Testing (consolidated)
test: ## Run all tests with smart detection
	@echo "🧪 Running comprehensive test suite..."
	@echo "📊 Running unit tests with coverage..."
	TESTING=1 uv run coverage run -m unittest discover lithicrivers
	@echo "🎮 Running TUI tests (with TERM detection)..."
	@if [ -z "$$TERM" ]; then \
		echo "⚠️  TERM not set - running mock-based tests only"; \
		uv run python -m unittest lithicrivers.test.test_tui_simple; \
		uv run python -m unittest lithicrivers.test.test_tui_advanced; \
	else \
		echo "✅ TERM detected - running all TUI tests"; \
		uv run python -m unittest lithicrivers.test.test_tui_simple; \
		uv run python -m unittest lithicrivers.test.test_tui_advanced; \
		uv run python -m unittest lithicrivers.test.test_tui_headless; \
		uv run python -m unittest lithicrivers.test.test_tui_visual; \
	fi
	@echo "✅ All tests completed!"

test-quick: ## Run quick tests only
	@echo "⚡ Running quick tests..."
	TESTING=1 uv run coverage run -m unittest discover lithicrivers
	uv run python -m unittest lithicrivers.test.test_tui_simple
	@echo "✅ Quick tests completed!"

test-lcov: ## Generate LCOV coverage report
	@echo "📊 Generating LCOV coverage report..."
	TESTING=1 uv run coverage run -m unittest discover lithicrivers
	uv run coverage lcov -o coverage/lcov.info
	@echo "✅ LCOV report generated!"

# Game
run: ## Run the game
	uv run python -m lithicrivers

# Building
build: ## Build executable
	uv run pyinstaller lithicrivers.spec

# Code quality
lint: ## Run linting
	uv run ruff check lithicrivers/
	uv run mypy lithicrivers/

format: ## Format code
	uv run ruff format lithicrivers/
	uv run ruff check --fix lithicrivers/

# Cleanup
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	rm -rf build/ dist/ *.egg-info/ .coverage htmlcov/ coverage/
	rm -f *.log
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	rm -rf .pytest_cache/ .mypy_cache/ .ruff_cache/
	@echo "✅ Clean complete!" 