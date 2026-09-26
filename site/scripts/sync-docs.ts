/**
 * Copies Markdown that lives elsewhere in the repository into the docs site, so there is one
 * source of truth: docs/architecture.md, docs/releasing.md and CHANGELOG.md.
 *
 * Each file's first `# Heading` becomes the page title, and links between those files are
 * rewritten to site URLs. The output is generated (and git-ignored): edit the originals.
 */
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

interface Source {
  from: string;
  to: string;
  description: string;
}

const siteDir = join(dirname(fileURLToPath(import.meta.url)), '..');
const repoDir = join(siteDir, '..');
const outDir = join(siteDir, 'src/content/docs/docs');
const repoUrl = 'https://github.com/singerbj/no-drama-llama/blob/main';

const sources: Source[] = [
  {
    from: 'docs/architecture.md',
    to: 'contributing/architecture.md',
    description: 'Threads, the state machine, GPU sizing, game detection, the install layout and how signed auto-update works.',
  },
  {
    from: 'docs/releasing.md',
    to: 'contributing/releasing.md',
    description: 'Set up the update signing key and cut a signed release.',
  },
  {
    from: 'CHANGELOG.md',
    to: 'reference/changelog.md',
    description: 'Every notable change to No Drama Llama.',
  },
];

/** Repository paths that have a page on the site. */
const pageFor: Record<string, string> = {
  'docs/architecture.md': '/docs/contributing/architecture/',
  'docs/releasing.md': '/docs/contributing/releasing/',
  'CHANGELOG.md': '/docs/reference/changelog/',
  'README.md': '/docs/',
};

function rewriteLinks(markdown: string, from: string): string {
  return markdown.replace(/\]\(([^)\s]+)\)/g, (match, target: string) => {
    if (/^(?:[a-z]+:|#|\/)/i.test(target)) return match;
    const [path, hash = ''] = target.split('#');
    const resolved = join(dirname(from), path).replaceAll('\\', '/');
    const page = pageFor[resolved];
    if (page) return `](${page}${hash ? `#${hash}` : ''})`;
    return `](${repoUrl}/${resolved}${hash ? `#${hash}` : ''})`;
  });
}

function toPage(markdown: string, source: Source): string {
  const heading = /^#\s+(.+)\r?\n/m.exec(markdown);
  if (!heading) throw new Error(`${source.from} has no "# Title" heading`);
  const body = rewriteLinks(markdown.slice(heading.index + heading[0].length).trimStart(), source.from);
  const frontmatter = [
    '---',
    `title: ${JSON.stringify(heading[1].trim())}`,
    `description: ${JSON.stringify(source.description)}`,
    'editUrl: ' + JSON.stringify(`https://github.com/singerbj/no-drama-llama/edit/main/${source.from}`),
    '---',
    '',
    `<!-- Generated from ${source.from} by site/scripts/sync-docs.ts. Edit that file instead. -->`,
    '',
  ].join('\n');
  return frontmatter + '\n' + body;
}

for (const source of sources) {
  const markdown = await readFile(join(repoDir, source.from), 'utf8');
  const target = join(outDir, source.to);
  await mkdir(dirname(target), { recursive: true });
  await writeFile(target, toPage(markdown, source));
  console.log(`synced ${source.from} -> ${target.slice(siteDir.length + 1)}`);
}
