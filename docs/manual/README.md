# Front 303 manual (sources)

The PDF manuals in `releases/` are built from these files:

```sh
cargo run -q --release -p front303-cli -- controls > docs/manual/controls.tsv
python3 docs/manual/build.py      # writes manual_en.html and manual_es.html
python3 docs/manual/render.py     # prints them to PDF with Chromium (Playwright)
```
