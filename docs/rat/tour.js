/* Cockpit-Seite: die Schiene markiert den Abschnitt, der gerade gelesen wird (wie die Schiene im Cockpit). */
(() => {
  "use strict";
  const links = [...document.querySelectorAll(".tour-schiene a[href^='#']")];
  const paare = links.map((a) => [document.getElementById(a.getAttribute("href").slice(1)), a]).filter(([el]) => el);
  if (!paare.length) return;
  let geplant = false;
  const markieren = () => {
    geplant = false;
    const linie = Math.min(260, window.innerHeight * 0.35);
    let aktiv = null;
    for (const [el] of paare) if (el.getBoundingClientRect().top <= linie) aktiv = el;
    for (const [el, a] of paare) a.setAttribute("aria-current", el === aktiv ? "true" : "false");
  };
  window.addEventListener("scroll", () => { if (!geplant) { geplant = true; requestAnimationFrame(markieren); } }, { passive: true });
  markieren();
})();
