## 1. 设计与规划
- [x] 1.1 选择 CHANGELOG 生成工具或实现方式（git log 解析 vs 现有工具）
- [x] 1.2 确定 CHANGELOG 格式标准（Keep a Changelog、语义化版本）
- [x] 1.3 定义提交信息解析规则（支持 feat、fix、chore、docs、refactor 等）

## 2. 实现生成脚本
- [x] 2.1 创建 changelog 生成脚本文件
- [x] 2.2 实现 Git 提交历史读取功能
- [x] 2.3 实现提交信息分类逻辑（按类型和作用域分组）
- [x] 2.4 实现版本标签识别和分组
- [x] 2.5 实现 CHANGELOG.md 格式化输出
- [ ] 2.6 支持增量更新（追加新版本而不覆盖旧记录）

## 3. 集成到项目
- [x] 3.1 在 package.json 中添加 `changelog` 命令
- [x] 3.2 在 package.json 中添加 `changelog:preview` 命令（预览不写入）
- [x] 3.3 更新 README.md 或开发文档说明 changelog 使用方法
- [ ] 3.4 在发布流程文档中添加生成 CHANGELOG 的步骤

## 4. 可选：CI/CD 自动化
- [ ] 4.1 评估是否需要在 CI 中自动生成 CHANGELOG
- [ ] 4.2 如需要，配置 GitHub Actions 自动生成并提交 CHANGELOG
- [ ] 4.3 设置发布时自动更新 CHANGELOG 的触发条件

## 5. 测试与验证
- [x] 5.1 使用现有提交历史测试生成效果
- [x] 5.2 验证不同类型的提交信息是否正确分类
- [x] 5.3 验证版本标签是否正确识别
- [x] 5.4 检查生成的 CHANGELOG.md 格式是否符合标准
- [x] 5.5 确保脚本在不同操作系统上可运行（macOS、Linux、Windows）

## 6. 文档更新
- [x] 6.1 编写 CHANGELOG 生成工具的使用文档
- [ ] 6.2 更新 Git 提交规范文档（强调语义化提交的重要性）
- [ ] 6.3 在 CLAUDE.md 或 AGENTS.md 中添加 changelog 相关指南
