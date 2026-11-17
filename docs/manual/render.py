import asyncio, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
from playwright.async_api import async_playwright
async def main():
    async with async_playwright() as p:
        b = await p.chromium.launch()
        pg = await b.new_page()
        for lang, name in [("en","Front303-0.4.0-Manual-EN.pdf"),("es","Front303-0.4.0-Manual-ES.pdf")]:
            await pg.goto(f"file://{HERE}/manual_{lang}.html")
            await pg.wait_for_load_state("networkidle")
            await pg.pdf(path=str(HERE / name), format="A4", print_background=True, prefer_css_page_size=True)
        await b.close()
asyncio.run(main())
