export function renderLayout({ title, activeTab, repoInfo, content, rootPath = '', currentRef = 'main' }) {
  const tabs = [
    { id: 'about', name: 'about', href: `${rootPath}about/` },
    { id: 'summary', name: 'summary', href: `${rootPath}` },
    { id: 'refs', name: 'refs', href: `${rootPath}refs/` },
    { id: 'log', name: 'log', href: `${rootPath}log/` },
    { id: 'tree', name: 'tree', href: `${rootPath}tree/` },
    { id: 'commit', name: 'commit', href: `${rootPath}commit/` },
    { id: 'diff', name: 'diff', href: `${rootPath}diff/` },
    { id: 'stats', name: 'stats', href: `${rootPath}stats/` }
  ];

  const tabLinks = tabs.map(t => {
    const activeClass = t.id === activeTab ? "class='active' " : '';
    return `<a ${activeClass}href='${t.href}'>${t.name}</a>`;
  }).join('');

  return `<!DOCTYPE html>
<html lang='en'>
<head>
<title>${escapeHtml(repoInfo.repoName)} - ${escapeHtml(title)}</title>
<meta name='generator' content='cgit v1.3-18-gd494'/>
<meta name='robots' content='index, nofollow'/>
<link rel='stylesheet' type='text/css' href='${rootPath}cgit.css'/>
<script type='text/javascript' src='${rootPath}cgit.js'></script>
<link rel='shortcut icon' href='${rootPath}favicon.ico'/>
<link rel='alternate' title='Atom feed' href='${rootPath}atom.xml' type='application/atom+xml'/>
<link rel='vcs-git' href='https://github.com/${repoInfo.owner}/${repoInfo.repoName}.git' title='${escapeHtml(repoInfo.repoName)} Git repository'/>
</head>
<body>
<div id='cgit'><table id='header'>
<tr>
<td class='logo' rowspan='2'><a href='${rootPath}'><img src='https://git.zx2c4.com/cgit.png' alt='cgit logo'/></a></td>
<td class='main'><a href='${rootPath}'>index</a> : <a href='${rootPath}'>${escapeHtml(repoInfo.repoName)}</a></td><td class='form'><form method='get'>
<select name='h' onchange='window.location.href="${rootPath}tree/?h=" + this.value;'>
${(repoInfo.branches || []).map(b => `<option value='${escapeHtml(b.name)}' ${b.name === currentRef ? "selected='selected'" : ''}>${escapeHtml(b.name)}</option>`).join('\n')}
</select> <input type='submit' value='switch'/></form></td></tr>
<tr><td class='sub'>${escapeHtml(repoInfo.desc)}</td><td class='sub right'>${escapeHtml(repoInfo.owner)}</td></tr></table>
<table class='tabs'><tr><td>
${tabLinks}</td><td class='form'><form class='right' method='get' action='${rootPath}log/'>
<select name='qt'>
<option value='grep'>log msg</option>
<option value='author'>author</option>
<option value='committer'>committer</option>
<option value='range'>range</option>
</select>
<input class='txt' type='search' size='10' name='q' value=''/>
<input type='submit' value='search'/>
</form>
</td></tr></table>
<div class='content'>${content}</div> <!-- class=content -->
<div class='footer'>Copyright &copy; 1996 &ndash; 2026 Jason A. Donenfeld. All Rights Reverse Engineered. Powered by GitHub Actions.</div>
</div> <!-- id=cgit -->
</body>
</html>
`;
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
