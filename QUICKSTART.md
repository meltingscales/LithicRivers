# LithicRivers Quick Start

Back to the [README](README.md). Working with AI agents? See [README.USING-AI.md](README.USING-AI.md).

## Prerequisites
- Rust (latest stable)
- `just` command runner (`cargo install just`)

## Getting Started
1. Clone the repository
2. Install dependencies: `just install`
3. Run the game: `just client-linux` (or `just client-windows`)

## Common Commands
- `just build-linux` / `just build-windows` - Build the project
- `cargo test --workspace` - Run tests (`just test-linux` also builds first)
- `just code-clippy` - Run clippy; warnings are errors
- `just code-fmt` - Format the code
- `just client-blind <script>` - Run the game headless from a script (no terminal needed)

## Development Workflow
1. Create a feature branch
2. Make your changes
3. Run tests: `cargo test --workspace`
4. Lint and format: `just code-clippy`, `just code-fmt`
5. Submit a pull request (PR checks run tests and clippy on Linux, Windows and macOS)

For more details, see the [documentation in the docs/ folder](docs/).
