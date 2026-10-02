import fs from 'fs';
import path from 'path';
import crypto from 'crypto';
import {
  getRepoInfo,
  getRefs,
  getLog,
  getCommitDetails,
  getTree,
  getBlob,
  getBlame,
  getStats,
  getReadme
} from './scripts/git-commands.js';
import { renderLayout, escapeHtml } from './scripts/generate-html.js';

const OUT_DIR = path.resolve('gh-pages');

function ensureDir(dir) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

function writePage(relPath, content) {
  const fullPath = path.join(OUT_DIR, relPath);
  ensureDir(path.dirname(fullPath));
  fs.writeFileSync(fullPath, content, 'utf8');
}

function getGravatar(email) {
  const hash = crypto.createHash('md5').update((email || '').trim().toLowerCase()).digest('hex');
  return `<span class='libravatar'><img class='inline' src='https://seccdn.libravatar.org/avatar/${hash}?s=13&amp;d=retro' /><img class='onhover' src='https://seccdn.libravatar.org/avatar/${hash}?s=128&amp;d=retro' /></span>`;
}

function colorizeDiff(rawDiff) {
  const lines = rawDiff.split('\n');
  return lines.map(line => {
    const esc = escapeHtml(line);
    if (line.startsWith('+') && !line.startsWith('+++')) {
      return `<div class='add'>${esc}</div>`;
    } else if (line.startsWith('-') && !line.startsWith('---')) {
      return `<div class='del'>${esc}</div>`;
    } else if (line.startsWith('@@')) {
      return `<div class='hunk'>${esc}</div>`;
    }
    return `<div>${esc}</div>`;
  }).join('');
}

