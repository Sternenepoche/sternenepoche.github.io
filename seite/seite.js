/* Sternenepoche-Website 0.4: Kopf-Film der Weltenkrümmer, Bildfenster, Inhaltsleiste mit Suche. Ohne Abhängigkeiten. */
(() => {
  "use strict";
  const ruhig = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const sparen = !!(navigator.connection && navigator.connection.saveData);

  /* ---------- Kopf im Wechsel: ein Weltenkrümmer schießt (freigestellt vor dem Planeten), dann der Spielfilm, dann der
     nächste Weltenkrümmer, wieder der Film … Weltenkrümmer: WebM mit Alpha, Safari/iOS MP4 mit Aufhellen-Mischung. ---------- */
  const film = document.querySelector("[data-kopffilm]");
  if (film) {
    const buehne = document.querySelector(".buehne");
    const kino = document.querySelector("[data-kinofilm]");
    const hud = document.querySelector(".buehne-hud");
    const titel = hud?.querySelector("b");
    const marken = JSON.parse(film.dataset.marken || "[]");
    const setzeVolk = (m) => {
      if (!m) return;
      buehne.style.setProperty("--volk", m.farbe);
      if (titel) titel.textContent = "Weltenkrümmer der " + m.name;
      hud?.querySelectorAll(".balken i").forEach((i, n) => i.classList.toggle("an", marken[n] === m));
    };
    setzeVolk(marken[0]);
    const standbild = () => { film.querySelector("img")?.removeAttribute("hidden"); film.classList.add("an"); };
    const knopf = hud?.querySelector("button");
    const klein = Math.min(innerWidth, screen.width || innerWidth) < 820;
    let wk = null, kv = null, aktiv = null, nr = 0, sichtbar = true;
    const spiele = (v) => { aktiv = v; if (sichtbar && !knopf?.dataset.aus) v.play().catch(() => {}); };
    const imKino = (an) => {
      buehne.classList.toggle("im-kino", an);
      hud?.classList.toggle("kino", an);
      if (an && titel) titel.textContent = kino.dataset.titel;
    };
    const kinoVideo = () => {
      if (kv || !kino) return kv;
      kv = document.createElement("video");
      Object.assign(kv, { muted: true, playsInline: true, preload: "auto" });
      kv.setAttribute("muted", ""); kv.setAttribute("playsinline", ""); kv.setAttribute("aria-hidden", "true");
      kv.poster = kino.dataset.standbild;
      kv.src = kino.dataset.quelle + (klein ? "-klein" : "") + ".mp4";
      kv.addEventListener("ended", () => wkTeil(nr + 1));
      kino.prepend(kv);
      return kv;
    };
    // Ein Volk: von seiner Marke bis zur nächsten; dort ist der Weltenkrümmer schon ausgeblendet.
    const wkTeil = (n) => {
      nr = n % marken.length;
      imKino(false);
      setzeVolk(marken[nr]);
      wk.currentTime = marken[nr].von;
      spiele(wk);
    };
    const insKino = () => {
      if (aktiv !== wk) return;
      wk.pause();
      const v = kinoVideo();
      if (!v) { wkTeil(nr + 1); return; }
      v.currentTime = 0;
      imKino(true);
      spiele(v);
    };
    const starte = (sofort) => {
      const apple = /Apple/.test(navigator.vendor || "");
      wk = document.createElement("video");
      Object.assign(wk, { muted: true, playsInline: true, preload: "auto" });
      wk.setAttribute("muted", ""); wk.setAttribute("playsinline", ""); wk.setAttribute("aria-hidden", "true");
      wk.poster = film.dataset.standbild;
      const basis = film.dataset.quelle + (klein ? "-klein" : "");
      if (apple) { film.classList.add("mischen"); wk.src = basis + ".mp4"; }
      else { wk.src = basis + ".webm"; wk.addEventListener("error", () => { film.classList.add("mischen"); wk.src = basis + ".mp4"; }, { once: true }); }
      wk.addEventListener("playing", () => {
        film.classList.add("an"); film.querySelector("img")?.setAttribute("hidden", "");
        kinoVideo();  // Film vorladen, solange der erste Weltenkrümmer läuft
      }, { once: true });
      wk.addEventListener("timeupdate", () => { if (wk.currentTime >= marken[nr].bis - 0.15) insKino(); });
      wk.addEventListener("ended", insKino);
      const start = () => { film.prepend(wk); aktiv = wk; wk.play().catch(standbild); };
      // Automatisch erst nach dem Laden der Seite, damit Text und Bilder Vorrang haben.
      if (sofort) start();
      else if (document.readyState === "complete") setTimeout(start, 200);
      else addEventListener("load", () => setTimeout(start, 200), { once: true });
      // Nur abspielen, wenn sichtbar und nicht angehalten.
      new IntersectionObserver(([e]) => {
        sichtbar = e.isIntersecting;
        if (!aktiv?.isConnected || knopf?.dataset.aus) return;
        sichtbar ? aktiv.play().catch(() => {}) : aktiv.pause();
      }).observe(buehne);
    };
    knopf?.addEventListener("click", () => {
      if (!wk) { starte(true); knopf.textContent = "Film anhalten"; return; }
      if (knopf.dataset.aus) { delete knopf.dataset.aus; aktiv?.play().catch(() => {}); knopf.textContent = "Film anhalten"; }
      else { knopf.dataset.aus = "1"; aktiv?.pause(); knopf.textContent = "Film abspielen"; }
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
