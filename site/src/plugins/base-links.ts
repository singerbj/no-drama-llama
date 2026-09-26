import { defineMdastPlugin } from 'satteri';

/**
 * Prefixes root-relative Markdown links (`/docs/install/`) with the site's base path, so
 * content can link to other pages without knowing where the site is deployed.
 */
export function baseLinks(base: string) {
  const prefix = base.replace(/\/+$/, '');
  const withBase = (url: string) =>
    prefix && url.startsWith('/') && !url.startsWith('//') && !url.startsWith(`${prefix}/`)
      ? prefix + url
      : url;
  return defineMdastPlugin({
    name: 'base-links',
    link(node, ctx) {
      const url = withBase(node.url);
      if (url !== node.url) ctx.setProperty(node, 'url', url);
    },
    definition(node, ctx) {
      const url = withBase(node.url);
      if (url !== node.url) ctx.setProperty(node, 'url', url);
    },
  });
}
