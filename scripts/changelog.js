import fs from 'node:fs/promises';
import path from 'node:path';

const ARTIFACTS_DIR = process.env.ARTIFACTS_DIR || './artifacts';
const OUTPUT_FILE = process.env.OUTPUT_FILE || path.join(ARTIFACTS_DIR, 'CHANGELOG.md');

function getRepo() {
  const repo = process.env.GITHUB_REPOSITORY;
  if (!repo) {
    throw new Error('GITHUB_REPOSITORY 未设置');
  }
  return repo;
}

function getTag() {
  const refName = process.env.GITHUB_REF_NAME;
  if (refName) {
    return refName;
  }
  const ref = process.env.GITHUB_REF;
  if (!ref) {
    throw new Error('GITHUB_REF_NAME 和 GITHUB_REF 都未设置');
  }
  const parts = ref.split('/');
  return parts[parts.length - 1];
}

function detectPlatformLabel(targetTriple) {
  if (targetTriple === 'aarch64-apple-darwin') {
    return 'macOS (Apple Silicon)';
  }
  if (targetTriple === 'x86_64-apple-darwin') {
    return 'macOS (Intel)';
  }
  if (targetTriple === 'x86_64-unknown-linux-gnu') {
    return 'Linux (x86_64)';
  }
  if (targetTriple === 'x86_64-pc-windows-msvc') {
    return 'Windows (x86_64)';
  }
  return targetTriple;
}

function isInstallerFile(fileName) {
  const lower = fileName.toLowerCase();
  const installerExts = ['.dmg', '.app.tar.gz', '.appimage', '.msi', '.exe', '.deb', '.rpm', '.zip', '.tar.gz'];
  if (lower.endsWith('.sig') || lower.endsWith('.sha256') || lower.endsWith('.sha256sum')) {
    return false;
  }
  return installerExts.some((ext) => lower.endsWith(ext));
}

async function collectArtifacts() {
  const result = [];
  const entries = await fs.readdir(ARTIFACTS_DIR, { withFileTypes: true });
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    const artifactDir = path.join(ARTIFACTS_DIR, entry.name);
    const prefix = 'local-db-';
    const targetTriple = entry.name.startsWith(prefix) ? entry.name.slice(prefix.length) : entry.name;
    const platformLabel = detectPlatformLabel(targetTriple);
    const files = await fs.readdir(artifactDir);
    for (const fileName of files) {
      if (!isInstallerFile(fileName)) {
        continue;
      }
      result.push({
        platform: platformLabel,
        targetTriple,
        fileName
      });
    }
  }
  return result;
}

function buildDownloadUrl(repo, tag, fileName) {
  return `https://github.com/${repo}/releases/download/${tag}/${fileName}`;
}

function groupByPlatform(artifacts) {
  const map = new Map();
  for (const artifact of artifacts) {
    const key = artifact.platform;
    if (!map.has(key)) {
      map.set(key, []);
    }
    map.get(key).push(artifact);
  }
  return map;
}

function buildMarkdown(repo, tag, artifacts) {
  const lines = [];
  lines.push(`# local-db ${tag}`);
  lines.push('');
  lines.push('构建产物下载链接：');
  lines.push('');
  if (artifacts.length === 0) {
    lines.push('- 暂未发现可用的安装包构建产物。');
    return lines.join('\n');
  }
  const grouped = groupByPlatform(artifacts);
  for (const [platform, items] of grouped.entries()) {
    lines.push(`- ${platform}`);
    for (const artifact of items) {
      const url = buildDownloadUrl(repo, tag, artifact.fileName);
      lines.push(`  - [${artifact.fileName}](${url})`);
    }
  }
  lines.push('');
  lines.push('以上链接为当前版本的安装包下载地址，可直接点击下载。');
  return lines.join('\n');
}

async function main() {
  const repo = getRepo();
  const tag = getTag();
  const artifacts = await collectArtifacts();
  const markdown = buildMarkdown(repo, tag, artifacts);
  await fs.mkdir(path.dirname(OUTPUT_FILE), { recursive: true });
  await fs.writeFile(OUTPUT_FILE, markdown, 'utf8');
  console.log(`已生成 changelog: ${OUTPUT_FILE}`);
}

main().catch((error) => {
  console.error('生成 changelog 失败:', error);
  process.exit(1);
});
