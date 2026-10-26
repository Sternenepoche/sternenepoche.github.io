// Kräfteatlas unter KI-Hinweis und Kopfleiste der Website; Shadow DOM, damit Seiten- und Atlasgestaltung sich nicht mischen.
const kopfhoehe = () => document.documentElement.style.setProperty('--kopfhoehe', document.querySelector('.sk-kopf').getBoundingClientRect().bottom + 'px');
kopfhoehe(); addEventListener('resize', kopfhoehe);
const wurzel = document.getElementById('atlas').attachShadow({mode: 'open'}), css = document.createElement('link');
css.rel = 'stylesheet'; css.href = "web-client/kraefteatlas/atlas.css?v=6880b21d4f";
css.onload = css.onerror = () => KraefteatlasStart({wurzel, basis: "web-client/", eingebettet: false, vollbild: true});
wurzel.append(css);
