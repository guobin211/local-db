# 变更：自动生成 CHANGELOG.md 文件

## 为什么

当前项目缺少对变更历史的自动化追踪机制，开发者需要手动维护 CHANGELOG.md 文件，这容易导致：
- 遗漏重要变更记录
- 格式不一致
- 发布时忘记更新 CHANGELOG
- 维护成本高且容易出错

通过自动从 Git 提交历史生成 CHANGELOG，可以确保每次发布都有完整、一致的变更记录，并减少手动维护负担。

## 变更内容

- 添加自动生成 CHANGELOG.md 的构建脚本
- 从 Git 提交信息中提取语义化的变更类型（feat、fix、chore 等）
- 按版本和日期组织变更记录
- 支持手动运行和 CI/CD 集成
- 生成符合 Keep a Changelog 格式的输出
- 支持多语言输出（中文优先，可选英文）

## 影响

- **受影响规范**：新增 `build-automation` 功能规范
- **受影响代码**：
  - 新增：`scripts/generate-changelog.js` 或 `scripts/generate-changelog.sh`
  - 可能修改：`package.json`（添加脚本命令）
  - 可能修改：`.github/workflows/` 中的 CI 配置（如需自动化）
- **用户体验**：用户可以在 CHANGELOG.md 中清晰查看每个版本的变更内容
- **开发流程**：发布前需要运行 changelog 生成脚本
