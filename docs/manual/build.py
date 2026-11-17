#!/usr/bin/env python3
"""Builds Front 303's manual (English and Spanish) as HTML, then PDF through Chromium."""
import csv, html, sys
from pathlib import Path

HERE = Path(__file__).parent
VERSION = "0.4.0"
CONTROLS = list(csv.reader(open(HERE / "controls.tsv"), delimiter="\t"))

CSS = r"""
@page { size: A4; margin: 18mm 17mm 20mm 17mm;
  @bottom-center { content: counter(page); font: 8.5pt Inter, sans-serif; color: #888; }
  @bottom-left { content: "Front 303 · v__VERSION__"; font: 8pt Inter, sans-serif; color: #aaa; }
  @bottom-right { content: "__RIGHT__"; font: 8pt Inter, sans-serif; color: #aaa; } }
@page cover { margin: 0; @bottom-center { content: none } @bottom-left { content: none } @bottom-right { content: none } }
* { box-sizing: border-box; }
body { margin: 0; font-family: Inter, "DejaVu Sans", sans-serif; font-size: 9.6pt; line-height: 1.5; color: #1d1e21; }
h1 { font-size: 19pt; margin: 9mm 0 4mm; color: #141416; letter-spacing: -0.3pt; page-break-after: avoid; padding-top: 2mm; border-bottom: 3px solid #ff8a1c; padding-bottom: 2mm; }
h2 { font-size: 13.5pt; margin: 7mm 0 2mm; color: #141416; page-break-after: avoid; }
h3 { font-size: 11pt; margin: 5mm 0 1.5mm; color: #c45a00; page-break-after: avoid; }
p { margin: 0 0 2.4mm; text-align: justify; hyphens: auto; }
ul, ol { margin: 0 0 2.6mm; padding-left: 5mm; } li { margin-bottom: 1mm; }
code, .id { font-family: "DejaVu Sans Mono", monospace; font-size: 8.3pt; background: #f2f2f4; padding: 0 2px; border-radius: 2px; }
table { width: 100%; border-collapse: collapse; margin: 2mm 0 4mm; font-size: 8.4pt; page-break-inside: auto; }
tr { page-break-inside: avoid; }
th { background: #26272b; color: #fff; text-align: left; padding: 1.6mm 2mm; font-weight: 600; }
td { border-bottom: 1px solid #e3e3e6; padding: 1.4mm 2mm; vertical-align: top; }
tr:nth-child(even) td { background: #fafafb; }
td.k { white-space: nowrap; font-weight: 600; }
td.r { color: #444; }
h1.brk { page-break-before: always; margin-top: 0; }
table.params td:nth-child(1) { width: 24%; } table.params td:nth-child(2) { width: 15%; }
table.params td:nth-child(3) { width: 17%; } table.params td:nth-child(4) { width: 8%; }
table.params td.k { white-space: normal; }
figure { margin: 3mm 0 4mm; page-break-inside: avoid; text-align: center; }
figure img { width: 100%; border-radius: 3px; box-shadow: 0 1px 4px rgba(0,0,0,.25); }
figure.half img { width: 62%; }
figcaption { font-size: 8pt; color: #666; margin-top: 1.5mm; }
.note { background: #fff5ea; border-left: 3px solid #ff8a1c; padding: 2mm 3mm; margin: 2mm 0 3mm; font-size: 9pt; }
.cover { page: cover; height: 295mm; overflow: hidden; page-break-after: always; background: radial-gradient(circle at 70% 20%, #3a3b40 0%, #141416 70%); color: #eee; padding: 30mm 22mm; position: relative; }
.cover .logo { font-size: 58pt; font-weight: 800; letter-spacing: -1pt; margin-top: 10mm; }
.cover .logo b { color: #ff8a1c; }
.cover .sub { font-size: 14pt; color: #bbb; margin-top: 2mm; }
.cover img { width: 86%; display: block; margin: 12mm auto 0; border-radius: 4px; box-shadow: 0 6px 24px rgba(0,0,0,.6); }
.cover .meta { position: absolute; bottom: 16mm; left: 22mm; right: 22mm; font-size: 9.5pt; color: #999; line-height: 1.6; }
.toc { columns: 2; column-gap: 10mm; font-size: 9.5pt; }
.toc div { margin-bottom: 1.2mm; break-inside: avoid; }
.two { display: flex; gap: 4mm; } .two figure { flex: 1; }
.flow { width: 100%; margin: 2mm 0 4mm; }
"""


def esc(s):
    return html.escape(s)


def fig(src, caption, cls=""):
    return f'<figure class="{cls}"><img src="img/{src}"><figcaption>{caption}</figcaption></figure>'


