"""Weiche Trennstellen für lange deutsche Wörter – dieselben Regeln wie window.Silben im Spiel (web-client/stable-dom.js):
erst Wortfugen bekannter Grundwörter, dann Silben je Teil. Karl 25./26.10.2026: nie mitten im Wort umbrechen; Edge am Desktop
trennt mit hyphens:auto nicht."""
import re

FUGEN = ['synthesizer', 'kolonisieren', 'wissenschaft', 'transporter', 'verbrauch', 'kapazität', 'infanterie', 'generator',
         'zerstörer', 'kraftwerk', 'förderung', 'abkommen', 'spionage', 'schlacht', 'frachter', 'satellit', 'extraktor', 'geschütz',
         'schleier', 'krümmer', 'kreuzer', 'antrieb', 'technik', 'bündnis', 'phalanx', 'zentrum', 'pionier', 'kaution', 'brecher',
         'gebäude', 'archiv', 'punkte', 'gürtel', 'schlucht', 'rakete', 'drohne', 'panzer', 'kräfte', 'werfer', 'schiff', 'stoff',
         'lager', 'kunde', 'sonde', 'frost', 'werft', 'werk', 'welt', 'mond', 'pakt', 'netz', 'dock']
VOKAL = re.compile(r"[aeiouäöüy]", re.I)
DOPPEL = ['au', 'äu', 'eu', 'ei', 'ai', 'ey', 'ie', 'aa', 'ee', 'oo']
ANFANG = ['schl', 'schm', 'schn', 'schr', 'schw', 'sch', 'bl', 'br', 'dr', 'fl', 'fr', 'gl', 'gr', 'kl', 'kn', 'kr', 'pf',
          'pl', 'pr', 'tr', 'zw', 'ch', 'ck', 'ph', 'th', 'qu']
SHY = "­"


def _teil(w: str) -> str:
    if len(w) < 6:
        return w
    k, stellen, i = w.lower(), [], 0
    while i < len(k) and not VOKAL.match(k[i]):
        i += 1
    while i < len(k):
        e = i + 1
        while e < len(k) and VOKAL.match(k[e]) and k[e - 1:e + 1] in DOPPEL:
            e += 1
        n = e
        while n < len(k) and not VOKAL.match(k[n]):
            n += 1
        if n >= len(k):
            break
        if n == e:
            s = e
        else:
            c = k[e:n]
            a = next((x for x in ANFANG if c.endswith(x) and (len(c) > len(x) or x != 'ck')), None)
            s = n - (len(a) if a else 1)
            if a == 'ck':
                s = n - 1
        if s >= 2 and len(k) - s >= 2:
            stellen.append(s)
        i = n
    out, z = "", 0
    for s in stellen:
        out += w[z:s] + SHY
        z = s
    return out + w[z:]


def _wort(w: str) -> str:
    if len(w) < 10:
        return w
    k = w.lower()
    f = next((x for x in FUGEN if k.endswith(x) and len(w) - len(x) >= 3), None)
    if f:
        return _wort(w[:len(w) - len(f)]) + SHY + _teil(w[len(w) - len(f):])
    return _teil(w)


def trennen(text: str) -> str:
    return re.sub(r"[A-Za-zÄÖÜäöüß]{10,}", lambda m: _wort(m[0]), text)


def html(text: str) -> str:
    """Für schon maskierten HTML-Text: weiche Trennstellen als &shy;."""
    return trennen(text).replace(SHY, "&shy;")
