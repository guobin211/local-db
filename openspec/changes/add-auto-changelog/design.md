## 上下文

当前项目使用 Git 进行版本控制，遵循语义化提交信息格式（feat、fix、chore 等）。需要一个自动化工具从 Git 历史中提取变更信息，生成标准化的 CHANGELOG.md 文件。

**背景**：
- 项目使用 pnpm 作为包管理器
- 主要开发平台为 macOS，需支持跨平台
- 现有提交历史包含中文和英文混合的提交信息
- 项目遵循语义化版本规范

**约束**：
- CHANGELOG 必须符合 Keep a Changelog 格式
- 生成脚本需要跨平台兼容
- 不应依赖外部网络服务
- 应保留现有的手动 CHANGELOG 条目（如果存在）

## 目标 / 非目标

### 目标
- 自动从 Git 提交历史生成结构化的 CHANGELOG.md
- 支持增量更新（新版本追加到文件顶部）
- 按提交类型（feat、fix、chore 等）分类变更
- 支持版本标签识别和分组
- 提供预览模式（不写入文件）
- 生成中文优先的输出内容

### 非目标
- 不实现复杂的变更分析或影响评估
- 不集成到 Git hooks（避免阻塞提交流程）
- 不实现多语言自动翻译（仅支持原始提交语言）
- 不生成 HTML 或其他格式的 CHANGELOG

## 决策

### 决策 1：使用 Node.js 脚本而非 Shell 脚本

**选择**：使用 Node.js (JavaScript) 编写生成脚本

**理由**：
- 项目前端使用 Node.js 生态，开发者熟悉
- 更好的跨平台兼容性（Windows、macOS、Linux）
- 可以使用 npm 包（如 `simple-git`）简化 Git 操作
- 易于解析和格式化 JSON、字符串等数据
- 可以复用项目的 pnpm 依赖管理

### 决策 2：采用 Keep a Changelog 格式

**选择**：生成的 CHANGELOG.md 遵循 Keep a Changelog 1.0.0 标准

**格式示例**：
```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2024-01-15

### Added
- 新增功能描述

### Fixed
- 修复的问题描述

### Changed
- 变更的内容描述
```

**理由**：
- 业界标准，用户熟悉
- 结构清晰，易于阅读
- 支持多种变更类型分类
- 与语义化版本规范配合良好

### 决策 3：基于语义化提交信息分类

**选择**：使用提交信息的类型前缀（feat:、fix: 等）进行分类

**分类映射**：
- `feat:` → Added（新功能）
- `fix:` → Fixed（修复）
- `refactor:` → Changed（重构/变更）
- `perf:` → Changed（性能优化）
- `docs:` → Documentation（文档）
- `style:` → Changed（样式调整）
- `chore:` → Maintenance（维护工作）
- `test:` → Testing（测试相关）

**理由**：
- 项目已遵循语义化提交规范
- 自动分类准确度高
- 减少手动编辑需求

### 决策 4：支持增量更新模式

**选择**：默认为增量更新模式，保留现有 CHANGELOG 内容，新版本插入到顶部

**行为**：
- 检测 CHANGELOG.md 是否存在
- 如存在，解析现有版本记录
- 仅生成尚未记录的版本变更
- 将新内容插入到 [Unreleased] 后面

**理由**：
- 保留手动编辑的内容
- 避免重复生成旧版本
- 支持渐进式迁移

### 考虑的替代方案

#### 替代方案 1：使用现成的 CHANGELOG 生成工具

**选项**：
- `conventional-changelog`（npm 包）
- `standard-version`（自动化版本管理）
- `release-it`（发布自动化工具）

**为什么不选择**：
- `conventional-changelog` 配置复杂，定制化需求高
- `standard-version` 和 `release-it` 包含版本管理功能，超出需求范围
- 自定义脚本更灵活，可以完全控制输出格式
- 减少外部依赖

#### 替代方案 2：使用 Shell 脚本

