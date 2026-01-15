<!-- OPENSPEC:START -->
# OpenSpec 使用说明

这些说明适用于在此项目中工作的AI助手。

## 语言偏好设置

**默认使用中文**：除非明确说明使用英文，否则所有输出都应使用中文，包括：
- 文档内容
- 代码注释
- 提交信息
- 规范说明

## 工作流程

当请求满足以下条件时，始终打开`@/openspec/AGENTS.md`：
- 提及规划或提案（如提案、规范、变更、计划等词语）
- 引入新功能、重大变更、架构变更或大型性能/安全工作时
- 听起来不明确，需要在编码前了解权威规范时

使用`@/openspec/AGENTS.md`了解：
- 如何创建和应用变更提案
- 规范格式和约定
- 项目结构和指南

保持此托管块，以便'openspec-cn update'可以刷新说明。

<!-- OPENSPEC:END -->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**local-db** is a cross-platform database management tool built with Tauri 2, React 19, and Rust. It simplifies local database installation, configuration, and management for multiple database types (MySQL, PostgreSQL, MongoDB, Redis, Qdrant, Neo4j, SeekDB, SurrealDB).

**Key Constraint**: Each database type can only have ONE instance installed at a time (enforced in the backend).

## Development Commands

### Frontend Development

```bash
pnpm dev              # Start Vite dev server (port 1420)
pnpm build            # TypeScript compile + Vite production build
pnpm preview          # Preview production build
```

### Code Formatting

```bash
pnpm fmt              # Format both JS/TS and Rust files
pnpm fmt:js           # Format JS/TS with Prettier
pnpm fmt:rs           # Format Rust with cargo fmt
```

### CHANGELOG Generation

```bash
pnpm changelog        # Generate CHANGELOG.md from Git history
pnpm changelog:preview # Preview CHANGELOG without writing to file
```

**Usage Notes**:
- Automatically extracts changes from Git commit history
- Categorizes commits by type (feat, fix, chore, etc.)
- Groups changes by version tags
- Follows Keep a Changelog format
- Run before releases to update CHANGELOG.md

### Tauri Development

```bash
pnpm tauri dev        # Run Tauri in development mode (starts both frontend and backend)
pnpm tauri build      # Build production Tauri application
```

### Testing

No test framework is currently configured. Consider adding Vitest for frontend and standard Rust tests for backend.

## Architecture Overview

### Frontend (React + TypeScript)

**State Management**:

- Local component state using React hooks (useState, useEffect)
- AppState in Rust backend serves as the source of truth
- Frontend calls Tauri commands to interact with backend

**Component Structure**:

```
src/
├── components/       # React components (Dashboard, Sidebar, Header, etc.)
├── command/         # Tauri command wrappers (TypeScript -> Rust)
├── types.ts         # Shared type definitions (DBStatus, DBInstance, etc.)
├── App.tsx          # Root component with view routing
└── index.tsx        # Entry point
```

**View System**: The app uses a single-page navigation model with view types:

- `dashboard`: System overview with resource monitoring
- `instances`: Database list and management
- `logs`: Log viewing (not yet implemented)
- `backup`: Backup management (not yet implemented)
- `settings`: Application settings

### Backend (Rust + Tauri)

**Core Modules**:

```
src-tauri/src/
├── app.rs                    # AppState - central state management
├── command/                  # Tauri command handlers (frontend API)
│   ├── database.rs          # Database operations
│   ├── settings.rs          # Settings management
│   └── system_info.rs       # System resource monitoring
└── core/                     # Core business logic
    ├── db_manager.rs        # Database lifecycle management
    ├── types.rs             # Data structures (DatabaseInfo, GlobalSettings, etc.)
    └── utils.rs             # Utility functions
```

**AppState**: Central state container holding:

- `databases`: HashMap of all installed databases (keyed by ID)
- `settings`: GlobalSettings (theme, language, auto-backup config, etc.)
- `db_manager`: DatabaseManager for process management

**Directory Structure** (managed by DatabaseManager):

```
~/.local-db/          # Default storage path (customizable)
├── bin/              # Database binaries
│   ├── mysql/
│   ├── mongodb/
│   └── ...
├── config/           # Database configuration files
├── data/             # Database data directories
├── logs/             # Database logs
└── backups/          # Backup files
```

