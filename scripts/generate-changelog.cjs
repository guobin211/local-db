#!/usr/bin/env node

const { simpleGit } = require('simple-git');
const fs = require('fs');
const path = require('path');

// 提交类型映射到 Keep a Changelog 类别
const TYPE_MAP = {
  feat: '新增功能',
  fix: '修复',
  refactor: '变更',
  perf: '变更',
  docs: '文档',
  style: '变更',
  chore: '维护',
  test: '测试',
};

// 解析命令行参数
const args = process.argv.slice(2);
const isDryRun = args.includes('--dry-run') || args.includes('--preview');
const isOverwrite = args.includes('--overwrite');

// 初始化 git
const git = simpleGit();
const changelogPath = path.join(process.cwd(), 'CHANGELOG.md');

// 解析提交信息
function parseCommit(message) {
  // 匹配格式: type(scope): description 或 type: description
  const match = message.match(/^(\w+)(?:\(([^)]+)\))?: (.+)$/);

  if (match) {
    const [, type, scope, description] = match;
    return {
      type: type.toLowerCase(),
      scope: scope || null,
      description: description.trim(),
    };
  }

  // 不匹配格式的提交归入 "变更"
  return {
    type: 'other',
    scope: null,
    description: message.split('\n')[0].trim(),
  };
}

// 按类型分组提交
function groupCommitsByType(commits) {
  const groups = {};

  commits.forEach((commit) => {
    const parsed = parseCommit(commit.message);
    const category = TYPE_MAP[parsed.type] || '变更';

    if (!groups[category]) {
      groups[category] = [];
    }

    groups[category].push({
      description: parsed.description,
      hash: commit.hash.substring(0, 7),
      scope: parsed.scope,
    });
  });

  return groups;
}

// 格式化日期为 YYYY-MM-DD
function formatDate(date) {
  const d = new Date(date);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 生成版本部分的 Markdown
function generateVersionSection(version, date, commits) {
  const groups = groupCommitsByType(commits);
  const versionHeader = version === 'Unreleased'
    ? `## [${version}]`
    : `## [${version}] - ${date}`;

  let section = `${versionHeader}\n\n`;

  // 按预定义顺序输出类别
  const categoryOrder = ['新增功能', '修复', '变更', '文档', '维护', '测试'];

  categoryOrder.forEach((category) => {
    if (groups[category] && groups[category].length > 0) {
      section += `### ${category}\n\n`;
      groups[category].forEach((item) => {
        section += `- ${item.description}\n`;
      });
      section += '\n';
    }
  });

  return section;
}

// 生成完整的 CHANGELOG
async function generateChangelog() {
  try {
    // 检查是否是 Git 仓库
    const isRepo = await git.checkIsRepo();
    if (!isRepo) {
      console.error('错误：当前目录不是 Git 仓库');
      process.exit(1);
    }

    console.log('正在读取 Git 提交历史...');

    // 获取所有标签
    const tags = await git.tags();
    const tagList = tags.all.reverse(); // 最新的在前

    // 获取所有提交
    const log = await git.log();
    const commits = log.all;

    console.log(`找到 ${commits.length} 个提交，${tagList.length} 个标签`);

    // 按版本分组提交
    const versions = [];
    let lastTagHash = null;

    // 处理未发布的提交 (Unreleased)
    if (tagList.length > 0) {
      const latestTag = tagList[0];
      const latestTagCommit = commits.find((c) => c.refs.includes(`tag: ${latestTag}`));

      if (latestTagCommit) {
        const unreleasedCommits = [];
        for (const commit of commits) {
          if (commit.hash === latestTagCommit.hash) break;
          unreleasedCommits.push(commit);
        }

        if (unreleasedCommits.length > 0) {
          versions.push({
            version: 'Unreleased',
            date: null,
            commits: unreleasedCommits,
          });
        }

        lastTagHash = latestTagCommit.hash;
      }
    } else {
      // 没有标签，所有提交都是 Unreleased
      versions.push({
        version: 'Unreleased',
        date: null,
        commits: commits,
      });
    }

    // 处理已发布的版本
    for (let i = 0; i < tagList.length; i++) {
      const tag = tagList[i];
      const nextTag = tagList[i + 1];

      const tagCommit = commits.find((c) => c.refs.includes(`tag: ${tag}`));
      if (!tagCommit) continue;

      const versionCommits = [];
      let startAdding = false;

      for (const commit of commits) {
        if (commit.hash === tagCommit.hash) {
          startAdding = true;
        }

        if (startAdding) {
          versionCommits.push(commit);

          if (nextTag) {
            const nextTagCommit = commits.find((c) => c.refs.includes(`tag: ${nextTag}`));
            if (nextTagCommit && commit.hash === nextTagCommit.hash) {
              versionCommits.pop(); // 不包含下一个标签的提交
              break;
            }
          }
        }
      }

      versions.push({
        version: tag.replace(/^v/, ''), // 移除 'v' 前缀
        date: formatDate(tagCommit.date),
        commits: versionCommits,
      });
    }

    // 生成 CHANGELOG 内容
    let changelogContent = '# Changelog\n\n';
    changelogContent += 'All notable changes to this project will be documented in this file.\n\n';
    changelogContent += 'The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\n';
    changelogContent += 'and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n';

    // 添加各版本内容
    versions.forEach((ver) => {
      changelogContent += generateVersionSection(ver.version, ver.date, ver.commits);
    });

    // 输出或写入文件
    if (isDryRun) {
      console.log('\n=== 预览模式 - 生成的 CHANGELOG 内容 ===\n');
      console.log(changelogContent);
      console.log('\n=== 预览结束（未写入文件）===');
    } else {
      fs.writeFileSync(changelogPath, changelogContent, 'utf-8');
      console.log(`\n✅ CHANGELOG.md 已成功生成: ${changelogPath}`);
      console.log(`   包含 ${versions.length} 个版本`);
    }

  } catch (error) {
    console.error('生成 CHANGELOG 时出错:', error.message);
    process.exit(1);
  }
}

// 执行
generateChangelog();
