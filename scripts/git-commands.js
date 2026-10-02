import { execSync } from 'child_process';
import path from 'path';

function runGit(args, cwd = process.cwd()) {
  try {
    return execSync(`git ${args}`, { cwd, encoding: 'utf8', maxBuffer: 10 * 1024 * 1024 }).trim();
  } catch (err) {
    return '';
  }
}

export function getRepoInfo() {
  const repoName = process.env.GITHUB_REPOSITORY ? process.env.GITHUB_REPOSITORY.split('/')[1] : path.basename(process.cwd());
  const owner = process.env.GITHUB_REPOSITORY ? process.env.GITHUB_REPOSITORY.split('/')[0] : 'local';
  const desc = runGit('config --get gitweb.description') || 'A git repository hosted entirely on GitHub Actions + Pages';
  const head = runGit('rev-parse --short HEAD') || 'HEAD';
  const defaultBranch = runGit('rev-parse --abbrev-ref HEAD') || 'main';
  return { repoName, owner, desc, head, defaultBranch };
}

export function getRefs() {
  const raw = runGit('for-each-ref --sort=-committerdate --format="%(refname:short)%09%(refname)%09%(objectname:short)%09%(committerdate:relative)%09%(authorname)%09%(subject)" refs/heads refs/tags');
  if (!raw) return { branches: [], tags: [] };
  const branches = [];
  const tags = [];
  raw.split('\n').filter(Boolean).forEach(line => {
    const [short, full, commit, age, author, subject] = line.split('\t');
    if (full.startsWith('refs/heads/')) {
      branches.push({ name: short, commit, age, author, subject });
    } else if (full.startsWith('refs/tags/')) {
      tags.push({ name: short, commit, age, author, subject });
    }
  });
  return { branches, tags };
}

export function getLog(limit = 100, ref = 'HEAD') {
  const format = '%H%x09%h%x09%an%x09%ae%x09%at%x09%s';
  const raw = runGit(`log -n ${limit} --format="${format}" ${ref}`);
  if (!raw) return [];
  return raw.split('\n').filter(Boolean).map(line => {
    const [hash, shortHash, author, email, timestamp, subject] = line.split('\t');
    const date = new Date(parseInt(timestamp, 10) * 1000).toISOString().split('T')[0];
    return { hash, shortHash, author, email, date, subject };
  });
}

export function getCommitDetails(hash) {
  const raw = runGit(`show --stat --patch --format=fuller ${hash}`);
  const stat = runGit(`show --stat --oneline ${hash}`);
  return { fullText: raw, stat };
}

export function getTree(ref = 'HEAD', subpath = '') {
  const target = subpath ? `${ref}:${subpath}` : ref;
  const raw = runGit(`ls-tree -l ${target}`);
  if (!raw) return [];
  return raw.split('\n').filter(Boolean).map(line => {
    const [mode, type, sha, size, ...nameParts] = line.trim().split(/\s+/);
    const name = nameParts.join(' ');
    return { mode, type, sha, size: size === '-' ? '' : size, name };
  });
}

export function getBlob(sha) {
  return runGit(`cat-file -p ${sha}`);
}

export function getBlame(path, ref = 'HEAD') {
  return runGit(`blame --line-porcelain ${ref} -- "${path}"`);
}

export function getReadme() {
  const files = ['README.md', 'README', 'readme.md', 'README.txt'];
  for (const f of files) {
    const content = runGit(`show HEAD:${f}`);
    if (content) return { filename: f, content };
  }
  return null;
}