def table(head, rows, classes=None, cls=""):
    out = [f'<table class="{cls}"><tr>' + "".join(f"<th>{h}</th>" for h in head) + "</tr>"]
    for r in rows:
        cells = []
        for i, c in enumerate(r):
            cls = (classes or [""] * len(r))[i]
            cells.append(f'<td class="{cls}">{c}</td>')
        out.append("<tr>" + "".join(cells) + "</tr>")
    return "\n".join(out) + "</table>"


FLOW_SVG = """
<svg class="flow" viewBox="0 0 760 250" xmlns="http://www.w3.org/2000/svg" font-family="Inter, sans-serif" font-size="11">
<defs><marker id="a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0,0 L10,5 L0,10 z" fill="#555"/></marker></defs>
<g fill="#26272b" stroke="none">
<rect x="5" y="20" width="110" height="34" rx="5"/><rect x="5" y="70" width="110" height="34" rx="5"/>
<rect x="5" y="120" width="110" height="34" rx="5"/><rect x="5" y="170" width="110" height="34" rx="5"/></g>
<g fill="#fff" text-anchor="middle"><text x="60" y="34">SEQ 1 → 303-1</text><text x="60" y="47" font-size="9" fill="#ffb066">__MODEL__</text>
<text x="60" y="84">SEQ 2 → 303-2</text><text x="60" y="97" font-size="9" fill="#ffb066">__MODEL__</text>
<text x="60" y="134">SEQ 3 → 303-3</text><text x="60" y="147" font-size="9" fill="#ffb066">__MODEL__</text>
<text x="60" y="184">SEQ 4 → 303-4</text><text x="60" y="197" font-size="9" fill="#ffb066">__MODEL__</text></g>
<rect x="150" y="20" width="170" height="184" rx="6" fill="#fff1e2" stroke="#ff8a1c"/>
<text x="235" y="40" text-anchor="middle" font-weight="700">__STRIP__</text>
<text x="235" y="62" text-anchor="middle">DIST (ReBirth / Setekh)</text>
<text x="235" y="80" text-anchor="middle">PCF</text>
<text x="235" y="98" text-anchor="middle">COMP (ReBirth / Valentine)</text>
<text x="235" y="116" text-anchor="middle">LEVEL · PAN</text>
<text x="235" y="146" text-anchor="middle" fill="#666" font-size="10">__STEMS__</text>
<rect x="360" y="20" width="150" height="70" rx="6" fill="#e8ffe9" stroke="#3fae4a"/>
<text x="435" y="42" text-anchor="middle" font-weight="700">DELAY</text><text x="435" y="60" text-anchor="middle" font-size="10">ReBirth / Tape / Intervals</text><text x="435" y="76" text-anchor="middle" font-size="9" fill="#555">__DLYSEND__</text>
<rect x="360" y="110" width="150" height="70" rx="6" fill="#f3eaff" stroke="#9b5ce0"/>
<text x="435" y="132" text-anchor="middle" font-weight="700">REVERB</text><text x="435" y="150" text-anchor="middle" font-size="10">Hall / Early / Aether / Cloud</text><text x="435" y="166" text-anchor="middle" font-size="9" fill="#555">__REVSEND__</text>
<rect x="550" y="20" width="200" height="70" rx="6" fill="#26272b"/>
<text x="650" y="42" text-anchor="middle" fill="#fff" font-weight="700">MASTER</text><text x="650" y="60" text-anchor="middle" fill="#ddd" font-size="10">__MASTER__</text><text x="650" y="76" text-anchor="middle" fill="#ffb066" font-size="10">__MAINOUT__</text>
<rect x="550" y="110" width="200" height="94" rx="6" fill="#fff" stroke="#999" stroke-dasharray="4 3"/>
<text x="650" y="130" text-anchor="middle" font-weight="700">MULTI-OUT</text><text x="650" y="150" text-anchor="middle" font-size="10">303-1…4 → 3/4 · 5/6 · 7/8 · 9/10</text><text x="650" y="168" text-anchor="middle" font-size="10">FX → 11/12</text><text x="650" y="188" text-anchor="middle" font-size="9" fill="#666">__MULTI__</text>
<g stroke="#555" stroke-width="1.4" fill="none" marker-end="url(#a)">
<path d="M115 37 H150"/><path d="M115 87 H150"/><path d="M115 137 H150"/><path d="M115 187 H150"/>
<path d="M320 55 H360"/><path d="M320 145 H360"/><path d="M435 90 V110"/>
<path d="M510 55 H550"/><path d="M510 145 C530 145 530 70 550 70"/><path d="M320 170 C450 230 600 230 650 204"/></g>
<text x="160" y="232" font-size="10" fill="#555">__MIDI__</text>
</svg>"""


