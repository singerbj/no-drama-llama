import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { satteri } from '@astrojs/markdown-satteri';
import { baseLinks } from './src/plugins/base-links.ts';

// The site lives at the root of https://nodramallama.benjaminjsinger.com/ (a GitHub Pages custom
// domain). The Pages workflow passes the real origin and base path; these are the local defaults.
// An empty BASE_PATH means the site is at the root of its domain.
const site = process.env.SITE_URL || 'https://nodramallama.benjaminjsinger.com';
const base = process.env.BASE_PATH ?? '';

export default defineConfig({
  site,
  base: base || '/',
  trailingSlash: 'always',
  // The design system (tokens and fonts) lives in ../design, outside this package.
  vite: { server: { fs: { allow: ['.', '../design'] } } },
  markdown: {
    processor: satteri({ mdastPlugins: [baseLinks(base)] }),
  },
  integrations: [
    starlight({
      title: 'No Drama Llama',
      description:
        'An always-on local LLM server for Windows gaming PCs that gets out of the way when you play.',
      logo: { src: './src/assets/logo.svg', replacesTitle: false },
      favicon: '/favicon.svg',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/singerbj/no-drama-llama' },
      ],
      editLink: {
        baseUrl: 'https://github.com/singerbj/no-drama-llama/edit/main/site/',
      },
      lastUpdated: true,
      customCss: ['./src/styles/theme.css'],
      // Code panels are always dark (design system), in both themes.
      expressiveCode: {
        themes: ['vitesse-dark'],
        useStarlightUiThemeColors: false,
        styleOverrides: {
          borderRadius: '14px',
          borderColor: 'var(--border)',
          codeFontFamily: 'var(--font-mono)',
          uiFontFamily: 'var(--font-sans)',
          codeBackground: 'var(--code-bg)',
          frames: {
            editorBackground: 'var(--code-bg)',
            editorTabBarBackground: 'var(--code-bg)',
            editorActiveTabBackground: 'var(--code-bg)',
            editorActiveTabIndicatorTopColor: 'transparent',
            editorActiveTabIndicatorBottomColor: 'transparent',
            editorTabBarBorderBottomColor: 'var(--wool-700)',
            terminalBackground: 'var(--code-bg)',
            terminalTitlebarBackground: 'var(--code-bg)',
            terminalTitlebarBorderBottomColor: 'var(--wool-700)',
            frameBoxShadowCssValue: 'none',
          },
        },
      },
      sidebar: [
        {
          label: 'Getting started',
          items: [
            { label: 'Introduction', slug: 'docs' },
            { label: 'Installation', slug: 'docs/install' },
            { label: 'First steps', slug: 'docs/first-steps' },
          ],
        },
        {
          label: 'Guides',
          items: [
            { label: 'Chat and API', slug: 'docs/guides/api' },
            { label: 'Models and GPUs', slug: 'docs/guides/models' },
            { label: 'Game detection', slug: 'docs/guides/game-detection' },
            { label: 'Laya (decision model)', slug: 'docs/guides/laya' },
            { label: 'Use it from other devices', slug: 'docs/guides/network' },
            { label: 'Always-on PC', slug: 'docs/guides/always-on' },
            { label: 'Updates', slug: 'docs/guides/updates' },
            { label: 'Privacy', slug: 'docs/guides/privacy' },
            { label: 'Uninstall', slug: 'docs/guides/uninstall' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Tray menu', slug: 'docs/reference/tray-menu' },
            { label: 'Settings file', slug: 'docs/reference/settings' },
            { label: 'Command line', slug: 'docs/reference/cli' },
            { label: 'Troubleshooting', slug: 'docs/reference/troubleshooting' },
            { label: 'Changelog', slug: 'docs/reference/changelog' },
          ],
        },
        {
          label: 'Contributing',
          items: [
            { label: 'Development', slug: 'docs/contributing/development' },
            { label: 'Architecture', slug: 'docs/contributing/architecture' },
            { label: 'Releasing', slug: 'docs/contributing/releasing' },
          ],
        },
      ],
    }),
  ],
});
