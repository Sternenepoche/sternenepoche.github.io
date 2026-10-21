/* Hörchronik: ein Hördeck für fünf Folgen – Kapitel, Tempo, gemerkte Hörstelle, Mini-Leiste, Nachlesen. Ohne Abhängigkeiten. */
(() => {
  "use strict";
  const wurzel = document.querySelector("[data-chronik]");
  if (!wurzel) return;
  const folgen = JSON.parse(wurzel.dataset.folgen);
  const deck = wurzel.querySelector(".hc-deck");
  const a = new Audio();
  a.preload = "metadata";
  const $ = (s, w = deck) => w.querySelector(s);
  const SPIEL = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4.5v15l12-7.5z"/></svg>';
  const PAUSE = '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 4h4.5v16H6zM13.5 4H18v16h-4.5z"/></svg>';
  const mmss = (s) => { s = Math.max(0, s | 0); const h = s / 3600 | 0, m = (s % 3600) / 60 | 0; return (h ? h + ":" + String(m).padStart(2, "0") : m) + ":" + String(s % 60).padStart(2, "0"); };
  const SPEICHER = "sternenepoche-hoerchronik";
  const lies = () => { try { return JSON.parse(localStorage.getItem(SPEICHER) || "{}"); } catch (_) { return {}; } };
  const merke = (id, t) => { try { const m = lies(); m[id] = t; localStorage.setItem(SPEICHER, JSON.stringify(m)); } catch (_) { /* privat */ } };

  if (!a.canPlayType('audio/ogg; codecs="opus"')) {
    const p = document.createElement("p");
    p.className = "hc-hinweis";
    p.textContent = "Dieser Browser spielt das Tonformat Opus nicht ab. Aktuelle Fassungen von Chrome, Edge, Firefox und Safari können es.";
    $(".hc-kopf").append(p);
  }

  let f = null, kapitelAktiv = -1, textGeladen = null;
  const leiste = $(".hc-leiste"), voll = leiste.querySelector(".voll");
  const kapListe = $(".hc-kapitel");
  const mini = document.querySelector(".hc-mini");
  const lesen = wurzel.querySelector(".hc-lesen");
  const karten = [...wurzel.querySelectorAll(".hc-folge")];

  const kapitelBei = (t) => f.kapitel.reduce((x, k, i) => (t + 0.3 >= k.start ? i : x), 0);
  const zeichne = () => {
    if (!f) return;
    const t = a.currentTime || 0;
    voll.style.width = (100 * t / f.dauer) + "%";
    $(".hc-zeit").textContent = `${mmss(t)} / ${mmss(f.dauer)}`;
    leiste.setAttribute("aria-valuenow", Math.round(t));
    leiste.setAttribute("aria-valuetext", mmss(t));
    const i = kapitelBei(t);
    if (i !== kapitelAktiv) {
      kapitelAktiv = i;
      kapListe.querySelectorAll("li").forEach((li, n) => li.classList.toggle("an", n === i));
      $(".hc-jetzt b").textContent = f.kapitel[i].titel;
      lesen?.querySelectorAll("h3").forEach((h, n) => h.classList.toggle("an", n === i));
      if (mini) mini.querySelector("span").textContent = f.kapitel[i].titel;
      if ("mediaSession" in navigator && navigator.mediaSession.metadata) navigator.mediaSession.metadata.album = f.kapitel[i].titel;
    }
  };

  const waehle = (id, starten) => {
    const neu = folgen.find((x) => x.id === id);
    if (!neu) return;
    if (f && f.id !== id && a.currentTime > 5) merke(f.id, a.currentTime);
    f = neu; kapitelAktiv = -1;
    deck.style.setProperty("--farbe", f.farbe);
    $(".hc-bild img.motiv").src = f.bild;
    const w = $(".hc-bild .wappen");
    if (f.wappen) { w.src = f.wappen; w.hidden = false; } else w.hidden = true;
    $(".hc-kopf .kennung").textContent = `Folge ${Number(f.id.slice(0, 2))} · ${f.gastgeber}${f.rolle ? " · " + f.rolle : ""}`;
    $(".hc-kopf h2").textContent = f.titel;
    $(".hc-kopf p").textContent = f.kurz;
    leiste.querySelectorAll("i").forEach((i) => i.remove());
    leiste.setAttribute("aria-valuemax", Math.round(f.dauer));
    f.kapitel.slice(1).forEach((k) => { const i = document.createElement("i"); i.style.left = (100 * k.start / f.dauer) + "%"; leiste.append(i); });
    kapListe.innerHTML = "";
    f.kapitel.forEach((k, n) => {
      const li = document.createElement("li"), b = document.createElement("button");
      b.type = "button"; b.innerHTML = `<span>${mmss(k.start)}</span>`; b.append(k.titel.replace(/^Kapitel \d+ – /, ""));
      b.addEventListener("click", () => { a.currentTime = k.start; a.play().catch(() => {}); });
      li.append(b); kapListe.append(li);
    });
    karten.forEach((k) => k.setAttribute("aria-current", String(k.dataset.id === id)));
    a.src = f.url;
    const gemerkt = lies()[id];
    a.addEventListener("loadedmetadata", () => { if (gemerkt && gemerkt < f.dauer - 20) a.currentTime = gemerkt; zeichne(); }, { once: true });
    if (mini) { mini.querySelector("img").src = f.bild; mini.querySelector("b").textContent = f.titel; }
    if ("mediaSession" in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({ title: f.titel, artist: "Hörchronik der Sternenepoche · " + f.gastgeber, album: f.kapitel[0].titel,
        artwork: [{ src: new URL(f.bild, location.href).href, sizes: "512x512", type: "image/webp" }] });
    }
    if (lesen) { lesen.innerHTML = ""; textGeladen = null; if (lesen.closest("details").open) ladeText(); }
    zeichne();
    if (starten) a.play().catch(() => {});
  };

  const knopf = $(".hc-spiel");
  const zustand = () => {
    const laeuft = !a.paused;
    knopf.innerHTML = laeuft ? PAUSE : SPIEL;
    knopf.setAttribute("aria-label", laeuft ? "Anhalten" : "Abspielen");
    deck.classList.toggle("hc-spielt", laeuft);
    if (mini) mini.querySelector("button").innerHTML = laeuft ? PAUSE : SPIEL;
  };
  ["play", "pause", "ended"].forEach((e) => a.addEventListener(e, zustand));
  a.addEventListener("timeupdate", zeichne);
  let letzt = 0;
  a.addEventListener("timeupdate", () => { if (Math.abs(a.currentTime - letzt) > 10) { letzt = a.currentTime; merke(f.id, a.currentTime); } });
  a.addEventListener("ended", () => { merke(f.id, 0); const n = folgen.indexOf(f) + 1; if (n < folgen.length) waehle(folgen[n].id, true); });
  const umschalten = () => (a.paused ? a.play().catch(() => {}) : a.pause());
  knopf.addEventListener("click", umschalten);
  $("[data-k=zurueck]").addEventListener("click", () => { a.currentTime = Math.max(0, a.currentTime - 15); });
  $("[data-k=vor]").addEventListener("click", () => { a.currentTime = Math.min(f.dauer, a.currentTime + 15); });
  const tempo = $(".hc-tempo"), stufen = [1, 1.25, 1.5, 0.85];
  tempo.addEventListener("click", () => { const n = stufen[(stufen.indexOf(a.playbackRate) + 1) % stufen.length]; a.playbackRate = n; tempo.textContent = String(n).replace(".", ",") + "×"; });
  a.addEventListener("loadedmetadata", () => { const n = parseFloat(tempo.textContent.replace(",", ".")) || 1; a.playbackRate = n; });

  // Fortschrittsleiste: Klicken, Ziehen, Pfeiltasten.
  const anteil = (e) => { const r = leiste.getBoundingClientRect(); return Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)); };
  let ziehen = false;
  leiste.addEventListener("pointerdown", (e) => { ziehen = true; leiste.setPointerCapture(e.pointerId); a.currentTime = anteil(e) * f.dauer; zeichne(); });
  leiste.addEventListener("pointermove", (e) => { if (ziehen) { a.currentTime = anteil(e) * f.dauer; zeichne(); } });
  leiste.addEventListener("pointerup", () => { ziehen = false; });
  leiste.addEventListener("keydown", (e) => {
    if (e.key === "ArrowRight" || e.key === "ArrowLeft") { e.preventDefault(); a.currentTime += e.key === "ArrowRight" ? 15 : -15; }
  });

  karten.forEach((k) => k.addEventListener("click", () => {
    waehle(k.dataset.id, true);
    deck.scrollIntoView({ behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth", block: "start" });
  }));
  // Gemerkte Hörstellen auf den Karten.
  const gemerkt = lies();
  karten.forEach((k) => { const t = gemerkt[k.dataset.id]; if (t > 30) { const em = document.createElement("em"); em.textContent = "Weiterhören bei " + mmss(t); k.querySelector("div").append(em); } });

  // Mini-Leiste, solange das Deck nicht zu sehen ist und etwas läuft.
  if (mini) {
    let deckSichtbar = true;
    const mz = () => mini.classList.toggle("an", !deckSichtbar && (!a.paused || a.currentTime > 1) && !mini.dataset.zu);
    new IntersectionObserver(([e]) => { deckSichtbar = e.isIntersecting; mz(); }).observe(deck);
    ["play", "pause"].forEach((e) => a.addEventListener(e, mz));
    mini.querySelector("button").addEventListener("click", umschalten);
    mini.querySelector(".hc-mini-zu").addEventListener("click", () => { a.pause(); mini.dataset.zu = "1"; mz(); });
    a.addEventListener("play", () => { delete mini.dataset.zu; mz(); });
    mini.querySelector("div").addEventListener("click", () => deck.scrollIntoView({ block: "start" }));
  }
  if ("mediaSession" in navigator) {
    navigator.mediaSession.setActionHandler("play", () => a.play());
    navigator.mediaSession.setActionHandler("pause", () => a.pause());
    navigator.mediaSession.setActionHandler("seekbackward", () => { a.currentTime -= 15; });
    navigator.mediaSession.setActionHandler("seekforward", () => { a.currentTime += 15; });
    navigator.mediaSession.setActionHandler("previoustrack", () => { const k = f.kapitel[Math.max(0, kapitelBei(a.currentTime) - 1)]; a.currentTime = k.start; });
    navigator.mediaSession.setActionHandler("nexttrack", () => { const k = f.kapitel[kapitelBei(a.currentTime) + 1]; if (k) a.currentTime = k.start; });
  }

  // Nachlesen: Text der Folge erst beim Aufklappen laden.
  function ladeText() {
    if (!lesen || textGeladen === f.id) return;
    const id = f.id; textGeladen = id;
    lesen.textContent = "Text wird geladen …";
    fetch(f.textUrl).then((r) => r.json()).then((d) => {
      if (f.id !== id) return;
      lesen.innerHTML = "";
      d.kapitel.forEach((k, n) => {
        const h = document.createElement("h3"); h.textContent = k.titel; if (n === kapitelAktiv) h.classList.add("an");
        lesen.append(h);
        let vorher = "";
        for (const z of k.zeilen) {
          const p = document.createElement("p");
          if (z.wer !== vorher) { const b = document.createElement("b"); b.textContent = z.wer; b.style.color = (window.HC_FARBEN || {})[z.volk] || "var(--gold2)"; p.append(b); }
          p.append(z.text); lesen.append(p); vorher = z.wer;
        }
      });
    }).catch(() => { lesen.textContent = "Der Text konnte nicht geladen werden."; textGeladen = null; });
  }
  lesen?.closest("details").addEventListener("toggle", (e) => { if (e.target.open) ladeText(); });
  window.HC_FARBEN = JSON.parse(wurzel.dataset.farben || "{}");

  // Tastatur im Deck: Leertaste spielt/hält.
  deck.addEventListener("keydown", (e) => { if (e.key === " " && e.target === deck) { e.preventDefault(); umschalten(); } });

  zustand();
  const zuletzt = Object.entries(lies()).filter(([, t]) => t > 30).map(([id]) => id).pop();
  waehle(location.hash && folgen.some((x) => "#" + x.id === location.hash) ? location.hash.slice(1) : (zuletzt || folgen[0].id), false);
})();
