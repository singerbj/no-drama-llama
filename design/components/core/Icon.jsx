import React from 'react';
/** No Drama Llama's own line icons (assets/icons/*.svg: 24px grid, stroke 1.8, round caps), inlined
    so they take any token colour through currentColor. Keep this map in step with that folder. */
const ICONS = {
  "api": "<path d=\"m8 9-4 3 4 3M16 9l4 3-4 3M13.5 6l-3 12\"/>",
  "box": "<path d=\"M21 8 12 3 3 8v8l9 5 9-5V8Z\"/><path d=\"m3 8 9 5 9-5M12 13v8\"/>",
  "chevron-down": "<path d=\"m7 10 5 5 5-5\"/>",
  "eye": "<path d=\"M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>",
  "gpu": "<rect x=\"3\" y=\"6\" width=\"18\" height=\"12\" rx=\"2\"/><circle cx=\"9\" cy=\"12\" r=\"2.5\"/><circle cx=\"15.5\" cy=\"12\" r=\"2.5\"/><path d=\"M6 18v2M18 18v2M3 10H1M3 14H1\"/>",
  "key": "<rect x=\"2\" y=\"7\" width=\"20\" height=\"10\" rx=\"2\"/><path d=\"M6 11h.01M10 11h.01M14 11h.01M18 11h.01M7 14h10\"/>",
  "moon": "<path d=\"M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5Z\"/>",
  "shield": "<path d=\"M12 3 4 6v6c0 5 3.5 8 8 9 4.5-1 8-4 8-9V6l-8-3Z\"/><path d=\"m9 12 2 2 4-4\"/>",
  "theme": "<path d=\"M12 3a9 9 0 1 0 0 18V3Z\" fill=\"currentColor\" stroke=\"none\"/><circle cx=\"12\" cy=\"12\" r=\"9\"/>",
  "tray": "<rect x=\"3\" y=\"4\" width=\"18\" height=\"16\" rx=\"2\"/><path d=\"M3 15h18\"/><circle cx=\"16.5\" cy=\"17.5\" r=\"1\" fill=\"currentColor\"/>",
  "windows": "<path d=\"M3 5.5 10.5 4.5v7H3v-6Zm0 13 7.5 1v-7H3v6Zm8.5 1.1L21 21v-8.5h-9.5v7.1Zm0-15.2v7.1H21V3l-9.5 1.4Z\" fill=\"currentColor\" stroke=\"none\"/>"
};
export function Icon({ name, size = 18, color = 'currentColor', style }) {
  return <svg aria-hidden="true" viewBox="0 0 24 24" width={size} height={size} fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" style={{ display: 'inline-block', flex: 'none', color, ...style }} dangerouslySetInnerHTML={{ __html: ICONS[name] || '' }} />;
}
