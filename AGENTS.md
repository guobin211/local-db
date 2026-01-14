<!-- OPENSPEC:START -->

# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:

- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:

- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# AGENTS.md - Coding Guidelines for local-db

This guide provides essential information for agentic coding systems working in this repository.

## Project Overview

- **Stack**: Tauri 2 + React 19 + TypeScript + Vite + Tailwind CSS 4
- **Purpose**: Local database management tool (MongoDB, Redis, Neo4j, etc.)
- **Icons**: React Icons (use `react-icons/fi` for Feather icons)

## Build, Lint, and Test Commands

### Development

```bash
pnpm dev              # Start Vite dev server (port 1420)
pnpm preview          # Preview production build
```

### Build

```bash
pnpm build            # TypeScript compile + Vite production build
```

### Formatting

```bash
pnpm fmt              # Format JS/TS + Rust files
pnpm fmt:js           # Format JS/TS with Prettier
pnpm fmt:rs           # Format Rust with cargo fmt
```

### Testing

⚠️ **No test framework configured** - Add tests if needed (Vitest recommended)

- No test commands currently available
- Test files should follow naming: `*.test.ts` or `*.spec.ts`

### Build Verification (IMPORTANT FOR AGENTS)

🚫 **NEVER use `tauri build` for testing** - Use these lightweight verification commands instead:

```bash
# Rust backend check only (fast)
cd src-tauri && cargo check

# Frontend build only (fast)
pnpm build

# Combined verification (recommended for agents)
cd src-tauri && cargo check && pnpm build
```

**Why not `tauri build`?**

- Full Tauri build takes 5-10+ minutes per platform
- Requires platform-specific dependencies
- Only needed for release preparation, not development/testing
- Use CI/CD workflow for actual cross-platform builds

**When to use `tauri build`:**

- Preparing for production release
- Testing native bundler behavior
- Only when explicitly requested by user

## Code Style Guidelines

### TypeScript Configuration

- **Strict mode**: Enabled
- **Target**: ES2020
- **Module**: ESNext (bundler mode)
- **JSX**: `react-jsx` transform
- **Unused checks**: `noUnusedLocals`, `noUnusedParameters` enabled
- **Never suppress errors**: No `@ts-ignore`, `@ts-expect-error`, or `as any`

### Import Ordering

```typescript
// 1. React imports first
import React, { useState, useEffect } from 'react';

// 2. Third-party library imports
import { BarChart, Bar } from 'recharts';

// 3. Local relative imports
import { DBInstance, ViewType } from '../types';
import { Sidebar } from './components/Sidebar';
```

### Component Structure

```typescript
import React from 'react';
import { ViewType } from '../types';

// Define interface for props
interface ComponentProps {
  view: ViewType;
  onChange: (view: ViewType) => void;
}

// Functional component with explicit typing
export const ComponentName: React.FC<ComponentProps> = ({ view, onChange }) => {
  // State hooks at the top
  const [state, setState] = useState(null);

  // Effects after state
  React.useEffect(() => {
    // Effect logic
  }, [dependency]);

  // Handler functions
  const handleClick = () => {
    // Handler logic
  };

  // Conditional rendering helper functions
  const renderContent = () => {
    // Return JSX
  };

  // Return JSX
  return <div>...</div>;
};

// Default export for main components
export default App;
```

### Type Definitions

```typescript
// Use enums for constant sets
export enum DBStatus {
  RUNNING = 'Running',
  STOPPED = 'Stopped',
  NOT_INSTALLED = 'Not Installed'
}

// Use interfaces for objects
export interface DBInstance {
  id: string;
  name: string;
  version: string;
  type: string;
  status: DBStatus;
  port: string;
  meta: string;
  icon: string;
  colorClass: string;
}

// Use type aliases for unions/aliases
export type ViewType = 'dashboard' | 'instances' | 'logs' | 'backup' | 'settings';
```

### Naming Conventions

- **Components**: PascalCase (`Dashboard`, `InstancesView`)
- **Functions/Variables**: camelCase (`handleClick`, `searchQuery`)
- **Constants**: UPPER_SNAKE_CASE (`MOCK_DATABASES`, `DB_STATUS`)
- **Types/Interfaces**: PascalCase (`ViewType`, `DBInstance`)
- **Booleans**: Prefix with `is/has/can` (`isDark`, `hasError`)

### Formatting (Prettier)

```json
{
  "printWidth": 120,
  "tabWidth": 2,
  "useTabs": false,
  "semi": true,
  "singleQuote": true,
  "trailingComma": "none",
  "bracketSpacing": true,
  "arrowParens": "avoid",
  "endOfLine": "lf"
}
```

### Styling Guidelines (Tailwind CSS 4)

```typescript
// Dark mode support with dark: prefix
<div className="bg-white dark:bg-card-dark text-slate-900 dark:text-white">

// Responsive with md/lg: prefixes
<div className="grid grid-cols-1 gap-4 md:grid-cols-3">

// Common patterns:
// - Cards: `rounded-xl border border-gray-200 dark:border-border-dark bg-white dark:bg-card-dark shadow-sm`
// - Primary text: `text-slate-900 dark:text-white`
// - Secondary text: `text-slate-500 dark:text-[#9da6b9]`
// - Primary action: `bg-primary hover:bg-primary/90 text-white rounded-lg px-4 py-1.5`
// - Icon wrapper: `flex size-10 items-center justify-center rounded-lg bg-[color]/10 text-[color]`

// React Icons (Feather Icons):
import { FiHome, FiSearch, FiSettings } from 'react-icons/fi';

// Usage with size prop:
<FiHome size={20} />
<FiSearch size={18} />

// With styling:
<FiSettings size={20} className="text-slate-500" />
```

### Error Handling

- No specific patterns observed in codebase
- When adding error handling, use try-catch with proper typing
- Consider error boundaries for React components
- Validate props with TypeScript types and runtime checks if needed

### State Management

- Use React hooks (`useState`, `useEffect`) for local component state
- Pass state up via callbacks for parent communication
- Use TypeScript interfaces for all state objects

### File Organization

```
src/
├── components/          # React components (one file per component)
├── command/            # Tauri command handlers (Rust backend comms)
├── types.ts            # Shared type definitions and enums
├── App.tsx             # Root component
└── index.tsx           # Entry point
```

### Key Color Tokens (Tailwind)

- Primary: `#135bec` (blue)
- Background light: `#f8fafc`
- Background dark: Custom dark card colors
- Border dark: Custom border colors
- Success: `text-green-500`
- Warning: `text-amber-500`
- Danger: `text-red-500`

## When Delegating Visual Changes

If you need to modify UI/UX (colors, spacing, layout, animations), delegate to `frontend-ui-ux-engineer`. Pure logic changes in frontend files can be handled directly.

## Verification Checklist

Before marking work complete:

- [ ] Run `pnpm build` - TypeScript compilation must pass
- [ ] Run `pnpm fmt:js` - Code must be formatted
- [ ] Run `lsp_diagnostics` on changed files - No errors allowed
- [ ] Dark mode classes present when relevant (`dark:bg-...`, `dark:text-...`)
- [ ] Material Symbols icons used consistently
