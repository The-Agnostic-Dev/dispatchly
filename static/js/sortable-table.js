// Click a heading in a table marked data-sortable to sort by that column; click again to reverse.
document.querySelectorAll('table[data-sortable]').forEach((table) => {
  const headers = [...table.querySelectorAll('thead th')];
  headers.forEach((th, index) => {
    if (!th.hasAttribute('data-sort')) return;
    th.classList.add('sortable');
    th.addEventListener('click', () => {
      const dir = th.dataset.dir === 'asc' ? 'desc' : 'asc';
      headers.forEach((h) => delete h.dataset.dir);
      th.dataset.dir = dir;
      const body = table.tBodies[0];
      const rows = [...body.rows];
      rows.sort((a, b) => {
        const x = a.cells[index].textContent.trim();
        const y = b.cells[index].textContent.trim();
        return dir === 'asc' ? x.localeCompare(y) : y.localeCompare(x);
      });
      rows.forEach((row) => body.appendChild(row));
    });
  });
});
