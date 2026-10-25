// Karte beginnt unter KI-Hinweis und Kopfleiste der Website.
const kopfhoehe = () => document.documentElement.style.setProperty('--kopfhoehe', document.querySelector('.sk-kopf').getBoundingClientRect().bottom + 'px');
kopfhoehe(); addEventListener('resize', kopfhoehe);
// Kräfteatlas: eigene Seite; #e=<id> öffnet den Eintrag dort.
const atlas = {kennt: id => !!window.KRAEFTEATLAS_INDEX?.ids?.includes(id), zeige: id => { location.href = 'Kraefteatlas.html' + (id ? '#e=' + encodeURIComponent(id) : ''); }};
TechbaumStart({wurzel: document, basis: "web-client/", atlas});
