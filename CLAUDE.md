# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust Incubator learning course - a step-by-step progression from Rust basics to web backend development. The repository is structured as a learning path with exercises organized into sequential steps, each covering specific concepts, idioms, ecosystem tools, or backend technologies.

## Repository Structure

The repository uses a Cargo workspace architecture with the following hierarchy:

```
rust-incubator/
├── 0_basics/          # Step 0: Rust fundamentals (reading/exercises)
├── 1_concepts/        # Step 1: Core Rust concepts
│   ├── 1_1_default_clone_copy/
│   ├── 1_2_box_pin/
│   ├── 1_3_rc_cell/
│   └── ... (1_4 through 1_9)
├── 2_idioms/          # Step 2: Rust idioms and patterns
│   └── 2_1 through 2_6/
├── 3_ecosystem/       # Step 3: Common ecosystem crates
│   └── 3_1 through 3_11/
├── 4_backend/         # Step 4: Backend development
│   └── 4_1 through 4_3/
└── Cargo.toml         # Workspace root
```

### Workspace Organization

- Root `Cargo.toml` defines workspace members using glob patterns (`1_concepts/1_*`, etc.)
- Each major step (1_concepts, 2_idioms, etc.) has its own package named `step_N`
- Each sub-step is a separate package named `step_N_M` (e.g., `step_1_1`, `step_1_8`)
- All sub-steps contain a `README.md` with learning objectives, resources, and tasks

## Essential Commands

### Running Code

```bash
# Run a specific step from project root
cargo run -p step_1_8

# Run from within a step directory
cd 1_concepts/1_8_thread_safety && cargo run

# Run a specific example (if defined)
cargo run -p step_3_1 --example my_example
```

### Testing

```bash
# Test a specific step
cargo test -p step_1_1

# Test all workspace members
cargo test --workspace

# Run tests from within a step directory
cd 2_idioms/2_1_type_safety && cargo test
```

### Code Quality

```bash
# Format code (always use before committing)
cargo fmt

# Format check without modifying files
cargo fmt --check

# Run Clippy linter (always use before committing)
cargo clippy

# Run Clippy on all workspace members
cargo clippy --workspace

# Run Clippy with pedantic warnings
cargo clippy -- -W clippy::pedantic
```

### Building

```bash
# Build a specific step
cargo build -p step_2_3

# Build all workspace members
cargo build --workspace

# Build in release mode
cargo build --release -p step_4_2
```

## Development Workflow

### Working on Steps

Each step follows a PR-based workflow:

1. Complete the sub-step by reading its README.md and implementing the required tasks
2. Create a new branch for the step (e.g., `step-1-1-default-clone-copy`)
3. Implement the solution in the sub-step's `src/` directory
4. Run `cargo fmt` and `cargo clippy` to ensure code quality
5. Test the implementation with `cargo test -p step_N_M`
6. Create a PR with an appropriate name
7. After merging, check off the step in the root README.md schedule

### Edition and Toolchain

- Project uses Rust edition 2024 (see individual `Cargo.toml` files)
- Stable Rust channel specified in `rust-toolchain.toml`
- Use `rustup` to ensure correct toolchain is installed

### Code Style

The repository uses `.editorconfig` for consistent formatting:

- Rust files: 4-space indentation, UTF-8, LF line endings
- Max line length: 80 characters
- TOML files: 4-space indentation
- Always trim trailing whitespace (except in `.md` files)

## Learning Path Context

### Step 0: Become Familiar with Rust Basics (3 days)

Reading and exercises from Rust Book, Rust By Example, and Rustlings. No code in this repository - external resources only.

### Step 1: Concepts (2 days total)

Core Rust concepts including ownership, smart pointers, interior mutability, type conversions, dispatch mechanisms, sized types, thread safety, and phantom types. Final task: implement a thread-safe doubly linked list.

### Step 2: Idioms (2 days total)

Rust design patterns including type safety, `mem::replace`, trait bounds, generic parameters, exhaustive matching, and sealing traits.

### Step 3: Common Ecosystem (varies)

Working with popular crates for testing, macros, date/time, parsing, collections, serialization, crypto, logging, CLI args, multithreading, and async I/O.

### Step 4: Backend Ecosystem (3 days total)

Web backend development covering databases/ORMs, HTTP servers/clients, and API design.

## Important Notes

- When modifying code in a step, always read the step's README.md first to understand requirements
- Each step builds on previous steps - concepts accumulate
- The `0_basics` directory contains no code, only documentation and references
- Step numbering format: major steps (1, 2, 3, 4) contain sub-steps (1_1, 1_2, etc.)
- Package names use underscores: `step_1_8`, `step_3_11`
- All packages have `publish = false` - this is a learning repository, not publishable crates

## Syncing with Upstream

This repository may be a fork/copy of the template. To sync with upstream:

```bash
# One-time setup
git remote add upstream https://github.com/instrumentisto/rust-incubator.git
git fetch upstream main
git merge upstream/main --allow-unrelated-histories

# Regular updates
git fetch upstream main
git merge upstream/main
```
