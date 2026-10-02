import fs from 'fs';
import path from 'path';
import {
  getRepoInfo,
  getRefs,
  getLog,
  getCommitDetails,
  getTree,
  getBlob,
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
      return `<span class="diff-add">${esc}</span>`;
    } else if (line.startsWith('-') && !line.startsWith('---')) {
      return `<span class="diff-del">${esc}</span>`;
    } else if (line.startsWith('@@')) {
      return `<span class="diff-hunk">${esc}</span>`;
    }
    return esc;
  }).join('\n');
}

async function build() {
  console.log('Generating cgit static pages...');
  const repoInfo = getRepoInfo();
  const refs = getRefs();
  const logs = getLog(100);
  const readme = getReadme();

  // 1. Summary (index.html)
  let readmeHtml = '';
  if (readme) {
    readmeHtml = `
      <h3>${readme.filename}</h3>
      <div class="readme">
        <pre>${escapeHtml(readme.content)}</pre>
      </div>
    `;
  }

  const recentCommitsHtml = `
    <h3>Recent Commits</h3>
    <table class="list">
      <tr><th>Age</th><th>Commit message</th><th>Author</th></tr>
      ${logs.slice(0, 10).map(c => `
        <tr>
          <td>${c.date}</td>
          <td><a href="commit/${c.hash}.html">${escapeHtml(c.subject)}</a></td>
          <td>${escapeHtml(c.author)}</td>
        </tr>
      `).join('')}
    </table>
  `;

  const branchesHtml = `
    <h3>Branches</h3>
    <table class="list">
      <tr><th>Branch</th><th>Commit</th><th>Age</th><th>Author</th></tr>
      ${refs.branches.map(b => `
        <tr>
          <td><a href="tree/${b.name}/index.html">${b.name}</a></td>
          <td><a href="commit/${b.commit}.html">${b.commit}</a></td>
          <td>${b.age}</td>
          <td>${escapeHtml(b.author)}</td>
        </tr>
      `).join('')}
    </table>
  `;

  const summaryContent = `
    <div>
      <p><b>Clone:</b> <code>https://github.com/${repoInfo.owner}/${repoInfo.repoName}.git</code></p>
    </div>
    ${branchesHtml}
    ${recentCommitsHtml}
    ${readmeHtml}
  `;

  writePage('index.html', renderLayout({
    title: 'Summary',
    activeTab: 'summary',
    repoInfo,
    content: summaryContent,
    rootPath: ''
  }));

  // 2. Refs (refs/index.html)
  const refsContent = `
    <h3>Branches</h3>
    <table class="list">
      <tr><th>Branch</th><th>Latest Commit</th><th>Age</th><th>Author</th></tr>
      ${refs.branches.map(b => `
        <tr>
          <td><a href="../tree/${b.name}/index.html">${b.name}</a></td>
          <td><a href="../commit/${b.commit}.html">${b.commit}</a> - ${escapeHtml(b.subject)}</td>
          <td>${b.age}</td>
          <td>${escapeHtml(b.author)}</td>
        </tr>
      `).join('')}
    </table>

    <h3>Tags</h3>
    <table class="list">
      <tr><th>Tag</th><th>Target Commit</th><th>Age</th><th>Author</th></tr>
      ${refs.tags.length ? refs.tags.map(t => `
        <tr>
          <td>${t.name}</td>
          <td><a href="../commit/${t.commit}.html">${t.commit}</a> - ${escapeHtml(t.subject)}</td>
          <td>${t.age}</td>
          <td>${escapeHtml(t.author)}</td>
        </tr>
      `).join('') : '<tr><td colspan="4">No tags found</td></tr>'}
    </table>
  `;

  writePage('refs/index.html', renderLayout({
    title: 'Refs',
    activeTab: 'refs',
    repoInfo,
    content: refsContent,
    rootPath: '../'
  }));

  // 3. Log (log/index.html)
  const logContent = `
    <h3>Commit Log (${logs.length} commits)</h3>
    <table class="list">
      <tr><th>Date</th><th>Commit Message</th><th>Author</th><th>SHA</th></tr>
      ${logs.map(c => `
        <tr>
          <td>${c.date}</td>
          <td><a href="../commit/${c.hash}.html">${escapeHtml(c.subject)}</a></td>
          <td>${escapeHtml(c.author)}</td>
          <td><code>${c.shortHash}</code></td>
        </tr>
      `).join('')}
    </table>
  `;

  writePage('log/index.html', renderLayout({
    title: 'Commit Log',
    activeTab: 'log',
    repoInfo,
    content: logContent,
    rootPath: '../'
  }));

  // 4. Commits details & patches
  for (const c of logs) {
    const details = getCommitDetails(c.hash);
    const colored = colorizeDiff(details.fullText);
    const commitHtml = `
      <h3>Commit ${c.hash}</h3>
      <p><b>Author:</b> ${escapeHtml(c.author)} &lt;${escapeHtml(c.email)}&gt;</p>
      <p><b>Date:</b> ${c.date}</p>
      <p><a href="../patch/${c.hash}.patch">View raw patch</a></p>
      <hr>
      <pre class="diff">${colored}</pre>
    `;

    writePage(`commit/${c.hash}.html`, renderLayout({
      title: `Commit ${c.shortHash}`,
      activeTab: 'log',
      repoInfo,
      content: commitHtml,
      rootPath: '../'
    }));

    writePage(`patch/${c.hash}.patch`, details.fullText);
  }

  // 5. Tree & Blobs for HEAD and branches
  const branchesToRender = refs.branches.length ? refs.branches.map(b => b.name) : ['HEAD'];
  if (!branchesToRender.includes('HEAD')) branchesToRender.push('HEAD');

  for (const branch of branchesToRender) {
    const treeEntries = getTree(branch);
    const treeTable = `
      <h3>Tree for ${branch}</h3>
      <table class="list">
        <tr><th>Mode</th><th>Name</th><th>Size</th><th>Action</th></tr>
        ${treeEntries.map(e => {
          const isDir = e.mode === '040000';
          const link = isDir 
            ? `#` 
            : `../../blob/${branch}/${e.sha}.html`;
          return `
            <tr>
              <td class="mode">${e.mode}</td>
              <td><a href="${link}" class="${isDir ? 'dir' : 'file'}">${escapeHtml(e.name)}</a></td>
              <td>${e.size}</td>
              <td>${isDir ? 'dir' : `<a href="../../blob/${branch}/${e.sha}.html">view</a> | <a href="../../blob/${branch}/${e.sha}.raw">raw</a>`}</td>
            </tr>
          `;
        }).join('')}
      </table>
    `;

    writePage(`tree/${branch}/index.html`, renderLayout({
      title: `Tree - ${branch}`,
      activeTab: 'tree',
      repoInfo,
      content: treeTable,
      rootPath: '../../'
    }));

    // Render blobs
    for (const e of treeEntries) {
      if (e.mode !== '040000') {
        const blobContent = getBlob(e.sha);
        const blobHtml = `
          <h3>Blob: ${escapeHtml(e.name)} (${e.sha.slice(0, 8)})</h3>
          <p><a href="${e.sha}.raw">Download raw</a></p>
          <pre class="blob">${escapeHtml(blobContent)}</pre>
        `;

        writePage(`blob/${branch}/${e.sha}.html`, renderLayout({
          title: `Blob - ${e.name}`,
          activeTab: 'tree',
          repoInfo,
          content: blobHtml,
          rootPath: '../../../'
        }));

        writePage(`blob/${branch}/${e.sha}.raw`, blobContent);
      }
    }
  }

  // 6. Atom feed (atom.xml)
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

  console.log('Build completed successfully!');
}

build().catch(err => {
  console.error(err);
  process.exit(1);
});
