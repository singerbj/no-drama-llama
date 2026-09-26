// Renders the favicon's llama (without its status dot) to icons/tray-llama.rgba: 32x32 raw
// RGBA that the tray app draws the status dot onto (src/win/tray.rs). Rerun it after changing
// site/public/favicon.svg:
//
//   NODE_PATH="$(npm root -g)" node scripts/tray-icon.ts     (needs `npm i -g playwright`)
import { readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const SIZE = 32; // tray::ICON_SIZE
const { chromium } = createRequire(import.meta.url)('playwright');

const favicon = readFileSync('site/public/favicon.svg', 'utf8');
// Drop the status dot and the pause bars on it; the tray draws its own dot.
const llama = favicon.replace(/\s*<circle[^>]*r="9"[^>]*\/>/, '').replace(/\s*<path stroke="#fff"[^>]*\/>/, '');
if (llama.length === favicon.length || /r="9"|#fff"/.test(llama)) {
  throw new Error('favicon.svg changed shape: update the dot-removal patterns');
}

const browser = await chromium.launch();
const page = await browser.newPage();
const pixels: number[] = await page.evaluate(
  async ({ svg, size }: { svg: string; size: number }) => {
    const img = new Image(size, size);
    img.src = `data:image/svg+xml;base64,${btoa(svg)}`;
    await img.decode();
    const ctx = Object.assign(document.createElement('canvas'), { width: size, height: size }).getContext('2d')!;
    ctx.drawImage(img, 0, 0, size, size);
    return Array.from(ctx.getImageData(0, 0, size, size).data);
  },
  { svg: llama, size: SIZE },
);
await browser.close();

writeFileSync('icons/tray-llama.rgba', Uint8Array.from(pixels));
console.log(`wrote icons/tray-llama.rgba (${pixels.length} bytes)`);