def build(lang):
    T = TEXT[lang]
    flow = FLOW_SVG
    for k, v in T["flow"].items():
        flow = flow.replace(f"__{k}__", v)
    body = T["body"](flow)
    css = CSS.replace("__VERSION__", VERSION).replace("__RIGHT__", T["right"])
    doc = f"""<!DOCTYPE html><html lang="{lang}"><head><meta charset="utf-8"><title>{T['title']}</title>
<style>{css}</style></head><body>{body}</body></html>"""
    out = HERE / f"manual_{lang}.html"
    out.write_text(doc, encoding="utf-8")
    return out


# ---------------------------------------------------------------------------------------------
# parameter tables

def short(rng):
    parts = rng.split(" / ")
    if len(rng) > 48 and len(parts) > 3:
        return " / ".join(parts[:3]) + f" … ({len(parts)})"
    return rng


def per_voice_rows(desc):
    rows = []
    for idv, name, group, rng, default in CONTROLS:
        if not idv.startswith("v1_"):
            continue
        suffix = idv[3:]
        rows.append([f'<span class="id">v1…v4_{suffix}</span>', esc(name.replace("303-1 ", "")),
                     esc(short(rng)), esc(default), desc.get(suffix, "")])
    return rows


def global_rows(desc, ids):
    rows = []
    for idv, name, group, rng, default in CONTROLS:
        if idv in ids:
            rows.append([f'<span class="id">{idv}</span>', esc(name), esc(short(rng)), esc(default), desc.get(idv, "")])
    return rows


GROUPS = [
    ("transport", ["run", "tempo", "shuffle", "root", "scale", "master", "quality", "output"]),
    ("dist", ["dist_mode", "dist_amount", "dist_shape", "dist_level"]),
    ("pcf", ["pcf_channel", "pcf_mode", "pcf_pattern", "pcf_freq", "pcf_reso", "pcf_amount", "pcf_decay"]),
    ("comp", ["comp_mode", "comp_route", "comp_amount", "comp_threshold", "comp_ratio", "comp_attack",
              "comp_release", "comp_saturate", "comp_crush", "comp_mix"]),
    ("delay", ["delay_on", "delay_mode", "delay_steps", "delay_triplet", "delay_feedback", "delay_pan",
               "delay_a", "delay_b", "delay_c", "delay_kind"]),
    ("reverb", ["rev_on", "rev_mode", "rev_program", "rev_predelay", "rev_size", "rev_decay", "rev_damping",
                "rev_mod", "rev_width", "rev_delay", "rev_level"]),
]

HALL = ["Bright Room", "Large Chamber", "Acoustic Studio", "Small Clear Hall", "Medium Bright Hall",
        "Medium Dark Hall", "Large Clear Hall", "Great Hall"]
EARLY = ["Abrupt Echo", "Backstage Pass", "Concert Venue", "Damaged Goods", "Elevator Pitch",
         "Floor Thirteen", "Garage Band", "Home Studio"]
AETHER = ["Velvet Hall", "Wood Room", "Stone Chamber", "Cathedral", "Bright Plate", "Slow Bloom",
          "Dark Cave", "Endless"]
CLOUD = ["Dull Echoes", "Hyperplane", "Medium Space", "Noise in the Hallway", "Rubi-Ka Fields",
         "Small Room", "The 90s Are Back", "Through the Looking Glass"]
PATTERNS = [("STRAIGHT", 8), ("PING PONG", 12), ("DOTTED", 13), ("TRESILLO", 16), ("SWELL", 12),
            ("GALLOP", 12), ("SCATTER", 16), ("STUTTER", 16)]


def programs_table(head):
    rows = [[str(i + 1), HALL[i], EARLY[i], AETHER[i], CLOUD[i]] for i in range(8)]
    return table(head, rows, ["k", "", "", "", ""])


def param_tables(T):
    out = []
    head = T["param_head"]
    cls = ["", "k", "r", "r", ""]
    out.append(f"<h2>{T['group_names']['voice']}</h2>")
    out.append(table(head, per_voice_rows(T["voice_desc"]), cls, "params"))
    for g, ids in GROUPS:
        out.append(f"<h2>{T['group_names'][g]}</h2>")
        out.append(table(head, global_rows(T["global_desc"], ids), cls, "params"))
    return "\n".join(out)


from manual_text import TEXT  # noqa: E402  (the prose, in its own file)

if __name__ == "__main__":
    import manual_text
    manual_text.setup(sys.modules[__name__])
    for lang in ("en", "es"):
        print(build(lang))
