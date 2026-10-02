import fs from 'fs';
import path from 'path';
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
  console.log('Generating exact cgit layout...');
  const repoInfo = getRepoInfo();
  const refs = getRefs();
  repoInfo.branches = refs.branches;
  const logs = getLog(100);
  const readme = getReadme();
  const stats = getStats();

  // 1. About View
  let aboutHtml = `<div class='readme-header'>${readme ? readme.filename : 'README'}</div><div class='readme-body'><pre>${readme ? escapeHtml(readme.content) : 'No README found in repository.'}</pre></div>`;
  writePage('about/index.html', renderLayout({
    title: 'about',
    activeTab: 'about',
    repoInfo,
    content: aboutHtml,
    rootPath: '../'
  }));

  // 2. Summary View (index.html)
  const branchesHtml = `
    <table class='list nowrap'>
      <tr class='nohover'><th class='left' colspan='4'>Branches</th></tr>
      <tr><th class='left'>Branch</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left'>Age</th></tr>
      ${refs.branches.map(b => `
        <tr>
          <td><a href='tree/${escapeHtml(b.name)}/index.html'>${escapeHtml(b.name)}</a></td>
          <td><a href='commit/${b.commit}.html'>${escapeHtml(b.subject)}</a></td>
          <td>${escapeHtml(b.author)}</td>
          <td>${b.age}</td>
        </tr>
      `).join('')}
    </table>
  `;

  const tagsHtml = `
    <table class='list nowrap'>
      <tr class='nohover'><th class='left' colspan='4'>Tags</th></tr>
      <tr><th class='left'>Tag</th><th class='left'>Target</th><th class='left'>Author</th><th class='left'>Age</th></tr>
      ${refs.tags.length ? refs.tags.map(t => `
        <tr>
          <td><a href='tree/${escapeHtml(t.name)}/index.html'>${escapeHtml(t.name)}</a></td>
          <td><a href='commit/${t.commit}.html'>${t.commit}</a></td>
          <td>${escapeHtml(t.author)}</td>
          <td>${t.age}</td>
        </tr>
      `).join('') : '<tr><td colspan="4">No tags</td></tr>'}
    </table>
  `;

  const recentCommitsHtml = `
    <table class='list nowrap'>
      <tr class='nohover'><th class='left' colspan='4'>Shortlog</th></tr>
      <tr><th class='left'>Age</th><th class='left'>Commit message</th><th class='left'>Author</th><th class='left'>Files</th></tr>
      ${logs.slice(0, 10).map(c => `
        <tr>
          <td>${c.date}</td>
          <td><a href='commit/${c.hash}.html'>${escapeHtml(c.subject)}</a></td>
          <td>${escapeHtml(c.author)}</td>
          <td><a href='commit/${c.hash}.html'>diff</a></td>
        </tr>
      `).join('')}
    </table>
  `;

  const cloneHtml = `
    <table class='list nowrap'>
      <tr class='nohover'><th class='left'>Clone</th></tr>
      <tr><td><code>git clone https://github.com/${repoInfo.owner}/${repoInfo.repoName}.git</code></td></tr>
    </table>
  `;

  const summaryContent = `${cloneHtml}<br/>${branchesHtml}<br/>${tagsHtml}<br/>${recentCommitsHtml}`;

  writePage('index.html', renderLayout({
    title: 'summary',
    activeTab: 'summary',
    repoInfo,
    content: summaryContent,
    rootPath: ''
  }));

  // 3. Refs View (refs/index.html)
  const refsContent = `${branchesHtml}<br/>${tagsHtml}`;
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
      <tr class='nohover'>
        <th class='left'>Age</th>
        <th class='left'>Commit message</th>
        <th class='left'>Author</th>
        <th class='left'>SHA</th>
      </tr>
      ${logs.map(c => `
        <tr>
          <td>${c.date}</td>
          <td><a href='../commit/${c.hash}.html'>${escapeHtml(c.subject)}</a></td>
          <td>${escapeHtml(c.author)}</td>
          <td><code>${c.shortHash}</code></td>
        </tr>
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

    // Diff tab mirror
    writePage(`diff/${c.hash}.html`, renderLayout({
      title: `diff: ${c.shortHash}`,
      activeTab: 'diff',
      repoInfo,
      content: commitHtml,
      rootPath: '../'
    }));

    writePage(`patch/${c.hash}.patch`, details.fullText);
  }

  // 6. Tree, Blobs & Blame Views
  const branchesToRender = refs.branches.length ? refs.branches.map(b => b.name) : ['main'];
  for (const branch of branchesToRender) {
    const treeEntries = getTree(branch);

    const treeTable = `
      <table class='list nowrap'>
        <tr class='nohover'>
          <th class='left'>Mode</th>
          <th class='left'>Name</th>
          <th class='right'>Size</th>
          <th class='left'>Links</th>
        </tr>
        ${treeEntries.map(e => {
          const isDir = e.mode === '040000';
          const blobUrl = `../../blob/${branch}/${e.sha}.html`;
          const blameUrl = `../../blame/${branch}/${encodeURIComponent(e.name)}.html`;
          return `
            <tr>
              <td class='ls-mode'>${e.mode}</td>
              <td class='ls-dir'>
                <a class='ls-name' href='${isDir ? '#' : blobUrl}'>${escapeHtml(e.name)}</a>
              </td>
              <td class='ls-size right'>${e.size}</td>
              <td class='ls-mod'>
                ${isDir ? 'tree' : `<a href='${blobUrl}'>blob</a> | <a href='${blameUrl}'>blame</a> | <a href='../../blob/${branch}/${e.sha}.raw'>raw</a>`}
              </td>
            </tr>
          `;
        }).join('')}
      </table>
    `;

    writePage(`tree/${branch}/index.html`, renderLayout({
      title: `tree: ${branch}`,
      activeTab: 'tree',
      repoInfo,
      content: treeTable,
      rootPath: '../../',
      currentRef: branch
    }));

    // Blobs & Blame
    for (const e of treeEntries) {
      if (e.mode !== '040000') {
        const blobContent = getBlob(e.sha);
        const lines = blobContent.split('\n');
        const numberedLines = lines.map((l, idx) => `<tr><td class='linenumbers'><a id='n${idx+1}' href='#n${idx+1}'>${idx+1}</a></td><td class='lines'><pre><code>${escapeHtml(l)}</code></pre></td></tr>`).join('');

        const blobHtml = `
          <div class='path'>blob: ${escapeHtml(e.name)} (<a href='${e.sha}.raw'>raw</a>)</div>
          <table class='blob'>
            ${numberedLines}
          </table>
        `;

        writePage(`blob/${branch}/${e.sha}.html`, renderLayout({
          title: `blob: ${e.name}`,
          activeTab: 'tree',
          repoInfo,
          content: blobHtml,
          rootPath: '../../../',
          currentRef: branch
        }));

        writePage(`blob/${branch}/${e.sha}.raw`, blobContent);

        // Blame
        const blameRaw = getBlame(e.name, branch);
        const blameLines = blameRaw.split('\n').filter(Boolean);
        const blameHtml = `
          <div class='path'>blame: ${escapeHtml(e.name)}</div>
          <table class='blame'>
            ${blameLines.map((line, idx) => {
              const parts = line.split(/\s+/);
              const commit = parts[0];
              const rest = parts.slice(1).join(' ');
              return `<tr><td class='commit'><a href='../../../commit/${commit}.html'>${commit}</a></td><td class='linenumbers'>${idx+1}</td><td class='lines'><pre>${escapeHtml(rest)}</pre></td></tr>`;
            }).join('')}
          </table>
        `;

        writePage(`blame/${branch}/${encodeURIComponent(e.name)}.html`, renderLayout({
          title: `blame: ${e.name}`,
          activeTab: 'tree',
          repoInfo,
          content: blameHtml,
          rootPath: '../../../',
          currentRef: branch
        }));
      }
    }
  }

  // 7. Stats View (stats/index.html)
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

  console.log('Build completed with authentic cgit views!');
}

build().catch(err => {
  console.error(err);
  process.exit(1);
});
