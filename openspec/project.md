# 项目 上下文

## 目的

**local-db** 是一个跨平台的本地数据库管理工具，旨在简化开发者对多种数据库的本地安装、配置和管理。

**核心目标**：
- 提供一键安装、启动、停止多种数据库的能力
- 支持 MySQL、PostgreSQL、MongoDB、Redis、Qdrant、Neo4j、SeekDB、SurrealDB
- 跨平台支持（macOS 优先，未来支持 Windows 和 Linux）
- 提供直观的 GUI 界面和系统资源监控
- 简化数据库配置和备份管理

## 技术栈

### 前端
- **React 19** - UI 框架
- **TypeScript** - 类型安全
- **Vite** - 构建工具
- **Tailwind CSS 4** - 样式框架
- **react-icons** (Feather Icons) - 图标库
- **pnpm** - 包管理器

### 后端
- **Rust** - 核心业务逻辑
- **Tauri 2** - 桌面应用框架
- **serde** - 序列化/反序列化
- **tokio** - 异步运行时
- **anyhow** - 错误处理

### 工具链
- **Prettier** - JS/TS 代码格式化
- **rustfmt** - Rust 代码格式化
- **TypeScript Compiler** - 类型检查

## 项目约定

### 代码风格

**TypeScript**:
- 启用严格模式，禁止使用 `@ts-ignore` 或 `as any`
- 使用 `enum` 定义常量集合（如 `DBStatus`）
- 使用 `interface` 定义对象类型（如 `DBInstance`）
- 组件 props 必须有显式的接口定义
- 导入顺序：React → 第三方库 → 本地模块
- 使用 Prettier 格式化（`pnpm fmt:js`）

**Rust**:
- 遵循标准 Rust 编码规范
- 使用 `anyhow::Result` 进行错误处理
- 枚举序列化使用 `#[serde(rename_all = "lowercase")]`
- 命令处理器保持简洁，业务逻辑委托给 `db_manager` 或 `AppState`
- 使用 `cargo fmt` 格式化（`pnpm fmt:rs`）

**Tailwind CSS**:
- 暗色模式：统一使用 `dark:` 前缀
- 主色调：Primary `#135bec`，背景色 `#f8fafc` (浅色模式)
- 保持样式类的语义化和一致性

### 架构模式

**前后端分离架构**:
- 前端：React 组件 + 本地状态管理（useState/useEffect）
- 后端：Rust AppState 作为唯一数据源
- 通信：通过 Tauri Commands（前端 `invoke()` → 后端命令处理器）

**核心模块**:
```
src-tauri/src/
├── app.rs              # AppState - 中央状态管理
├── command/            # Tauri 命令处理器（前端 API）
│   ├── database.rs    # 数据库操作
│   ├── settings.rs    # 设置管理
│   └── system_info.rs # 系统资源监控
└── core/               # 核心业务逻辑
    ├── db_manager.rs  # 数据库生命周期管理
    ├── types.rs       # 数据结构
    └── utils.rs       # 工具函数
```

**状态管理**:
- `AppState` 持有：
  - `databases`: HashMap<String, DatabaseInfo> - 所有数据库实例
  - `settings`: GlobalSettings - 全局设置
  - `db_manager`: DatabaseManager - 进程管理器

**目录结构**:
```
~/.local-db/          # 默认存储路径（可配置）
├── bin/              # 数据库二进制文件
├── config/           # 配置文件
├── data/             # 数据目录
├── logs/             # 日志文件
└── backups/          # 备份文件
```

### 测试策略

**当前状态**：暂未配置测试框架

**计划**：
- 前端：使用 Vitest 进行单元测试和组件测试
- 后端：使用标准 Rust 测试框架（`#[cfg(test)]` 模块）
- 集成测试：测试 Tauri 命令的端到端流程

### Git工作流

**分支策略**：
- `main` - 主分支，始终保持可发布状态
- 功能分支：从 `main` 创建，完成后合并回 `main`

**提交约定**：
- 遵循语义化提交信息格式
- 类型：`feat`、`fix`、`chore`、`docs`、`refactor`、`test`
- 格式：`<类型>: <简短描述>`（中文或英文）
- 示例：
  - `feat: 添加 PostgreSQL 支持`
  - `fix: 修复数据库启动失败的问题`
  - `chore: 更新依赖版本`

**Co-Authored-By**：
- 使用 AI 助手时添加：`Co-Authored-By: Claude <noreply@anthropic.com>`

## 领域上下文

### 数据库管理领域知识

**支持的数据库类型**：
- **MySQL** - 关系型数据库，默认端口 3306
- **PostgreSQL** - 关系型数据库，默认端口 5432
- **MongoDB** - 文档型数据库，默认端口 27017
- **Redis** - 键值存储，默认端口 6379
- **Qdrant** - 向量数据库，默认端口 6333
- **Neo4j** - 图数据库，默认端口 7474/7687
- **SeekDB** - 时序数据库
- **SurrealDB** - 多模型数据库，默认端口 8000

**数据库生命周期**：
1. **安装**：下载官方二进制 → 解压到 `~/.local-db/bin/` → 创建目录结构
2. **配置**：生成配置文件到 `~/.local-db/config/`
3. **启动**：通过 `DatabaseManager` 启动进程，记录 PID
4. **监控**：检查进程状态，更新 `DatabaseStatus`
5. **停止**：终止进程，清理 PID
6. **删除**：可选删除数据目录

**状态追踪**：
- `Running` - 进程正在运行（有 PID）
- `Stopped` - 已安装但未运行
- `NotInstalled` - 未安装

## 重要约束

### 技术约束
1. **单实例限制**：每种数据库类型只能安装一个实例（后端强制执行）
2. **平台优先级**：macOS 是第一优先支持平台，Windows 和 Linux 为未来计划
3. **进程管理**：数据库进程由 Tauri 后端管理，非系统服务
4. **目录权限**：需要读写 `~/.local-db/` 目录的权限

### 业务约束
1. **数据安全**：删除数据库时必须明确用户是否保留数据
2. **自动启动**：支持应用启动时自动启动指定数据库
3. **配置持久化**：所有配置必须持久化到磁盘

### 开发约束
1. **无过度工程**：只实现明确需求的功能，避免提前优化
2. **保持简洁**：不添加不必要的错误处理、抽象或特性开关
3. **最小化复杂度**：三行相似代码优于过早抽象

## 外部依赖

### 数据库官方源
- 各数据库的官方下载源（用于安装时下载二进制文件）
- 需要网络连接以下载数据库安装包

### 系统依赖
- **macOS**: 系统原生 API 用于进程管理和资源监控
- **Windows** (计划): Windows API
- **Linux** (计划): procfs 和系统调用

### Tauri 插件
- Tauri 核心插件（文件系统、进程管理、对话框等）
- 系统信息获取能力

### 前端运行时
- Node.js (开发时)
- Vite 开发服务器（开发时，端口 1420）

### 无外部 API 依赖
- 本项目为纯本地应用，不依赖外部 Web 服务或 API