### Database Type System

**Rust Enum** (`DatabaseType` in `core/types.rs`):

- Maps to lowercase strings for serialization (e.g., "mysql", "postgresql")
- Defines default ports for each database type
- Used as a unique identifier (only ONE instance per type allowed)

**Frontend Mapping**: Frontend uses string literals matching the lowercase database type.

### Tauri Commands (Frontend-Backend Bridge)

Commands are defined in `src-tauri/src/command/*.rs` and exposed via `invoke_handler` in `lib.rs`:

**Database Commands**:

- `get_databases()` - List all databases
- `get_database(id)` - Get single database by ID
- `get_database_by_type(db_type)` - Get database by type (e.g., "mysql")
- `start_database(id, config?)` - Start database with optional config
- `stop_database(id)` - Stop database
- `restart_database(id)` - Restart database
- `get_database_status(id)` - Get current status
- `delete_database(id, with_data)` - Delete database (optionally with data)
- `install_database(db_type, version?, config?)` - Install new database
- `update_database_autostart(id, auto_start)` - Toggle auto-start

**Settings Commands**:

- `get_settings()` - Get global settings
- `update_settings(settings)` - Update global settings

**System Info Commands**:

- `get_system_info()` - Get OS/platform info
- `get_cpu_usage()` - Get CPU usage percentage
- `get_memory_info()` - Get memory stats
- `get_disk_info()` - Get disk usage

### Frontend Command Wrappers

Located in `src/command/*.ts`:

- `database.ts` - Wraps database-related commands
- `settings.ts` - Wraps settings commands
- `system_info.ts` - Wraps system info commands

Example usage:

```typescript
import { getDatabases, startDatabase } from './command/database';

const databases = await getDatabases();
await startDatabase('db-id-123');
```

## Key Implementation Details

### Auto-Start Mechanism

When Tauri app launches (`lib.rs` setup hook):

1. `AppState::start_autostart_databases()` is called
2. Backend iterates through all databases with `auto_start: true`
3. Each auto-start database is started via `DatabaseManager::start_database()`

### Database Installation Flow

1. Frontend calls `install_database(db_type, version, config)`
2. Backend checks if database type already exists (max 1 per type)
3. Downloads binary from official source
4. Extracts to `~/.local-db/bin/{db_type}/`
5. Creates data/log/config directories
6. Creates `DatabaseInfo` record with unique ID
7. Adds to `AppState.databases`

### Database Status Tracking

- `DatabaseStatus` enum: `Running`, `Stopped`, `NotInstalled`
- `pid` field stores process ID when running
- `DatabaseManager` uses system APIs to check process status

## Code Style Guidelines

### TypeScript

- Strict mode enabled, no `@ts-ignore` or `as any`
- Use enums for constant sets (`DBStatus`)
- Use interfaces for objects (`DBInstance`)
- Component props must have explicit interface
- Import order: React → Third-party → Local

### Rust

- Follow standard Rust conventions
- Use `anyhow::Result` for error handling
- Serialize enums with `#[serde(rename_all = "lowercase")]`
- Keep command handlers thin, delegate to `db_manager` or `AppState`

### Styling (Tailwind CSS 4)

- Dark mode: Use `dark:` prefix consistently
- Color tokens: Primary `#135bec`, background light `#f8fafc`
- Icons: Use `react-icons/fi` (Feather icons)

## Important Files to Know

- **PRD.md**: Complete product requirements document (in Chinese)
- **AGENTS.md**: Detailed coding guidelines for agents
- **docs/**: Platform-specific implementation guides (macos.md, linux.md, windows.md)
- **src-tauri/src/lib.rs**: Tauri app entry point with plugin setup
- **src-tauri/src/app.rs**: AppState - central state management
- **src-tauri/src/core/db_manager.rs**: Core database lifecycle logic
- **src/App.tsx**: Frontend view routing
- **src/types.ts**: Frontend type definitions

## Platform Support

Current priority: **macOS** (first-class support)
Future: Windows and Linux

Platform-specific logic should be isolated in `core/db_manager.rs` and use conditional compilation when necessary.

## Known Limitations

- No test coverage yet
- Backup/restore features planned but not implemented
- Log viewing UI planned but not implemented
- Single instance per database type (by design)
