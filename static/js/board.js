// Keep the board fresh on the dispatch screen: reload every minute unless someone is typing.
(function () {
  const REFRESH_MS = 60_000;
  const search = document.querySelector('.search input[name="q"]');
  setInterval(() => {
    if (document.hidden) return;
    if (search && document.activeElement === search) return;
    window.location.reload();
  }, REFRESH_MS);
})();
