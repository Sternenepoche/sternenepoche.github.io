(() => {
  'use strict';
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  const toggle = document.getElementById('space-toggle');
  let paused = reduced.matches;
  try { paused = reduced.matches || localStorage.getItem('sternenepoche-site-motion') === 'paused'; } catch {}
  function update() {
    document.body.dataset.motion = paused ? 'paused' : 'running';
    if (toggle) {
      toggle.hidden = false;
      toggle.textContent = paused ? '▷ Weltraumbewegung starten' : 'Ⅱ Weltraumbewegung pausieren';
      toggle.setAttribute('aria-pressed', String(paused));
    }
    document.dispatchEvent(new Event('space-motion'));
  }
  toggle?.addEventListener('click', () => {
    paused = !paused;
    try { localStorage.setItem('sternenepoche-site-motion', paused ? 'paused' : 'running'); } catch {}
    update();
  });
  reduced.addEventListener('change', e => { paused = e.matches; update(); });
  document.addEventListener('visibilitychange', () => { document.body.dataset.pageHidden = String(document.hidden); });
  document.querySelectorAll('.mobile-guide-menu a').forEach(a => a.addEventListener('click', () => { a.closest('details').open = false; }));
  update();

  const dialog = document.getElementById('image-dialog');
  if (dialog && typeof dialog.showModal === 'function') {
    document.querySelectorAll('.zoom').forEach(button => button.addEventListener('click', () => {
      const source = button.querySelector('img');
      dialog.querySelector('img').src = source.currentSrc || source.src;
      dialog.querySelector('img').alt = source.alt;
      dialog.querySelector('#image-dialog-title').textContent = source.alt;
      dialog.showModal();
      document.body.classList.add('no-scroll');
    }));
    dialog.querySelector('button').addEventListener('click', () => dialog.close());
    dialog.addEventListener('close', () => document.body.classList.remove('no-scroll'));
    dialog.addEventListener('click', event => { if (event.target === dialog) dialog.close(); });
  }
  const indexLinks = [...document.querySelectorAll('.guide-index a')];
  if (indexLinks.length && typeof IntersectionObserver === 'function') {
    const observer = new IntersectionObserver(entries => {
      const current = entries.find(entry => entry.isIntersecting);
      if (!current) return;
      const heading = current.target.querySelector('h3');
      indexLinks.forEach(link => link.setAttribute('aria-current', String(link.hash === '#' + heading.id)));
    }, {rootMargin:'-110px 0px -65% 0px',threshold:0});
    document.querySelectorAll('.guide-chapter').forEach(chapter => observer.observe(chapter));
  }
})();
