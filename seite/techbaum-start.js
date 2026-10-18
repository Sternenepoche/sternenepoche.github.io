// Karte beginnt unter KI-Hinweis und Kopfleiste der Website.
const kopfhoehe = () => document.documentElement.style.setProperty('--kopfhoehe', document.querySelector('.sk-kopf').getBoundingClientRect().bottom + 'px');
kopfhoehe(); addEventListener('resize', kopfhoehe);
TechbaumStart({wurzel: document, basis: "web-client/"});
