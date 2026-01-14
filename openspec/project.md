# Project Context

## Purpose

**local-db** is a cross-platform database management tool built with Tauri 2 and React 19. It simplifies the installation, configuration, and management of various local databases including MySQL, PostgreSQL, MongoDB, Redis, Qdrant, SurrealDB, Neo4j, and SeekDB. The goal is to provide a unified dashboard for developers to manage their local development data infrastructure easily.

## Tech Stack

- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS 4
- **Backend**: Rust, Tauri 2
- **State Management**: AppState in Rust (source of truth) + React hooks (local state)
- **Icons**: React Icons (specifically `react-icons/fi` for Feather icons)
- **Styling**: Tailwind CSS 4 with dark mode support
- **Monitoring**: Recharts for resource visualization

## Project Conventions

### Code Style

- **TypeScript**: Strict mode enabled. Use PascalCase for components and interfaces, camelCase for functions and variables.
- **Rust**: Standard Rust conventions. Use `anyhow::Result` for error handling and `serde` for serialization.
- **Import Ordering**:
  1. React and hooks
  2. Third-party libraries
  3. Local components and types
- **Formatting**: Managed by Prettier (JS/TS) and `cargo fmt` (Rust).

### Architecture Patterns

- **Frontend-Backend Bridge**: Tauri Commands are used for communication. Commands are defined in Rust (`src-tauri/src/command/`) and wrapped in TypeScript (`src/command/`).
- **State Management**: The backend `AppState` maintains the list of databases and settings, persisting to `~/.local-db/state.json`.
- **Database Lifecycle**: Managed by `DatabaseManager` in Rust, handling binary downloads, process management (PID tracking), and configuration.
- **Single Instance Policy**: Each database type (e.g., MySQL) can only have ONE instance installed at a time.

### Testing Strategy

- Currently, no formal test framework is configured.
- **Verification**: Use `pnpm build` for frontend and `cd src-tauri && cargo check` for backend verification.

### Git Workflow

- Standard branching (main/feature) and clear commit messages are expected.

## Domain Context

- **Storage Path**: Default storage is in `~/.local-db/`.
- **Database Types**: Supports relational (MySQL, PostgreSQL), NoSQL (MongoDB, Redis), Vector (Qdrant), and Multi-model (SurrealDB, Neo4j) databases.
- **Auto-Start**: Databases can be configured to start automatically when the application launches.

## Important Constraints

- **Platform Priority**: macOS is first-class, with Windows support in development.
- **Installation**: Relies on system-specific package managers (Homebrew on macOS) or direct binary management on Windows.
- **Permissions**: Requires permissions to create directories and manage processes in the user's home directory.

## External Dependencies

- **Tauri Plugins**: fs, log, os, process, autostart, updater.
- **Binaries**: Downloads official database binaries from various sources (GitHub releases, official CDNs).
