// Fee preview on the new and edit order forms.
(function () {
  const form = document.querySelector('form[data-quote-url]');
  if (!form) return;
  const output = form.querySelector('#fee-estimate');
  const button = document.getElementById('check-fee');

  async function showQuote() {
    const params = new URLSearchParams();
    for (const name of ['pickup_area_id', 'dropoff_area_id', 'distance_km']) {
      params.set(name, form.elements[name].value);
    }
    if (form.elements.express.checked) params.set('express', 'on');
    output.textContent = 'Checking…';
    try {
      const res = await fetch(`${form.dataset.quoteUrl}?${params}`, { headers: { Accept: 'application/json' } });
      const data = await res.json();
      output.textContent = data.ok
        ? `${data.total} (${data.zone}, ${data.band}${data.bridge ? ', bridge crossing' : ''})`
        : data.error;
      output.classList.toggle('muted', !data.ok);
    } catch {
      output.textContent = 'Could not check the fee. Try again.';
    }
  }

  button?.addEventListener('click', showQuote);
})();
