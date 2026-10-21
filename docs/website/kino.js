/* Trailer-Seite: eigener Spieler mit Szenenmarken, Ambilicht aus dem laufenden Bild und Sternenfeld. Ohne Abhängigkeiten.
   „Bewegung reduzieren“ und Datensparen: kein Autostart, kein bewegter Hintergrund. */
(() => {
  "use strict";
  const schirm = document.querySelector("[data-kino]");
  if (!schirm) return;
  const ruhig = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const sparen = !!(navigator.connection && navigator.connection.saveData);
  const koerper = document.body;
  const v = schirm.querySelector("video");
  const quellen = JSON.parse(schirm.dataset.quellen);
  const szenen = JSON.parse(schirm.dataset.szenen);
  let dauer = Number(schirm.dataset.dauer) || 82;
  const zeitleiste = schirm.querySelector(".kino-zeitleiste");
  const voll = zeitleiste.querySelector(".voll"), puffer = zeitleiste.querySelector(".puffer"), marke = zeitleiste.querySelector(".kino-marke");
  const zeitText = schirm.querySelector(".kino-zeit"), hudZeit = schirm.querySelector("[data-zeit]"), hudSzene = schirm.querySelector("[data-szene]");
  const knopfSpiel = schirm.querySelector("[data-k=spiel]"), ende = schirm.querySelector(".kino-ende");
  const karten = [...document.querySelectorAll(".kino-szenen li")];
  const SPIEL = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4.5v15l12-7.5z"/></svg>';
  const PAUSE = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 4h4.5v16H6zM13.5 4H18v16h-4.5z"/></svg>';

  // Qualität nach Bildschirm: 1080p nur, wenn das Bild so groß dargestellt wird und nicht gespart werden soll.
  const gross = Math.min(innerWidth, screen.width || innerWidth) * (devicePixelRatio || 1) >= 1300 && !sparen;
  v.src = gross ? quellen["1080"] : quellen["720"];
  const qual = schirm.querySelector("[data-qual]");
  if (qual) qual.textContent = gross ? "1080p" : "720p";

  const mmss = (s) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;
  const szeneBei = (t) => szenen.reduce((a, s, i) => (t + 0.05 >= s.t ? i : a), 0);
  let aktiv = -1;
  const zeichne = () => {
    const t = v.currentTime || 0;
    voll.style.width = (100 * t / dauer) + "%";
    zeitText.textContent = `${mmss(t)} / ${mmss(dauer)}`;
    if (hudZeit) hudZeit.textContent = mmss(t);
    zeitleiste.setAttribute("aria-valuenow", Math.round(t));
    zeitleiste.setAttribute("aria-valuetext", `${mmss(t)} – ${szenen[szeneBei(t)].titel}`);
    const i = szeneBei(t);
    if (i !== aktiv) {
      aktiv = i;
      karten.forEach((k, n) => k.classList.toggle("an", n === i));
      if (hudSzene) hudSzene.textContent = szenen[i].titel;
    }
  };
  v.addEventListener("loadedmetadata", () => { if (isFinite(v.duration)) dauer = v.duration; zeichne(); });
  v.addEventListener("timeupdate", zeichne);
  v.addEventListener("progress", () => { if (v.buffered.length) puffer.style.width = (100 * v.buffered.end(v.buffered.length - 1) / dauer) + "%"; });

  const spielen = () => { ende.classList.remove("an"); return v.play().catch(() => {}); };
  const zustand = () => {
    const laeuft = !v.paused && !v.ended;
    schirm.classList.toggle("kino-laeuft", laeuft || v.currentTime > 0.1);
    koerper.classList.toggle("kino-spielt", laeuft);
    knopfSpiel.innerHTML = laeuft ? PAUSE : SPIEL;
    knopfSpiel.setAttribute("aria-label", laeuft ? "Anhalten" : "Abspielen");
    if (laeuft) ruheBald(); else wecken();
  };
  ["play", "pause", "ended"].forEach((e) => v.addEventListener(e, zustand));
  v.addEventListener("ended", () => ende.classList.add("an"));
  schirm.querySelector(".kino-start").addEventListener("click", () => { spielen(); schirm.focus({ preventScroll: true }); });
  knopfSpiel.addEventListener("click", () => (v.paused ? spielen() : v.pause()));
  v.addEventListener("click", () => (v.paused ? spielen() : v.pause()));
  ende.querySelector("[data-k=nochmal]")?.addEventListener("click", () => { v.currentTime = 0; spielen(); });
  const springe = (t) => { v.currentTime = Math.max(0, Math.min(dauer - 0.05, t)); zeichne(); };
  karten.forEach((k, i) => k.querySelector("button").addEventListener("click", () => {
    springe(szenen[i].t); spielen();
    schirm.scrollIntoView({ behavior: ruhig ? "auto" : "smooth", block: "center" });
  }));

  // Zeitleiste: Klicken, Ziehen, Pfeiltasten; Marke zeigt die Szene unter dem Zeiger.
  const anteil = (e) => { const r = zeitleiste.getBoundingClientRect(); return Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)); };
  let ziehen = false;
  zeitleiste.addEventListener("pointerdown", (e) => { ziehen = true; zeitleiste.setPointerCapture(e.pointerId); springe(anteil(e) * dauer); });
  zeitleiste.addEventListener("pointermove", (e) => {
    const a = anteil(e);
    marke.style.left = (a * 100) + "%";
    marke.textContent = `${mmss(a * dauer)} · ${szenen[szeneBei(a * dauer)].titel}`;
    if (ziehen) springe(a * dauer);
  });
  zeitleiste.addEventListener("pointerup", () => { ziehen = false; });
  zeitleiste.addEventListener("keydown", (e) => {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") { e.preventDefault(); springe(v.currentTime + (e.key === "ArrowRight" ? 5 : -5)); }
  });

  // Vollbild (iOS: nur das Video selbst kann Vollbild).
  schirm.querySelector("[data-k=vollbild]").addEventListener("click", () => {
    if (document.fullscreenElement) document.exitFullscreen();
    else if (schirm.requestFullscreen) schirm.requestFullscreen().catch(() => {});
    else if (v.webkitEnterFullscreen) v.webkitEnterFullscreen();
  });

  // Tastatur, solange der Schirm den Fokus hat.
  schirm.addEventListener("keydown", (e) => {
    if (e.target === zeitleiste && e.key.startsWith("Arrow")) return;
    if (e.key === " " || e.key === "k") { e.preventDefault(); v.paused ? spielen() : v.pause(); }
    else if (e.key === "f") schirm.querySelector("[data-k=vollbild]").click();
    else if (e.key === "ArrowRight") springe(v.currentTime + 5);
    else if (e.key === "ArrowLeft") springe(v.currentTime - 5);
    wecken();
  });

  // Bedienung blendet sich beim Abspielen nach kurzer Ruhe aus.
  let ruheZeit = 0;
  function ruheBald() { clearTimeout(ruheZeit); ruheZeit = setTimeout(() => { if (!v.paused) schirm.classList.add("kino-ruhe"); }, 2600); }
  function wecken() { schirm.classList.remove("kino-ruhe"); clearTimeout(ruheZeit); if (!v.paused) ruheBald(); }
  schirm.addEventListener("pointermove", wecken);
  schirm.addEventListener("touchstart", wecken, { passive: true });

  // Ambilicht: mittlere Farbe der linken und rechten Bildhälfte färbt Rahmenschein und Raumlicht.
  const probe = document.createElement("canvas");
  probe.width = 8; probe.height = 4;
  const pc = probe.getContext("2d", { willReadFrequently: true });
  const misch = (d, x0) => {
    let r = 0, g = 0, b = 0, n = 0;
    for (let y = 0; y < 4; y++) for (let x = x0; x < x0 + 4; x++) { const i = (y * 8 + x) * 4; r += d[i]; g += d[i + 1]; b += d[i + 2]; n++; }
    const h = 1.6; // etwas kräftiger, damit der Schein sichtbar wird
    return `rgba(${Math.min(255, r / n * h) | 0},${Math.min(255, g / n * h) | 0},${Math.min(255, b / n * h) | 0},.62)`;
  };
  const licht = () => {
    if (!v.paused && !v.ended && v.readyState >= 2) {
      try {
        pc.drawImage(v, 0, 0, 8, 4);
        const d = pc.getImageData(0, 0, 8, 4).data;
        koerper.style.setProperty("--licht-l", misch(d, 0));
        koerper.style.setProperty("--licht-r", misch(d, 4));
      } catch (_) { /* fremde Quelle: kein Ambilicht */ }
    }
  };
  setInterval(licht, ruhig ? 1200 : 280);

  // Sternenfeld mit leichter Tiefe; folgt sanft dem Zeiger.
  const leinwand = document.querySelector(".kino-sterne");
  if (leinwand && !ruhig) {
    const c = leinwand.getContext("2d");
    let w = 0, h = 0, sterne = [], mx = 0, my = 0, zx = 0, zy = 0;
    const neu = () => {
      const r = leinwand.getBoundingClientRect(), d = Math.min(2, devicePixelRatio || 1);
      w = r.width; h = r.height; leinwand.width = w * d; leinwand.height = h * d; c.setTransform(d, 0, 0, d, 0, 0);
      sterne = Array.from({ length: Math.round(w * h / 5200) }, () => ({ x: Math.random() * w, y: Math.random() * h, z: Math.random() ** 2, p: Math.random() * 6.28 }));
    };
    neu(); addEventListener("resize", neu);
    addEventListener("pointermove", (e) => { mx = (e.clientX / innerWidth - 0.5); my = (e.clientY / innerHeight - 0.5); });
    let sichtbar = true;
    new IntersectionObserver(([e]) => { sichtbar = e.isIntersecting; }).observe(leinwand);
    const bild = (t) => {
      requestAnimationFrame(bild);
      if (!sichtbar || document.hidden) return;
      zx += (mx - zx) * 0.04; zy += (my - zy) * 0.04;
      c.clearRect(0, 0, w, h);
      const tempo = koerper.classList.contains("kino-spielt") ? 0.012 : 0.004;
      for (const s of sterne) {
        s.x -= tempo * (0.3 + s.z * 2.2) * 16; if (s.x < -4) s.x = w + 4;
        const x = s.x - zx * 30 * s.z, y = s.y - zy * 20 * s.z;
        const a = 0.25 + 0.6 * s.z * (0.65 + 0.35 * Math.sin(t / 900 + s.p));
        c.fillStyle = `rgba(${200 + 55 * s.z | 0},${215 + 30 * s.z | 0},255,${a})`;
        c.fillRect(x, y, 0.6 + s.z * 1.5, 0.6 + s.z * 1.5);
      }
    };
    requestAnimationFrame(bild);
  }

  zustand(); zeichne();
  // Autostart am großen Bildschirm, nachdem sich der Schirm geöffnet hat (Film ist stumm, Browser erlauben das).
  if (!ruhig && !sparen && innerWidth >= 900) setTimeout(() => { if (v.paused && v.currentTime === 0) spielen(); }, 1500);
})();
