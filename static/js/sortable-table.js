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
        const cellX = a.cells[index];
        const cellY = b.cells[index];
        const x = (cellX.dataset.value || cellX.textContent).trim();
        const y = (cellY.dataset.value || cellY.textContent).trim();
        const nx = Number(x);
        const ny = Number(y);
        let cmp;
        if (x !== '' && y !== '' && !Number.isNaN(nx) && !Number.isNaN(ny)) {
          cmp = nx - ny;
        } else {
          cmp = x.localeCompare(y);
        }
        return dir === 'asc' ? cmp : -cmp;
      });
      rows.forEach((row) => body.appendChild(row));
    });
  });
});
