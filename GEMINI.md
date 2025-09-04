# Gemini Context: coding-partner

This document provides context for the Gemini AI assistant to understand and effectively assist with this project.

## Project Overview

`coding-partner` is an early-stage project to build an AI-assisted coding assistant. It is written in Rust and structured as a workspace.

The main components are:
- A root workspace that defines shared dependencies and settings.
- A `ui` crate, which is currently placeholder code but is intended to contain the user interface.

The project utilizes the following core technologies:
- **Rust:** The primary programming language.
- **Tokio:** An asynchronous runtime for Rust, suggesting the application will handle concurrent operations.
- **env_logger:** For logging.

The project is in the initial phase of development. The `ui` crate contains only a template `add` function.

## Building and Running

Standard Rust cargo commands are used for managing the project.

- **Build the project:**
  ```bash
  cargo build
  ```

- **Run tests:**
  ```bash
  cargo test
  ```

- **Running the application:**
  - **TODO:** There is no runnable binary target defined in the project yet. The `ui` crate is a library.

## Development Conventions

- **Rust Edition:** 2024
- **Rust Version:** 1.85.0 or newer.
- **Workspace Structure:** The project is a Cargo workspace. New crates should be added to the `members` array in the root `Cargo.toml`.
- **Liniting:** The project uses `clippy` for linting, with rules configured in the root `Cargo.toml`.