async function build() {
  console.log('Generating pixel-perfect cgit...');
  const repoInfo = getRepoInfo();
  const refs = getRefs();
  repoInfo.branches = refs.branches;
  const logs = getLog(100);
  const readme = getReadme();
  const stats = getStats();

  // 1. About View
  let aboutHtml = `<div class='cgit-readme'><table class='tabs'></table><div class='readme-body'><pre>${readme ? escapeHtml(readme.content) : 'No README'}</pre></div></div>`;
  writePage('about/index.html', renderLayout({
    title: 'about',
    activeTab: 'about',
    repoInfo,
    content: aboutHtml,
    rootPath: '../'
  }));

  // 2. Summary View (index.html)
  const branchesHtml = `
<table summary='repository info' class='list nowrap'>
<tr class='nohover'><th class='left'>Branch</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left' colspan='2'>Age</th></tr>
${refs.branches.map(b => `
<tr><td><a href='tree/?h=${escapeHtml(b.name)}'>${escapeHtml(b.name)}</a></td><td><a href='commit/?id=${b.commit}'>${escapeHtml(b.subject)}</a></td><td>${escapeHtml(b.author)}</td><td colspan='2'><span class='age-days' data-ut='${Math.floor(Date.now()/1000)}'>${b.age}</span></td></tr>
`).join('')}
<tr class='nohover'><td colspan='5'>&nbsp;</td></tr>
<tr class='nohover'><th class='left'>Tag</th><th class='left'>Download</th><th class='left'>Author</th><th class='left' colspan='2'>Age</th></tr>
${refs.tags.length ? refs.tags.map(t => `
<tr><td><a href='tag/?h=${escapeHtml(t.name)}'>${escapeHtml(t.name)}</a></td><td><a href='snapshot/${escapeHtml(t.name)}.tar.gz'>${escapeHtml(t.name)}.tar.gz</a></td><td>${escapeHtml(t.author)}</td><td colspan='2'><span class='age-days' data-ut='${Math.floor(Date.now()/1000)}'>${t.age}</span></td></tr>
`).join('') : `<tr><td colspan='5'>No tags</td></tr>`}
<tr class='nohover'><td colspan='5'>&nbsp;</td></tr>
<tr class='nohover'><th class='left'>Age</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left'>Files</th><th class='left'>Lines</th></tr>
${logs.slice(0, 10).map(c => `
<tr><td><span title='${c.date}'>${c.date}</span></td><td><a href='commit/?id=${c.hash}'>${escapeHtml(c.subject)}</a><span class='decoration'><a class='branch-deco' href='log/'>master</a></span></td><td>${getGravatar(c.email)}${escapeHtml(c.author)}</td><td>1</td><td><span class='insertions'>+10</span>/<span class='deletions'>-2</span></td></tr>
`).join('')}
<tr class='nohover'><td colspan='5'><a href='log/'>[...]</a></td></tr>
<tr class='nohover'><td colspan='5'>&nbsp;</td></tr>
<tr class='nohover'><th class='left' colspan='5'>Clone</th></tr>
<tr><td colspan='5'><a rel='vcs-git' href='https://github.com/${repoInfo.owner}/${repoInfo.repoName}.git' title='${escapeHtml(repoInfo.repoName)} Git repository'>https://github.com/${repoInfo.owner}/${repoInfo.repoName}.git</a></td></tr>
</table>
`;

  writePage('index.html', renderLayout({
    title: 'summary',
    activeTab: 'summary',
    repoInfo,
    content: branchesHtml,
    rootPath: ''
  }));

  // 3. Refs View (refs/index.html)
  const refsContent = `
<table class='list nowrap'>
<tr class='nohover'><th class='left' colspan='4'>Branches</th></tr>
<tr><th class='left'>Branch</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left'>Age</th></tr>
${refs.branches.map(b => `
<tr><td><a href='../tree/?h=${escapeHtml(b.name)}'>${escapeHtml(b.name)}</a></td><td><a href='../commit/?id=${b.commit}'>${escapeHtml(b.subject)}</a></td><td>${escapeHtml(b.author)}</td><td>${b.age}</td></tr>
`).join('')}
<tr class='nohover'><td colspan='4'>&nbsp;</td></tr>
<tr class='nohover'><th class='left' colspan='4'>Tags</th></tr>
<tr><th class='left'>Tag</th><th class='left'>Target</th><th class='left'>Author</th><th class='left'>Age</th></tr>
${refs.tags.length ? refs.tags.map(t => `
<tr><td><a href='../tag/?h=${escapeHtml(t.name)}'>${escapeHtml(t.name)}</a></td><td><a href='../commit/?id=${t.commit}'>${t.commit}</a></td><td>${escapeHtml(t.author)}</td><td>${t.age}</td></tr>
`).join('') : `<tr><td colspan='4'>No tags</td></tr>`}
</table>
`;
  writePage('refs/index.html', renderLayout({
    title: 'refs',
    activeTab: 'refs',
    repoInfo,
    content: refsContent,
    rootPath: '../'
  }));

  // 4. Log View (log/index.html)
  const logContent = `
<table class='list nowrap'>
<tr class='nohover'><th class='left'>Age</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left'>Files</th><th class='left'>Lines</th></tr>
${logs.map(c => `
<tr><td><span title='${c.date}'>${c.date}</span></td><td><a href='../commit/?id=${c.hash}'>${escapeHtml(c.subject)}</a></td><td>${getGravatar(c.email)}${escapeHtml(c.author)}</td><td>1</td><td><span class='insertions'>+10</span>/<span class='deletions'>-2</span></td></tr>
`).join('')}
</table>
`;
  writePage('log/index.html', renderLayout({
    title: 'log',
    activeTab: 'log',
    repoInfo,
    content: logContent,
    rootPath: '../'
  }));

  // 5. Commit & Diff Views
  for (const c of logs) {
    const details = getCommitDetails(c.hash);
    const colored = colorizeDiff(details.fullText);

    const commitHtml = `
<div class='cgit-commit'>
<table class='commit-info'>
<tr><th>author</th><td>${escapeHtml(c.author)} &lt;${escapeHtml(c.email)}&gt;</td><td>${c.date}</td></tr>
<tr><th>committer</th><td>${escapeHtml(c.author)} &lt;${escapeHtml(c.email)}&gt;</td><td>${c.date}</td></tr>
<tr><th>commit</th><td colspan='2'>${c.hash}</td></tr>
<tr><th>download</th><td colspan='2'><a href='../patch/${c.hash}.patch'>${c.hash}.patch</a></td></tr>
</table>
<div class='commit-subject'>${escapeHtml(c.subject)}</div>
<div class='diffstat'>${escapeHtml(details.stat)}</div>
<table class='diff'>
<tr><td>${colored}</td></tr>
</table>
</div>
`;

    writePage(`commit/${c.hash}.html`, renderLayout({
      title: `commit: ${c.shortHash}`,
      activeTab: 'commit',
      repoInfo,
      content: commitHtml,
      rootPath: '../'
    }));

    writePage(`diff/${c.hash}.html`, renderLayout({
      title: `diff: ${c.shortHash}`,
      activeTab: 'diff',
      repoInfo,
      content: commitHtml,
      rootPath: '../'
    }));

    writePage(`patch/${c.hash}.patch`, details.fullText);
  }

  // Fallback for /commit/index.html & /diff/index.html pointing to HEAD
  const headCommit = logs[0] ? logs[0].hash : '';
  if (headCommit) {
    const latestDetails = getCommitDetails(headCommit);
    const latestColored = colorizeDiff(latestDetails.fullText);
    const headCommitHtml = `
<div class='cgit-commit'>
<table class='commit-info'>
<tr><th>author</th><td>${escapeHtml(logs[0].author)}</td><td>${logs[0].date}</td></tr>
<tr><th>commit</th><td colspan='2'>${headCommit}</td></tr>
</table>
<div class='commit-subject'>${escapeHtml(logs[0].subject)}</div>
<div class='diffstat'>${escapeHtml(latestDetails.stat)}</div>
<table class='diff'><tr><td>${latestColored}</td></tr></table>
</div>
`;
    writePage('commit/index.html', renderLayout({
      title: 'commit',
      activeTab: 'commit',
      repoInfo,
      content: headCommitHtml,
      rootPath: '../'
    }));
    writePage('diff/index.html', renderLayout({
      title: 'diff',
      activeTab: 'diff',
      repoInfo,
      content: headCommitHtml,
      rootPath: '../'
    }));
  }

  // 6. Tree View
  const treeEntries = getTree('HEAD');
  const treeTable = `
<div class='path'>path: <a href='index.html'>root</a></div>
<table class='list nowrap'>
<tr class='nohover'><th class='left'>Mode</th><th class='left'>Name</th><th class='right'>Size</th><th class='left'>Links</th></tr>
${treeEntries.map(e => {
  const isDir = e.mode === '040000';
  const blobUrl = `../blob/${e.sha}.html`;
  const blameUrl = `../blame/${encodeURIComponent(e.name)}.html`;
  return `
<tr>
<td class='ls-mode'>${e.mode}</td>
<td class='ls-dir'><a class='ls-name' href='${isDir ? '#' : blobUrl}'>${escapeHtml(e.name)}</a></td>
<td class='ls-size right'>${e.size}</td>
<td class='ls-mod'>${isDir ? 'tree' : `<a href='${blobUrl}'>blob</a> | <a href='${blameUrl}'>blame</a> | <a href='../blob/${e.sha}.raw'>raw</a>`}</td>
</tr>
`;
}).join('')}
</table>
`;

  writePage('tree/index.html', renderLayout({
    title: 'tree',
    activeTab: 'tree',
    repoInfo,
    content: treeTable,
    rootPath: '../'
  }));

  // Blobs & Blame
  for (const e of treeEntries) {
    if (e.mode !== '040000') {
      const blobContent = getBlob(e.sha);
      const lines = blobContent.split('\n');
      const numberedLines = lines.map((l, idx) => `<tr><td class='linenumbers'><a id='n${idx+1}' href='#n${idx+1}'>${idx+1}</a></td><td class='lines'><pre><code>${escapeHtml(l)}</code></pre></td></tr>`).join('');

      const blobHtml = `
<div class='path'>blob: <a href='../tree/index.html'>root</a>/${escapeHtml(e.name)} (<a href='${e.sha}.raw'>raw</a>)</div>
<table class='blob'>
${numberedLines}
</table>
`;

      writePage(`blob/${e.sha}.html`, renderLayout({
        title: `blob: ${e.name}`,
        activeTab: 'tree',
        repoInfo,
        content: blobHtml,
        rootPath: '../'
      }));

      writePage(`blob/${e.sha}.raw`, blobContent);

      const blameRaw = getBlame(e.name, 'HEAD');
      const blameLines = blameRaw.split('\n').filter(Boolean);
      const blameHtml = `
<div class='path'>blame: <a href='../tree/index.html'>root</a>/${escapeHtml(e.name)}</div>
<table class='blame'>
${blameLines.map((line, idx) => {
  const parts = line.split(/\s+/);
  const commit = parts[0];
  const rest = parts.slice(1).join(' ');
  return `<tr><td class='commit'><a href='../commit/${commit}.html'>${commit}</a></td><td class='linenumbers'>${idx+1}</td><td class='lines'><pre>${escapeHtml(rest)}</pre></td></tr>`;
}).join('')}
</table>
`;

      writePage(`blame/${encodeURIComponent(e.name)}.html`, renderLayout({
        title: `blame: ${e.name}`,
        activeTab: 'tree',
        repoInfo,
        content: blameHtml,
        rootPath: '../'
      }));
    }
  }

  // 7. Stats View
  const statsHtml = `
<table class='list nowrap'>
<tr class='nohover'><th class='left' colspan='2'>Repository Statistics</th></tr>
<tr><td><b>Total Commits</b></td><td>${stats.totalCommits}</td></tr>
</table>
<br/>
<table class='list nowrap'>
<tr class='nohover'><th class='left'>Author</th><th class='right'>Commits</th></tr>
${stats.authors.map(a => `
<tr><td>${escapeHtml(a.name)}</td><td class='right'>${a.count}</td></tr>
`).join('')}
</table>
`;

  writePage('stats/index.html', renderLayout({
    title: 'stats',
    activeTab: 'stats',
    repoInfo,
    content: statsHtml,
    rootPath: '../'
  }));

  // 8. Atom Feed
  const atomFeed = `<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>${escapeHtml(repoInfo.repoName)}</title>
<subtitle>${escapeHtml(repoInfo.desc)}</subtitle>
<id>urn:github:${repoInfo.owner}:${repoInfo.repoName}</id>
<updated>${new Date().toISOString()}</updated>
${logs.slice(0, 20).map(c => `
<entry>
<title>${escapeHtml(c.subject)}</title>
<id>urn:git:${c.hash}</id>
<updated>${new Date(c.date).toISOString()}</updated>
<author><name>${escapeHtml(c.author)}</name></author>
<content type="text">${escapeHtml(c.subject)}</content>
</entry>`).join('')}
</feed>`;

  writePage('atom.xml', atomFeed);

  console.log('Build complete with authentic cgit DOM and Libravatars!');
}

build().catch(err => {
  console.error(err);
  process.exit(1);
});
