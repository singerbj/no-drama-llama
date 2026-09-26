// Builds every icon from icons/logo.svg. Rerun it after changing the logo:
//
//   NODE_PATH="$(npm root -g)" node scripts/icons.ts     (needs `npm i -g playwright`)
//
// Writes:
//   site/public/favicon.svg, site/src/assets/logo.svg   copies of the logo
//   icons/icon.ico                                      the exe and settings window icon
//   icons/tray-logo.rgba   32x32 RGBA: the logo without its circle
//   icons/tray-disc.a      32x32 alpha: the circle, which the tray fills with the status color
import { copyFileSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const LOGO = 'icons/logo.svg';
const TRAY = 32; // tray::ICON_SIZE
const ICO_SIZES = [16, 24, 32, 48, 64, 256];
const { chromium } = createRequire(import.meta.url)('playwright');

copyFileSync(LOGO, 'site/public/favicon.svg');
copyFileSync(LOGO, 'site/src/assets/logo.svg');

const browser = await chromium.launch();
const page = await browser.newPage();
const out: { rgba: Record<string, number[]>; png: Record<number, string> } = await page.evaluate(
  async ({ svg, tray, sizes }: { svg: string; tray: number; sizes: number[] }) => {
    // `part`: 'all', 'no-disc' (circle hidden) or 'disc' (only the circle).
    const variant = (part: string) => {
      const doc = new DOMParser().parseFromString(svg, 'image/svg+xml');
      const disc = doc.getElementById('status')!;
      if (part === 'no-disc') disc.setAttribute('fill', 'none');
      if (part === 'disc') {
        for (const el of [...doc.documentElement.children]) if (el !== disc) el.remove();
      }
      return new XMLSerializer().serializeToString(doc);
    };
    const draw = async (src: string, size: number) => {
      const img = new Image(size, size);
      img.src = `data:image/svg+xml;base64,${btoa(src)}`;
      await img.decode();
      const canvas = Object.assign(document.createElement('canvas'), { width: size, height: size });
      const ctx = canvas.getContext('2d')!;
      ctx.drawImage(img, 0, 0, size, size);
      return { canvas, ctx };
    };
    const rgba = async (part: string) => {
      const { ctx } = await draw(variant(part), tray);
      return Array.from(ctx.getImageData(0, 0, tray, tray).data);
    };
    const png: Record<number, string> = {};
    for (const s of sizes) png[s] = (await draw(svg, s)).canvas.toDataURL('image/png').split(',')[1];
    return { rgba: { logo: await rgba('no-disc'), disc: await rgba('disc') }, png };
  },
  { svg: readFileSync(LOGO, 'utf8'), tray: TRAY, sizes: ICO_SIZES },
);
await browser.close();

writeFileSync('icons/tray-logo.rgba', Uint8Array.from(out.rgba.logo));
writeFileSync('icons/tray-disc.a', Uint8Array.from(out.rgba.disc.filter((_, i) => i % 4 === 3)));

// ICO with PNG entries (Windows Vista and later).
const pngs = ICO_SIZES.map((s) => Buffer.from(out.png[s], 'base64'));
const header = Buffer.alloc(6 + 16 * pngs.length);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(pngs.length, 4);
let offset = header.length;
pngs.forEach((png, i) => {
  const e = 6 + 16 * i;
  const s = ICO_SIZES[i];
  header.writeUInt8(s % 256, e); // 256 is stored as 0
  header.writeUInt8(s % 256, e + 1);
  header.writeUInt16LE(1, e + 4); // planes
  header.writeUInt16LE(32, e + 6); // bits per pixel
  header.writeUInt32LE(png.length, e + 8);
  header.writeUInt32LE(offset, e + 12);
  offset += png.length;
});
writeFileSync('icons/icon.ico', Buffer.concat([header, ...pngs]));
console.log('wrote favicon.svg, logo.svg, icon.ico, tray-logo.rgba, tray-disc.a');
