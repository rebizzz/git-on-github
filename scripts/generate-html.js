export function renderLayout({ title, activeTab, repoInfo, content, rootPath = '' }) {
  const tabs = [
    { id: 'summary', name: 'summary', href: `${rootPath}index.html` },
    { id: 'refs', name: 'refs', href: `${rootPath}refs/index.html` },
    { id: 'log', name: 'log', href: `${rootPath}log/index.html` },
    { id: 'tree', name: 'tree', href: `${rootPath}tree/HEAD/index.html` },
    { id: 'atom', name: 'atom', href: `${rootPath}atom.xml` }
  ];

  const tabHeaders = tabs.map(t => {
    const isActive = t.id === activeTab ? 'class="active"' : '';
    return `<td ${isActive}><a href="${t.href}">${t.name}</a></td>`;
  }).join('\n');

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>${repoInfo.repoName} - ${title}</title>
  <link rel="stylesheet" type="text/css" href="${rootPath}cgit.css">
  <link rel="alternate" title="${repoInfo.repoName} Atom feed" href="${rootPath}atom.xml" type="application/atom+xml">
</head>
<body>
<div id="cgit">
  <table id="header">
    <tr>
      <td class="main"><a href="${rootPath}index.html">${repoInfo.repoName}</a></td>
      <td class="sub">${escapeHtml(repoInfo.desc)}</td>
    </tr>
  </table>
  <table class="tabs">
    <tr>
      ${tabHeaders}
    </tr>
  </table>
  <div class="content">
    ${content}
  </div>
  <footer>
    generated via git-on-github actions cgit engine | fully static GitHub Pages proof-of-concept
  </footer>
</div>
</body>
</html>`;
}

export function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
