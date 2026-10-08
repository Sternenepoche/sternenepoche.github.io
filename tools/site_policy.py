"""Gemeinsame KI-Kennzeichnung und Browser-Richtlinie für öffentliche HTML-Seiten."""
from __future__ import annotations

import base64
import hashlib
import html
import re
from pathlib import Path

NOTICE_CSS = """
.site-ai-notice{box-sizing:border-box;position:relative;z-index:6;display:flex;align-items:center;justify-content:center;flex-wrap:wrap;gap:4px 12px;margin:0;padding:8px 16px;background:#10212d;color:#d3e3e8;border-bottom:1px solid #38515d;font:12px/1.6 system-ui,'Segoe UI',sans-serif;letter-spacing:0;text-align:center}
.site-ai-notice .site-ai-tag{display:inline-block;padding:0 7px;border:1px solid #729e9c;border-radius:4px;color:#bce9de;font-weight:700;letter-spacing:.05em}
.site-ai-notice a{display:inline-flex;align-items:center;min-height:28px;color:#f2d3a2;text-decoration:underline;text-underline-offset:3px;font:inherit;white-space:nowrap}
.site-ai-notice a:focus-visible{outline:2px solid #f2d3a2;outline-offset:3px}
@media(max-width:480px){.site-ai-notice{font-size:11px;padding:7px 12px;gap:2px 8px}.site-ai-notice .site-ai-description{flex-basis:calc(100% - 50px);text-align:left}}
.ai-image-label{box-sizing:border-box;position:absolute;z-index:5;display:block;width:auto;min-width:0;margin:0;padding:1px 3px;border:1px solid #ffffff42;border-radius:3px;background:#07111dd9;color:#fff;font:600 8px/1.2 system-ui,'Segoe UI',sans-serif;letter-spacing:.03em;text-align:center;white-space:nowrap;pointer-events:none;transform:translate(-100%,-100%)}
.ai-image-label[hidden]{display:none!important}
.ai-background-label{right:3px;bottom:3px;transform:none}
.ai-page-label{position:fixed}
""".strip()


def notice(href: str) -> str:
    return ('<div class="site-ai-notice" data-ai-disclosure="true" role="note" aria-label="KI-Kennzeichnung">'
            '<span class="site-ai-tag">KI</span>'
            '<span class="site-ai-description">Bild, Ton und Text sind KI-generiert</span>'
            f'<a href="{html.escape(href, quote=True)}">KI-Hinweis</a></div>')


def decorate(document: str, href: str = "/KI-Hinweis.html", *,
             models: bool = False, handlers: bool = False, inline_style: bool = True) -> str:
    """Keine fremden Skriptquellen/Frames; geprüfte eigene Inline-Skripte per Hash erlauben.

    Der Betrachter hat vorhandene dynamische Inline-Klickhandler. Nur dort bleibt
    unsafe-inline für Skripte nötig; fremde Skript-URLs sind auch dort gesperrt.
    Modellverbindungen werden ausschließlich für den Spielclient freigegeben.
    """
    if 'data-ai-disclosure="true"' not in document:
        document, count = re.subn(r'(<body\b[^>]*>)', lambda m: m[1] + '\n' + notice(href), document, count=1, flags=re.I)
        if count != 1:
            raise ValueError("Öffentliche Seite hat kein body-Element")
    document = re.sub(r'(<span class="site-ai-description">).*?(</span>)',
                      r'\g<1>Bild, Ton und Text sind KI-generiert\2', document, flags=re.S)
    if inline_style:
        document = re.sub(r'<style id="site-ai-notice-style">.*?</style>\s*', '', document, flags=re.S)
        document = document.replace('</head>', '<style id="site-ai-notice-style">' + NOTICE_CSS + '</style>\n</head>', 1)
    # Standalone pages keep their labels offline; the game loads the same script externally.
    document = re.sub(r'<script id="site-ai-labels">.*?</script>\s*', '', document, flags=re.S)
    if not re.search(r'<script\b[^>]*(?:id="site-ai-labels"|src="ai-labels.js")', document):
        script = (Path(__file__).resolve().parents[1] / 'web-client/ai-labels.js').read_text(encoding='utf-8')
        document = document.replace('</body>', '<script id="site-ai-labels">' + script + '</script>\n</body>', 1)
    sources = ["'self'"]
    if handlers:
        sources.append("'unsafe-inline'")
    else:
        for script in re.finditer(r'<script\b([^>]*)>(.*?)</script\s*>', document, re.I | re.S):
            if not re.search(r'\bsrc\s*=', script[1], re.I):
                digest = base64.b64encode(hashlib.sha256(script[2].encode('utf-8')).digest()).decode('ascii')
                sources.append("'sha256-" + digest + "'")
    connect = "'self' https: http://127.0.0.1:* http://localhost:*" if models else "'self'"
    policy = ("default-src 'none'; script-src " + ' '.join(dict.fromkeys(sources))
              + "; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; "
              + "connect-src " + connect + "; media-src 'self'; frame-src 'none'; object-src 'none'; "
              + "worker-src 'none'; base-uri 'none'; form-action 'self'")
    document = re.sub(r'<meta\b[^>]*http-equiv\s*=\s*[\"\']Content-Security-Policy[\"\'][^>]*>\s*', '', document, flags=re.I)
    meta = '<meta http-equiv="Content-Security-Policy" content="' + html.escape(policy, quote=True) + '">'
    document, count = re.subn(r'(<meta\b[^>]*charset\s*=[^>]*>)', lambda m: m[1] + '\n' + meta, document, count=1, flags=re.I)
    if count != 1:
        raise ValueError("Öffentliche Seite braucht eine charset-Angabe")
    return document
