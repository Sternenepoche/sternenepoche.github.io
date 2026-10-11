/* Der Rat auf der Website: die vier Ratsmitglieder eines Volkes auf einer Kreisbühne (three.js).
   Klick auf eine Figur oder ihre Karte: die Kamera fährt zum Kopf, und das Ratsmitglied stellt sich in
   seiner Stimme vor (Untertitel Satz für Satz, Kopf nickt nach dem echten Pegel). Zurufe zum Anhören.
   Figuren ohne 3D-Modell stehen als freigestelltes Bild bzw. Hologramm-Porträt auf der Bühne.
   Ohne WebGL bleiben die Karten mit Porträt und Ton. Lädt die 3D-Modelle erst, wenn die Bühne sichtbar ist. */
(() => {
  "use strict";
  const wurzel = document.getElementById("rat-buehne");
  if (!wurzel) return;
  const BASIS = wurzel.dataset.basis || "docs/rat/";
  let T = window.THREE;
  const ROLLEN = ["stratege", "verwalter", "diplomat", "feldherr"];
  const TITEL = { stratege: "Stratege", verwalter: "Verwalter", diplomat: "Diplomat", feldherr: "Feldherr" };
  const FARBE = { stratege: "#ff6b83", verwalter: "#63b1ff", diplomat: "#4fd994", feldherr: "#ffb04a" };
  const ANLASS = { klick: "Angewählt", angriff: "Angriff!", erfolg: "Großer Erfolg", misserfolg: "Rückschlag", genervt: "Genervt", leerlauf: "Leerlauf" };
  const ruhig = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const el = (tag, klasse, text) => { const e = document.createElement(tag); if (klasse) e.className = klasse; if (text !== undefined) e.textContent = text; return e; };

  let daten = null, volk = wurzel.dataset.volk || "veyari";
  const leinwandHuelle = wurzel.querySelector(".rat-leinwand");
  const untertitel = wurzel.querySelector(".rat-untertitel");
  const karten = document.getElementById("rat-karten");
  const reiter = [...document.querySelectorAll("[data-rat-volk]")];

  // ------------------------------------------------------------------ Ton mit Pegel
  const format = new Audio().canPlayType('audio/ogg; codecs="opus"') ? "ogg" : "m4a";
  let ctx = null, laeuft = null, raf = 0;
  const puffer = new Float32Array(1024);
  function kontext() { if (!ctx) { try { ctx = new (window.AudioContext || window.webkitAudioContext)(); } catch (e) { ctx = null; } } if (ctx && ctx.state === "suspended") ctx.resume().catch(() => {}); return ctx; }
  function stopp() {
    const j = laeuft; laeuft = null;
    if (j) { try { j.audio.pause(); } catch (e) { /* schon zu */ } }
    cancelAnimationFrame(raf); setzePegel(null, 0); zeigeUntertitel("", null);
    wurzel.classList.remove("spricht"); for (const k of karten.querySelectorAll(".rat-karte")) k.classList.remove("spricht");
    fokus(null);
  }
  function abspielen(fid, eintrag, mitFokus) {
    stopp();
    const rolle = fid.split(".")[0];
    const a = new Audio(BASIS + eintrag[format]);
    const j = { fid, rolle, audio: a, eintrag, analyser: null };
    const c = kontext();
    try { if (!c) throw 0; const q = c.createMediaElementSource(a), an = c.createAnalyser(); an.fftSize = 1024; q.connect(an); an.connect(c.destination); j.analyser = an; } catch (e) { /* ohne Pegel */ }
    laeuft = j;
    a.onended = () => { if (laeuft === j) stopp(); };
    a.onerror = () => { if (laeuft === j) stopp(); };
    a.play().catch(() => { if (laeuft === j) stopp(); });
    wurzel.classList.add("spricht"); wurzel.style.setProperty("--rolle", FARBE[rolle]);
    const karte = karten.querySelector(`[data-fid="${fid}"]`); if (karte) karte.classList.add("spricht");
    if (mitFokus) fokus(rolle);
    const takt = () => {
      if (laeuft !== j) return;
      let p = 0;
      if (j.analyser) { j.analyser.getFloatTimeDomainData(puffer); let s = 0; for (let i = 0; i < puffer.length; i++) s += puffer[i] * puffer[i]; p = Math.min(1, Math.pow(Math.sqrt(s / puffer.length) * 3.6, 1.25)); }
      else if (!a.paused) p = 0.3 + 0.25 * Math.abs(Math.sin(performance.now() / 95));
      setzePegel(rolle, p);
      zeigeUntertitel(satz(eintrag, a.currentTime || 0), rolle);
      raf = requestAnimationFrame(takt);
    };
    raf = requestAnimationFrame(takt);
  }
  function satz(e, t) {
    if (!e.saetze || !e.saetze.length) return e.text;
    const s = e.saetze.find((x) => t >= x.start - 0.05 && t <= x.ende + 0.35);
    if (!s) return "";
    const teile = s.text.match(/[^.!?]+[.!?]+["“”»]*|[^.!?]+$/g) || [s.text];
    const anteil = (t - s.start) / Math.max(0.1, s.ende - s.start), gesamt = teile.reduce((a, x) => a + x.length, 0);
    let summe = 0; for (const x of teile) { summe += x.length; if (summe / gesamt >= anteil) return x.trim(); }
    return teile[teile.length - 1].trim();
  }
  function zeigeUntertitel(text, rolle) {
    if (untertitel.textContent !== text) untertitel.textContent = text;
    untertitel.classList.toggle("an", !!text);
    if (rolle) untertitel.style.setProperty("--rolle", FARBE[rolle]);
  }
  wurzel.addEventListener("pointerdown", kontext, { capture: true });

  // ------------------------------------------------------------------ Karten (funktionieren auch ohne WebGL)
  function kartenBauen() {
    karten.textContent = "";
    for (const rolle of ROLLEN) {
      const fid = `${rolle}.${volk}`, f = daten[fid];
      if (!f) continue;
      const k = el("article", "rat-karte"); k.dataset.fid = fid; k.style.setProperty("--rolle", FARBE[rolle]);
      const kopf = el("button", "rat-karte-kopf"); kopf.type = "button"; kopf.setAttribute("aria-label", `${f.name} stellt sich vor`);
      const bild = el("img"); bild.src = f.portraet; bild.alt = ""; bild.loading = "lazy"; bild.width = 72; bild.height = 72;
      const text = el("span", "rat-karte-text");
      text.append(el("small", null, TITEL[rolle]), el("b", null, f.name), el("span", null, f.kurz));
      kopf.append(bild, text, el("span", "rat-play", "▶"));
      kopf.onclick = () => (laeuft && laeuft.fid === fid ? stopp() : abspielen(fid, f.vorstellung, true));
      const chips = el("div", "rat-zurufe");
      for (const [anlass, titel] of Object.entries(ANLASS)) {
        const z = f.zeilen[anlass]; if (!z) continue;
        const b = el("button", "rat-chip", titel); b.type = "button"; b.title = `„${z.text}“`;
        b.onclick = () => abspielen(fid, z, true);
        chips.append(b);
      }
      k.append(kopf, chips);
      karten.append(k);
    }
  }

  // ------------------------------------------------------------------ 3D-Bühne
  let renderer = null, szene, kamera, figuren = {}, ziel = null, blick = null, gestartet = false, sichtbar = false;
  const PLATZ = { stratege: [-2.55, 0.35], verwalter: [-0.85, -0.25], diplomat: [0.85, -0.25], feldherr: [2.55, 0.35] };
  const UEBERSICHT = { pos: [0, 1.85, 7.3], blick: [0, 1.12, 0] };
  const pegelJe = {};
  let enge = 1;   // 1 auf breiten Bühnen, kleiner im Hochformat (Figuren rücken zusammen)
  const platz = (rolle) => { const [x, z] = PLATZ[rolle]; return [x * (0.52 + 0.48 * enge), z]; };
  const uebersicht = () => ({ pos: [0, 1.85 + (1 - enge) * 0.4, 7.3 + (1 - enge) * 3.2], blick: [0, 1.12, 0] });
  function setzePegel(rolle, p) { for (const r of ROLLEN) pegelJe[r] = r === rolle ? p : 0; }
  function fokus(rolle) {
    if (!renderer) return;
    if (!rolle) { ziel = uebersicht(); return; }
    const [x, z] = platz(rolle);
    ziel = { pos: [x * 0.86, 1.95, z + 2.05], blick: [x, 1.7, z] };
  }

  const TYP = { 5120: Int8Array, 5121: Uint8Array, 5122: Int16Array, 5123: Uint16Array, 5125: Uint32Array, 5126: Float32Array };
  const KOMP = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4, MAT4: 16 };
  /** Schlanker GLB-Lader (wie im Cockpit): Netze, PBR-Texturen aus dem Binärteil, aufrecht, 2 Einheiten hoch. */
  async function glb(pfad) {
    const daten = await (await fetch(pfad)).arrayBuffer(), dv = new DataView(daten);
    if (dv.getUint32(0, true) !== 0x46546c67) throw new Error("kein GLB");
    let pos = 12, json = null, bin = null;
    while (pos < daten.byteLength) { const n = dv.getUint32(pos, true), art = dv.getUint32(pos + 4, true), teil = daten.slice(pos + 8, pos + 8 + n);
      if (art === 0x4e4f534a) json = JSON.parse(new TextDecoder().decode(teil)); else if (art === 0x004e4942) bin = teil; pos += 8 + n; }
    const zugriff = (i) => { const a = json.accessors[i], bv = json.bufferViews[a.bufferView], Art = TYP[a.componentType], k = KOMP[a.type];
      const start = (bv.byteOffset || 0) + (a.byteOffset || 0), schritt = bv.byteStride || 0;
      if (!schritt || schritt === Art.BYTES_PER_ELEMENT * k) return { f: new Art(bin, start, a.count * k), k, n: !!a.normalized };
      const aus = new Art(a.count * k), q = new DataView(bin);
      for (let j = 0; j < a.count; j++) for (let c = 0; c < k; c++) { const o = start + j * schritt + c * Art.BYTES_PER_ELEMENT;
        aus[j * k + c] = Art === Float32Array ? q.getFloat32(o, true) : Art === Uint16Array ? q.getUint16(o, true) : Art === Uint32Array ? q.getUint32(o, true) : q.getUint8(o); }
      return { f: aus, k, n: !!a.normalized }; };
    const cache = new Map();
    const textur = async (info, farbe) => { if (!info) return null; const s = info.index + (farbe ? "f" : ""); if (cache.has(s)) return cache.get(s);
      const t = json.textures[info.index], img = json.images[t.source ?? t.extensions?.EXT_texture_webp?.source ?? 0], bv = json.bufferViews[img.bufferView];
      const bmp = await createImageBitmap(new Blob([new Uint8Array(bin, bv.byteOffset || 0, bv.byteLength)], { type: img.mimeType || "image/png" }), { imageOrientation: "none" });
      const tex = new T.Texture(bmp); tex.flipY = false; if (farbe) tex.colorSpace = T.SRGBColorSpace; tex.anisotropy = 4; tex.needsUpdate = true; cache.set(s, tex); return tex; };
    const material = async (i) => { const m = (json.materials || [])[i] || {}, p = m.pbrMetallicRoughness || {};
      const mat = new T.MeshStandardMaterial({ color: 0xffffff, metalness: p.metallicFactor ?? 1, roughness: p.roughnessFactor ?? 1 });
      if (p.baseColorFactor) mat.color.setRGB(p.baseColorFactor[0], p.baseColorFactor[1], p.baseColorFactor[2]);
      mat.map = await textur(p.baseColorTexture, true); const mr = await textur(p.metallicRoughnessTexture, false); if (mr) { mat.metalnessMap = mr; mat.roughnessMap = mr; }
      // ORM-Karte: Rot = Umgebungsverdeckung (Falten, unter den Armen), Grün = Rauheit, Blau = Metall.
      const ao = await textur(m.occlusionTexture, false); if (ao) { mat.aoMap = ao; mat.aoMapIntensity = m.occlusionTexture.strength ?? 1; }
      mat.normalMap = await textur(m.normalTexture, false); mat.side = m.doubleSided ? T.DoubleSide : T.FrontSide; return mat; };
    const gruppe = new T.Group();
    const knoten = async (ni, eltern) => { const n = json.nodes[ni], g = new T.Group();
      if (n.matrix) g.applyMatrix4(new T.Matrix4().fromArray(n.matrix)); else { if (n.translation) g.position.fromArray(n.translation); if (n.rotation) g.quaternion.fromArray(n.rotation); if (n.scale) g.scale.fromArray(n.scale); }
      if (n.mesh !== undefined) for (const prim of json.meshes[n.mesh].primitives) { const geo = new T.BufferGeometry();
        for (const [a, z] of [["POSITION", "position"], ["NORMAL", "normal"], ["TEXCOORD_0", "uv"]]) { if (prim.attributes[a] === undefined) continue; const w = zugriff(prim.attributes[a]); geo.setAttribute(z, new T.BufferAttribute(w.f, w.k, w.n)); }
        if (prim.indices !== undefined) geo.setIndex(new T.BufferAttribute(zugriff(prim.indices).f, 1));
        if (!geo.getAttribute("normal")) geo.computeVertexNormals();
        g.add(new T.Mesh(geo, await material(prim.material))); }
      for (const k of n.children || []) await knoten(k, g); eltern.add(g); };
    for (const ni of (json.scenes ? json.scenes[json.scene || 0].nodes : json.nodes.map((_, i) => i))) await knoten(ni, gruppe);
    let box = new T.Box3().setFromObject(gruppe), gr = box.getSize(new T.Vector3());
    if (gr.z > gr.y * 1.3) { gruppe.rotation.x = -Math.PI / 2; gruppe.updateMatrixWorld(true); box = new T.Box3().setFromObject(gruppe); gr = box.getSize(new T.Vector3()); }
    const huelle = new T.Group(), mitte = box.getCenter(new T.Vector3()); huelle.add(gruppe);
    gruppe.position.sub(new T.Vector3(mitte.x, box.min.y, mitte.z)); huelle.scale.setScalar(2 / Math.max(gr.y, 1e-6));
    return huelle;
  }
  /** Ohne 3D-Modell: freigestelltes Ganzkörperbild bzw. Porträt als leuchtende Tafel auf der Bühne.
      Die Breite folgt dem Seitenverhältnis des Bildes (sonst wirken schmale Ganzkörperbilder breit gezogen);
      der Alphatest stanzt die Figur sauber aus und schreibt Tiefe, damit der Bühnenboden nicht durch die Beine scheint. */
  function tafel(bild, ganz, farbe) {
    const g = new T.Group(), hoehe = ganz ? 2.0 : 1.25;
    const innen = new T.Group(); innen.position.y = ganz ? 1.0 : 1.35; g.add(innen);
    const flaeche = new T.Mesh(new T.PlaneGeometry(1, hoehe), new T.MeshBasicMaterial({ transparent: true, alphaTest: 0.35, side: T.DoubleSide }));
    flaeche.visible = false; innen.add(flaeche);
    const rahmen = ganz ? null : new T.Mesh(new T.PlaneGeometry(1, hoehe + 0.06), new T.MeshBasicMaterial({ color: new T.Color(farbe), transparent: true, opacity: 0.35 }));
    if (rahmen) { rahmen.position.z = -0.01; rahmen.visible = false; innen.add(rahmen); }
    const tex = new T.TextureLoader().load(bild, (t) => {
      const breite = hoehe * (t.image.width / t.image.height);
      flaeche.scale.x = breite; flaeche.visible = true;
      if (rahmen) { rahmen.scale.x = breite + 0.06; rahmen.visible = true; }
    });
    tex.colorSpace = T.SRGBColorSpace; flaeche.material.map = tex;
    g.userData.tafel = true; g.userData.bild = ganz ? null : innen; return g;
  }
  function bodenRing(farbe) {
    const g = new T.Group();
    const ring = new T.Mesh(new T.RingGeometry(0.48, 0.52, 96), new T.MeshBasicMaterial({ color: new T.Color(farbe), transparent: true, opacity: 0.7, side: T.DoubleSide, depthWrite: false }));
    ring.rotation.x = -Math.PI / 2; ring.position.y = 0.004; g.add(ring);
    const glanz = new T.Mesh(new T.CircleGeometry(0.48, 64), new T.MeshBasicMaterial({ color: new T.Color(farbe), transparent: true, opacity: 0.08, depthWrite: false }));
    glanz.rotation.x = -Math.PI / 2; glanz.position.y = 0.003; g.add(glanz);
    g.userData.ring = ring; return g;
  }
  /** three.js erst laden, wenn die Bühne in Sicht kommt (670 KB, die sonst niemand braucht). */
  function dreiLaden() {
    if (T) return Promise.resolve(T);
    return new Promise((ok, fehler) => {
      const s = document.createElement("script"); s.src = wurzel.dataset.three || "web-client/three.min.js";
      s.onload = () => { T = window.THREE; T ? ok(T) : fehler(new Error("three.js fehlt")); }; s.onerror = fehler;
      document.head.append(s);
    });
  }
  function buehneStarten() {
    if (gestartet) return; gestartet = true;
    const hinweis = wurzel.querySelector(".rat-hinweis"); if (hinweis) hinweis.textContent = "Der Rat betritt die Bühne …";
    dreiLaden().then(buehneBauen).catch(() => wurzel.classList.add("ohne-3d"));
  }
  function buehneBauen() {
    try { renderer = new T.WebGLRenderer({ antialias: true, alpha: true }); } catch (e) { renderer = null; wurzel.classList.add("ohne-3d"); return; }
    renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
    renderer.outputColorSpace = T.SRGBColorSpace; renderer.toneMapping = T.ACESFilmicToneMapping; renderer.toneMappingExposure = 1.3;
    leinwandHuelle.append(renderer.domElement);
    szene = new T.Scene(); kamera = new T.PerspectiveCamera(30, 2, 0.05, 100);
    szene.add(new T.HemisphereLight(0xcfe6ff, 0x20160c, 1.0));
    const l1 = new T.DirectionalLight(0xffe6c4, 2.4); l1.position.set(2.5, 5, 6); szene.add(l1);
    const l2 = new T.DirectionalLight(0x7fd3ff, 1.6); l2.position.set(-4, 3, -3); szene.add(l2);
    const l3 = new T.DirectionalLight(0xffffff, 0.5); l3.position.set(0, 2, 8); szene.add(l3);
    // Kreisbühne: großer Ring, Gitter und aufsteigende Funken.
    const boden = new T.Mesh(new T.RingGeometry(3.3, 3.36, 160), new T.MeshBasicMaterial({ color: 0x7fd3ff, transparent: true, opacity: 0.35, side: T.DoubleSide }));
    boden.rotation.x = -Math.PI / 2; szene.add(boden);
    const scheibe = new T.Mesh(new T.CircleGeometry(3.3, 128), new T.MeshBasicMaterial({ color: 0x0b2236, transparent: true, opacity: 0.55 }));
    scheibe.rotation.x = -Math.PI / 2; scheibe.position.y = -0.002; szene.add(scheibe);
    const n = 160, pkt = new Float32Array(n * 3);
    for (let i = 0; i < n; i++) { const w = Math.random() * Math.PI * 2, r = Math.random() * 3.2; pkt.set([Math.cos(w) * r, Math.random() * 3, Math.sin(w) * r * 0.6], i * 3); }
    const fgeo = new T.BufferGeometry(); fgeo.setAttribute("position", new T.BufferAttribute(pkt, 3));
    const funken = new T.Points(fgeo, new T.PointsMaterial({ color: 0x9fdcff, size: 0.02, transparent: true, opacity: 0.6, depthWrite: false })); szene.add(funken);
    blick = { pos: UEBERSICHT.pos.slice(), blick: UEBERSICHT.blick.slice() }; ziel = uebersicht();
    // Klick auf eine Figur: Vorstellung.
    const ray = new T.Raycaster(), maus = new T.Vector2();
    renderer.domElement.addEventListener("click", (e) => {
      const r = renderer.domElement.getBoundingClientRect(); maus.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
      ray.setFromCamera(maus, kamera);
      const treffer = ray.intersectObjects(Object.values(figuren).map((f) => f.ziel).filter(Boolean), false)[0];
      if (!treffer) { if (laeuft) stopp(); return; }
      const rolle = treffer.object.userData.rolle, fid = `${rolle}.${volk}`;
      if (laeuft && laeuft.fid === fid) stopp(); else abspielen(fid, daten[fid].vorstellung, true);
    });
    renderer.domElement.addEventListener("pointermove", (e) => {
      const r = renderer.domElement.getBoundingClientRect(); maus.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
      ray.setFromCamera(maus, kamera);
      renderer.domElement.style.cursor = ray.intersectObjects(Object.values(figuren).map((f) => f.ziel).filter(Boolean), false).length ? "pointer" : "default";
    });
    let letzte = performance.now();
    const schleife = (t) => {
      requestAnimationFrame(schleife);
      if (!sichtbar) return;
      const dt = Math.min(0.05, (t - letzte) / 1000); letzte = t;
      const w = leinwandHuelle.clientWidth, h = leinwandHuelle.clientHeight, c = renderer.domElement;
      if (w && h && (c.width !== Math.floor(w * renderer.getPixelRatio()) || c.height !== Math.floor(h * renderer.getPixelRatio()))) { renderer.setSize(w, h, false); kamera.aspect = w / h; kamera.fov = w / h < 1.3 ? 38 : 30; kamera.updateProjectionMatrix();
        enge = Math.max(0, Math.min(1, (w / h - 0.7) / 1.1)); if (!laeuft) ziel = uebersicht(); else fokus(laeuft.rolle); }
      const k = 1 - Math.pow(0.004, dt);
      for (let i = 0; i < 3; i++) { blick.pos[i] += (ziel.pos[i] - blick.pos[i]) * k; blick.blick[i] += (ziel.blick[i] - blick.blick[i]) * k; }
      const schwenk = ruhig || laeuft ? 0 : Math.sin(t / 6000) * 0.35;
      kamera.position.set(blick.pos[0] + schwenk, blick.pos[1], blick.pos[2]); kamera.lookAt(blick.blick[0], blick.blick[1], blick.blick[2]);
      for (const [rolle, f] of Object.entries(figuren)) {
        const [px, pz] = platz(rolle);
        f.ring.position.x = px; if (f.ziel) f.ziel.position.x = px;
        if (!f.obj) continue;
        f.obj.position.x = px; f.obj.position.z = pz;
        const p = (f.pegel = (f.pegel || 0) + ((pegelJe[rolle] || 0) - (f.pegel || 0)) * (1 - Math.pow(0.0005, dt)));
        const atmen = ruhig ? 0 : Math.sin(t / 1150 + f.phase), wiegen = ruhig ? 0 : Math.sin(t / 2700 + f.phase);
        if (f.obj.userData.tafel) { if (f.obj.userData.bild) f.obj.userData.bild.scale.setScalar(0.7 + 0.3 * enge); f.obj.position.y = 0.02 * atmen + p * 0.02 * Math.sin(t / 85); f.obj.lookAt(kamera.position.x, f.obj.position.y + 1, kamera.position.z); }
        else { f.obj.position.y = 0.006 * atmen + p * 0.012 * Math.sin(t / 85); f.obj.rotation.y = f.drehung + 0.05 * wiegen + p * 0.02 * Math.sin(t / 310); f.obj.rotation.x = 0.008 * atmen + p * (0.03 + 0.025 * Math.sin(t / 140)); }
        const dunkel = laeuft && laeuft.rolle !== rolle;
        f.ring.userData.ring.material.opacity = (dunkel ? 0.25 : 0.55) + Math.sin(t / 650 + f.phase) * 0.15 + p * 0.4;
      }
      const pos = fgeo.attributes.position;
      if (!ruhig) { for (let i = 0; i < n; i++) { let y = pos.getY(i) + dt * (0.08 + (i % 7) * 0.012); if (y > 3) y = 0; pos.setY(i, y); } pos.needsUpdate = true; }
      renderer.render(szene, kamera);
    };
    requestAnimationFrame(schleife);
    volkZeigen();
  }
  /** Die vier Figuren des gewählten Volkes auf die Bühne stellen (3D-Modell, sonst Tafel). */
  function volkZeigen() {
    if (!renderer) return;
    for (const f of Object.values(figuren)) { if (f.obj) szene.remove(f.obj); szene.remove(f.ring); if (f.ziel) szene.remove(f.ziel); }
    figuren = {};
    const hinweis = wurzel.querySelector(".rat-hinweis"), normal = "Ton an · Klick auf eine Figur: Vorstellung in ihrer Stimme";
    let offen = ROLLEN.length;
    const geladen = () => { offen -= 1; hinweis.textContent = offen > 0 ? `Der Rat betritt die Bühne … (${ROLLEN.length - offen}/${ROLLEN.length})` : normal; };
    hinweis.textContent = "Der Rat betritt die Bühne …";
    ROLLEN.forEach((rolle, i) => {
      const fid = `${rolle}.${volk}`, d = daten[fid]; if (!d) return;
      const [x, z] = PLATZ[rolle], drehung = Math.atan2(-x * 0.35, 6);
      const f = { phase: i * 1.7, drehung, ring: bodenRing(FARBE[rolle]) };
      f.ring.position.set(x, 0, z); szene.add(f.ring);
      const ziel = new T.Mesh(new T.CylinderGeometry(0.42, 0.42, 2.1, 12), new T.MeshBasicMaterial({ visible: false }));
      ziel.position.set(x, 1.05, z); ziel.userData.rolle = rolle; szene.add(ziel); f.ziel = ziel;
      figuren[rolle] = f;
      const stellen = (obj) => { if (volk !== fid.split(".")[1] || figuren[rolle] !== f) return; obj.position.set(x, 0, z); obj.rotation.y = drehung; f.obj = obj; szene.add(obj); geladen(); };
      if (d.glb) glb(BASIS + d.glb).then(stellen).catch(() => stellen(tafel(d.portraet, false, FARBE[rolle])));
      else stellen(tafel(d.ganz || d.portraet, !!d.ganz, FARBE[rolle]));
    });
  }

  // ------------------------------------------------------------------ Volk wählen, Start
  function volkWaehlen(v) {
    volk = v; stopp();
    for (const r of reiter) { const an = r.dataset.ratVolk === v; r.classList.toggle("an", an); r.setAttribute("aria-selected", an ? "true" : "false"); }
    wurzel.dataset.volk = v;
    if (!daten) return;   // rat.json lädt noch: die Wahl gilt, Karten und Bühne entstehen beim Eintreffen der Daten
    kartenBauen(); volkZeigen();
  }
  for (const r of reiter) r.addEventListener("click", () => volkWaehlen(r.dataset.ratVolk));
  fetch(BASIS + "rat.json").then((r) => r.json()).then((d) => {
    daten = d; volkWaehlen(volk);
    const beobachter = new IntersectionObserver((e) => { sichtbar = e.some((x) => x.isIntersecting); if (sichtbar) buehneStarten(); }, { rootMargin: "200px" });
    beobachter.observe(wurzel);
  }).catch(() => { wurzel.classList.add("ohne-3d"); });
})();
