import React from 'react';
/** Lucide glyph (CDN, lucide-static) painted with currentColor via CSS mask, so it takes any token colour. */
export function Icon({ name, size = 18, color = 'currentColor', style }) {
  const url = 'url(https://unpkg.com/lucide-static@0.460.0/icons/' + name + '.svg)';
  return <span aria-hidden="true" style={{ display: 'inline-block', flex: 'none', width: size, height: size, background: color, WebkitMask: url + ' center / contain no-repeat', mask: url + ' center / contain no-repeat', ...style }} />;
}
