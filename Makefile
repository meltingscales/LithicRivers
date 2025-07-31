.PHONY: help install test test-coverage run build clean lint format setup-dev migrate-to-uv

# Default target
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development setup
setup-dev: ## Install development dependencies
	uv sync
	uv run pre-commit install

install: ## Install dependencies
	uv sync

# Testing
test: ## Run all tests
	uv run coverage run -m unittest discover lithicrivers

test-coverage: ## Run tests with coverage report
	uv run coverage run -m unittest discover lithicrivers
	uv run coverage report
	uv run coverage html

test-lcov: ## Generate LCOV coverage report
	uv run coverage run -m unittest discover lithicrivers
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
	uv run flake8 lithicrivers/
	uv run mypy lithicrivers/

format: ## Format code
	uv run black lithicrivers/
	uv run isort lithicrivers/

# Cleanup
clean: ## Clean build artifacts
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf .coverage
	rm -rf htmlcov/
	rm -rf coverage/
	find . -type d -name __pycache__ -delete
	find . -type f -name "*.pyc" -delete

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

 