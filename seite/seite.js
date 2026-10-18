/* Sternenepoche-Website 0.4: Kopf-Film der Weltenkrümmer, Bildfenster, Inhaltsleiste mit Suche. Ohne Abhängigkeiten. */
(() => {
  "use strict";
  const ruhig = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const sparen = !!(navigator.connection && navigator.connection.saveData);

  /* ---------- Kopf-Film: freigestellt (WebM mit Alpha) vor dem Planeten; Safari/iOS: MP4 mit Aufhellen-Mischung ---------- */
  const film = document.querySelector("[data-kopffilm]");
  if (film) {
    const hud = document.querySelector(".buehne-hud");
    const marken = JSON.parse(film.dataset.marken || "[]");
    const setzeVolk = (m) => {
      if (!m) return;
      document.querySelector(".buehne").style.setProperty("--volk", m.farbe);
      if (hud) {
        hud.querySelector("b").textContent = "Weltenkrümmer der " + m.name;
        hud.querySelectorAll(".balken i").forEach((i, n) => i.classList.toggle("an", marken[n] === m));
      }
    };
    setzeVolk(marken[0]);
    const standbild = () => { film.querySelector("img")?.removeAttribute("hidden"); film.classList.add("an"); };
    const knopf = hud?.querySelector("button");
    let v = null;
    const starte = (sofort) => {
      const apple = /Apple/.test(navigator.vendor || "");
      const klein = Math.min(innerWidth, screen.width || innerWidth) < 820;
      v = document.createElement("video");
      Object.assign(v, { muted: true, loop: true, playsInline: true, autoplay: true, preload: "auto" });
      v.setAttribute("muted", ""); v.setAttribute("playsinline", ""); v.setAttribute("aria-hidden", "true");
      v.poster = film.dataset.standbild;
      const basis = film.dataset.quelle + (klein ? "-klein" : "");
      if (apple) { film.classList.add("mischen"); v.src = basis + ".mp4"; }
      else { v.src = basis + ".webm"; v.addEventListener("error", () => { film.classList.add("mischen"); v.src = basis + ".mp4"; }, { once: true }); }
      v.addEventListener("playing", () => { film.classList.add("an"); film.querySelector("img")?.setAttribute("hidden", ""); }, { once: true });
      v.addEventListener("timeupdate", () => setzeVolk(marken.find((m) => v.currentTime >= m.von && v.currentTime < m.bis)));
      const start = () => { film.prepend(v); v.play().catch(standbild); };
      // Automatisch erst nach dem Laden der Seite, damit Text und Bilder Vorrang haben.
      if (sofort) start();
      else if (document.readyState === "complete") setTimeout(start, 200);
      else addEventListener("load", () => setTimeout(start, 200), { once: true });
      // Nur abspielen, wenn sichtbar und nicht angehalten.
      new IntersectionObserver(([e]) => { if (!v.isConnected || knopf?.dataset.aus) return; e.isIntersecting ? v.play().catch(() => {}) : v.pause(); }).observe(film);
    };
    knopf?.addEventListener("click", () => {
      if (!v) { starte(true); knopf.textContent = "Film anhalten"; return; }
      if (v.paused) { delete knopf.dataset.aus; v.play().catch(() => {}); knopf.textContent = "Film anhalten"; }
      else { knopf.dataset.aus = "1"; v.pause(); knopf.textContent = "Film abspielen"; }
    });
    // „Bewegung reduzieren“ oder Datensparen: Standbild, der Film startet nur auf ausdrücklichen Klick.
    if (ruhig || sparen) { standbild(); if (knopf) knopf.textContent = "Film abspielen"; }
    else starte(false);
  }

  /* ---------- Bildfenster für Bildschirmfotos ---------- */
  let fenster = null;
  document.addEventListener("click", (e) => {
    const k = e.target.closest("[data-gross]");
    if (!k) return;
    if (!fenster) {
      fenster = document.createElement("dialog");
      fenster.className = "bildfenster";
      fenster.innerHTML = '<header><span></span><button type="button" aria-label="Schließen">✕</button></header><img alt="">';
      fenster.querySelector("button").addEventListener("click", () => fenster.close());
      fenster.addEventListener("click", (ev) => { if (ev.target === fenster) fenster.close(); });
      document.body.append(fenster);
    }
    fenster.querySelector("img").src = k.dataset.gross;
    fenster.querySelector("img").alt = k.querySelector("img")?.alt || "";
    fenster.querySelector("span").textContent = k.dataset.titel || "";
    fenster.showModal();
  });

  /* ---------- Inhaltsleiste: aktiver Abschnitt und Suche ---------- */
  const leiste = document.querySelector(".leiste");
  if (leiste) {
    const links = [...leiste.querySelectorAll("a[href^='#']")];
    const ziele = links.map((a) => document.getElementById(decodeURIComponent(a.hash.slice(1)))).filter(Boolean);
    const beob = new IntersectionObserver((eintraege) => {
      for (const e of eintraege) if (e.isIntersecting) {
        links.forEach((a) => a.classList.toggle("aktiv", a.hash.slice(1) === e.target.id));
      }
    }, { rootMargin: "-90px 0px -70% 0px" });
    ziele.forEach((z) => beob.observe(z));
    if (innerWidth <= 900) leiste.querySelectorAll("a").forEach((a) => a.addEventListener("click", () => leiste.querySelector("details")?.removeAttribute("open")));
    const suche = leiste.querySelector("input[type=search]");
    const bloecke = [...document.querySelectorAll("[data-suche]")];
    const status = document.getElementById("suche-status");
    if (suche) suche.addEventListener("input", () => {
      const q = suche.value.trim().toLocaleLowerCase("de");
      let n = 0;
      for (const b of bloecke) {
        const treffer = !q || b.textContent.toLocaleLowerCase("de").includes(q);
        b.hidden = !treffer; if (treffer) n++;
        if (q && treffer) b.querySelectorAll("details").forEach((d) => (d.open = true));
      }
      if (status) status.textContent = q ? `${n} Abschnitte enthalten „${suche.value.trim()}“` : "";
    });
  }
})();