**为什么不选择**：
- Windows 兼容性差（需要 Git Bash 或 WSL）
- 字符串处理和格式化复杂
- 难以维护和扩展

#### 替代方案 3：集成到 Git Hooks

**为什么不选择**：
- 会阻塞提交或推送流程
- 开发者体验差（每次提交都触发）
- CHANGELOG 生成应该是发布流程的一部分，不是提交流程

## 技术实现细节

### 脚本位置
`scripts/generate-changelog.js`

### 核心依赖
- `simple-git`：用于读取 Git 历史和标签
- Node.js 内置模块：`fs`、`path`

### 主要功能模块
1. **Git 历史读取**：获取所有提交和标签
2. **提交解析**：提取类型、作用域、描述
3. **版本分组**：按 Git 标签分组提交
4. **格式化输出**：生成 Keep a Changelog 格式
5. **文件合并**：增量更新现有 CHANGELOG.md

### package.json 脚本命令
```json
{
  "scripts": {
    "changelog": "node scripts/generate-changelog.js",
    "changelog:preview": "node scripts/generate-changelog.js --dry-run"
  }
}
```

## 风险 / 权衡

### 风险 1：提交信息不规范导致分类错误

**缓解措施**：
- 在文档中强调语义化提交的重要性
- 脚本提供 fallback 处理（未分类的提交归入 "Other" 或 "Changed"）
- 生成后可手动调整 CHANGELOG

### 风险 2：中英文混合输出可读性问题

**缓解措施**：
- 保持原始提交语言（不做翻译）
- 在类型标题使用中文（如"新增功能"而非"Added"）
- 在 CHANGELOG 开头说明格式标准

### 风险 3：现有 CHANGELOG 被覆盖

**缓解措施**：
- 默认增量更新模式
- 提供 `--overwrite` 标志供完全重写
- 建议在首次运行前备份现有文件

### 权衡：自动化 vs 手动控制

**选择**：半自动化（脚本生成 + 人工审核）

**理由**：
- 完全自动化可能生成质量不佳的内容
- 保留人工审核可以确保 CHANGELOG 质量
- 发布是重要流程，应有人工检查环节

## 迁移计划

### 步骤 1：准备阶段
1. 创建 `scripts/generate-changelog.js` 脚本
2. 安装必要依赖（`pnpm add -D simple-git`）
3. 在 `package.json` 添加脚本命令

### 步骤 2：首次生成
1. 备份现有 CHANGELOG.md（如果存在）
2. 运行 `pnpm changelog:preview` 预览生成结果
3. 检查输出质量和格式
4. 运行 `pnpm changelog` 生成正式文件
5. 手动审核和调整生成内容

### 步骤 3：集成到发布流程
1. 在发布清单中添加"生成 CHANGELOG"步骤
2. 更新 CLAUDE.md 和 AGENTS.md 文档
3. 在团队中宣传新的 CHANGELOG 生成方式

### 步骤 4：持续优化
1. 根据使用反馈调整脚本逻辑
2. 优化提交信息分类规则
3. 考虑是否需要 CI 自动化

### 回滚计划
- 如果脚本生成的 CHANGELOG 质量不佳，可以恢复备份文件
- 继续手动维护 CHANGELOG
- 脚本不影响 Git 历史，删除即可

## 待决问题

1. **是否需要支持多语言输出？**
   - 当前决策：仅支持中文类型标题 + 原始提交语言
   - 后续可考虑添加 `--lang=en` 选项

2. **是否需要过滤某些类型的提交？**
   - 例如：是否在 CHANGELOG 中包含 `chore:` 或 `test:` 类型的提交？
   - 建议：默认包含所有类型，提供配置选项

3. **是否需要支持作用域（scope）显示？**
   - 例如：`feat(database): 新增功能` 中的 `database`
   - 建议：初期不显示作用域，保持简洁

4. **是否需要链接到 commit 或 PR？**
   - 例如：在每个变更后添加 `(#123)` 或 commit hash
   - 建议：可选功能，通过 `--with-links` 启用
