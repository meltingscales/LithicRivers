.PHONY: help install test build run clean lint format log-monitor

# NixOS uv detection and path fixing
# Check if we're on NixOS and use system uv if available
ifeq ($(shell test -f /etc/os-release && grep -q "ID=nixos" /etc/os-release && echo "nixos"),nixos)
    # On NixOS, prefer system uv over user uv
    UV_CMD := $(shell if [ -f /run/current-system/sw/bin/uv ]; then echo "/run/current-system/sw/bin/uv"; else echo "uv"; fi)
else
    UV_CMD := uv
endif

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
	@echo "  run-debug            Run the game with remote debugging"
	@echo "  debug-attach         Show PyCharm debugging instructions"
	@echo "  log-monitor          Monitor game logs in real-time"
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
	@echo ""
	@echo "🐳 DOCKER (DISABLED)"
	@echo "-------------------"
	@echo "  docker-build         [DISABLED] Build Docker image"
	@echo "  docker-run           [DISABLED] Run Docker container"
	@echo "  docker-push          [DISABLED] Push to Docker Hub"

# Setup
install: ## Install dependencies
	$(UV_CMD) sync --extra dev

# Testing (consolidated)
test: ## Run all tests with smart detection
	@echo "🧪 Running comprehensive test suite..."
	@echo "📊 Running unit tests with coverage..."
	TESTING=1 $(UV_CMD) run coverage run -m unittest discover lithicrivers
	@echo "🎮 Running TUI tests (with TERM detection)..."
	@if [ -z "$$TERM" ]; then \
		echo "⚠️  TERM not set - running mock-based tests only"; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_simple; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_advanced; \
	else \
		echo "✅ TERM detected - running all TUI tests"; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_simple; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_advanced; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_headless; \
		$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_visual; \
	fi
	@echo "✅ All tests completed!"

test-quick: ## Run quick tests only
	@echo "⚡ Running quick tests..."
	TESTING=1 $(UV_CMD) run coverage run -m unittest discover lithicrivers
	$(UV_CMD) run python -m unittest lithicrivers.test.test_tui_simple
	@echo "✅ Quick tests completed!"

test-lcov: ## Generate LCOV coverage report
	@echo "📊 Generating LCOV coverage report..."
	TESTING=1 $(UV_CMD) run coverage run -m unittest discover lithicrivers
	$(UV_CMD) run coverage lcov -o coverage/lcov.info
	@echo "✅ LCOV report generated!"

# Game
run: ## Run the game
	$(UV_CMD) run python -m lithicrivers

run-debug: ## Run the game with remote debugging enabled
	@echo "🐛 Starting game with remote debugging..."
	@echo "📝 In PyCharm: Run -> Attach to Process -> Select this Python process"
	@echo "🔗 Or use: Run -> Edit Configurations -> + -> Python Debug Server"
	@echo "🌐 Debug server will be available on localhost:5678"
	PYTHONPATH=. $(UV_CMD) run python -m lithicrivers --debug

debug-attach: ## Show instructions for attaching to running process
	@echo "🔗 PyCharm Remote Debugging Instructions"
	@echo "========================================"
	@echo ""
	@echo "1. Start the game in another terminal:"
	@echo "   make run"
	@echo ""
	@echo "2. In PyCharm:"
	@echo "   - Go to Run -> Edit Configurations"
	@echo "   - Click + -> Python Debug Server"
	@echo "   - Set host: localhost, port: 5678"
	@echo "   - Click OK"
	@echo ""
	@echo "3. Start the debug server:"
	@echo "   - Run -> Start Debug Server"
	@echo ""
	@echo "4. In your running game terminal, add this line where you want to break:"
	@echo "   import pydevd; pydevd.settrace(suspend=False, trace_only_current_thread=True)"
	@echo ""
	@echo "5. Or use the debug target instead:"
	@echo "   make run-debug"
	@echo ""
	@echo "📚 More info: https://www.jetbrains.com/help/pycharm/remote-debugging-with-product.html"

log-monitor: ## Monitor game logs in real-time
	@echo "📋 Monitoring LithicRivers.log in real-time..."
	@echo "🔄 Press Ctrl+C to stop monitoring"
	@echo ""
	@if [ -f "LithicRivers.log" ]; then \
		echo "📄 Found existing log file, starting monitor..."; \
		tail -f LithicRivers.log; \
	else \
		echo "📄 No log file found yet. Starting monitor (will show logs when game runs)..."; \
		touch LithicRivers.log; \
		tail -f LithicRivers.log; \
	fi

# Building
build: ## Build executable
	$(UV_CMD) run pyinstaller lithicrivers.spec

# Code quality
lint: ## Run linting
	$(UV_CMD) run ruff check lithicrivers/
	$(UV_CMD) run mypy lithicrivers/

format: ## Format code
	$(UV_CMD) run ruff format lithicrivers/
	$(UV_CMD) run ruff check --fix lithicrivers/

# Cleanup
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	rm -rf build/ dist/ *.egg-info/ .coverage htmlcov/ coverage/
	rm -f *.log
	rm coverage.lcov
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	rm -rf .pytest_cache/ .mypy_cache/ .ruff_cache/
	@echo "✅ Clean complete!" 

# Docker targets (DISABLED - Steam publishing)
docker-build: ## Build Docker image
	@echo "🚫 Docker targets are disabled!"
	@echo "📦 LithicRivers is now being published on Steam!"
	@echo "🎮 Please use Steam to download and play the game."
	@echo "🔗 Visit: https://store.steampowered.com/app/[YOUR_APP_ID]"
	@exit 1

docker-run: ## Run Docker container
	@echo "🚫 Docker targets are disabled!"
	@echo "📦 LithicRivers is now being published on Steam!"
	@echo "🎮 Please use Steam to download and play the game."
	@echo "🔗 Visit: https://store.steampowered.com/app/[YOUR_APP_ID]"
	@exit 1

docker-push: ## Push to Docker Hub
	@echo "🚫 Docker targets are disabled!"
	@echo "📦 LithicRivers is now being published on Steam!"
	@echo "🎮 Please use Steam to download and play the game."
	@echo "🔗 Visit: https://store.steampowered.com/app/[YOUR_APP_ID]"
	@exit 1 