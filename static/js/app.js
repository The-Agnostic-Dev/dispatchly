// Shared behaviour for every page.

// Stop a second click on a form's button while the first submit is still being sent.
document.addEventListener('submit', (event) => {
  if (event.defaultPrevented) return;
  const button = event.target.querySelector('button[type="submit"]');
  if (button) setTimeout(() => (button.disabled = true), 0);
});
