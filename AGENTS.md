# AGENTS.md - Rust Incubator Development Guide

## Build/Lint/Test Commands
- **Build specific package**: `cargo build -p step_N_M`
- **Build all**: `cargo build --workspace`
- **Run specific package**: `cargo run -p step_N_M`
- **Test specific package**: `cargo test -p step_N_M`
- **Test all**: `cargo test --workspace`
- **Run single test**: `cargo test -p step_N_M test_function_name`
- **Format code**: `cargo fmt`
- **Check formatting**: `cargo fmt --check`
- **Lint code**: `cargo clippy`
- **Lint all with pedantic**: `cargo clippy --workspace -- -W clippy::pedantic`

## Code Style Guidelines
- **Edition**: Rust 2024
- **Toolchain**: Stable
- **Formatting**: 4 spaces, 80 char lines, UTF-8, LF endings
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Imports**: Group std, then external crates, then local modules
- **Error handling**: Use Result/Option, avoid unwrap/expect in production code
- **Types**: Prefer explicit types, use type aliases for complex types
- **Documentation**: Use /// for public APIs, no unnecessary comments
- **Testing**: Unit tests in same file, integration tests in tests/ directory

## Project Structure
- Workspace with packages named `step_N_M` (e.g., step_1_1, step_3_6)
- Each step has src/main.rs or src/lib.rs with Cargo.toml
- README.md in each step contains learning objectives and tasks</content>
<parameter name="filePath">AGENTS.md