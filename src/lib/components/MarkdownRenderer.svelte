<script lang="ts">
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import hljs from 'highlight.js/lib/core';

  // Register common languages for code highlighting
  import javascript from 'highlight.js/lib/languages/javascript';
  import typescript from 'highlight.js/lib/languages/typescript';
  import python from 'highlight.js/lib/languages/python';
  import json from 'highlight.js/lib/languages/json';
  import bash from 'highlight.js/lib/languages/bash';
  import css from 'highlight.js/lib/languages/css';
  import xml from 'highlight.js/lib/languages/xml';
  import markdown from 'highlight.js/lib/languages/markdown';
  import sql from 'highlight.js/lib/languages/sql';
  import yaml from 'highlight.js/lib/languages/yaml';
  import rust from 'highlight.js/lib/languages/rust';
  import csharp from 'highlight.js/lib/languages/csharp';

  hljs.registerLanguage('javascript', javascript);
  hljs.registerLanguage('js', javascript);
  hljs.registerLanguage('typescript', typescript);
  hljs.registerLanguage('ts', typescript);
  hljs.registerLanguage('python', python);
  hljs.registerLanguage('json', json);
  hljs.registerLanguage('bash', bash);
  hljs.registerLanguage('shell', bash);
  hljs.registerLanguage('css', css);
  hljs.registerLanguage('html', xml);
  hljs.registerLanguage('xml', xml);
  hljs.registerLanguage('markdown', markdown);
  hljs.registerLanguage('md', markdown);
  hljs.registerLanguage('sql', sql);
  hljs.registerLanguage('yaml', yaml);
  hljs.registerLanguage('yml', yaml);
  hljs.registerLanguage('rust', rust);
  hljs.registerLanguage('rs', rust);
  hljs.registerLanguage('csharp', csharp);
  hljs.registerLanguage('cs', csharp);

  let { content, isUser = false }: { content: string; isUser?: boolean } = $props();

  // Configure marked
  const renderer = new marked.Renderer();

  // Custom code block renderer with syntax highlighting + copy button
  renderer.code = function ({ text, lang}: { text: string; lang?: string }) {
    const language = lang && hljs.getLanguage(lang) ? lang : 'plaintext';
    let highlighted: string;
    try {
      highlighted = lang && hljs.getLanguage(lang)
        ? hljs.highlight(text, { language }).value
        : escapeHtml(text);
    } catch {
      highlighted = escapeHtml(text);
    }
    const langLabel = lang || 'code';
    return `<div class="code-block">
      <div class="code-header">
        <span class="code-lang">${escapeHtml(langLabel)}</span>
        <button class="code-copy-btn" data-code="${escapeAttr(text)}">Copy</button>
      </div>
      <pre><code class="hljs language-${escapeHtml(language)}">${highlighted}</code></pre>
    </div>`;
  };

  // Inline code
  renderer.codespan = function ({ text }: { text: string }) {
    return `<code class="inline-code">${text}</code>`;
  };

  // Tables
  renderer.table = function (token: any) {
    const header = token.header ?? '';
    const body = token.body ?? token.rows ?? '';
    return `<div class="table-wrapper"><table><thead>${header}</thead><tbody>${body}</tbody></table></div>`;
  };

  // Links open in default browser
  renderer.link = function ({ href, title, text }: { href: string; title?: string | null; text: string }) {
    const titleAttr = title ? ` title="${escapeAttr(title)}"` : '';
    return `<a href="${escapeAttr(href)}"${titleAttr} target="_blank" rel="noopener noreferrer">${text}</a>`;
  };

  marked.setOptions({
    renderer,
    gfm: true,
    breaks: true,
  });

  function escapeHtml(str: string): string {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  function escapeAttr(str: string): string {
    return str.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/'/g, '&#39;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  let renderedHtml = $derived.by(() => {
    if (isUser) {
      // User messages: just escape and add line breaks
      return escapeHtml(content).replace(/\n/g, '<br>');
    }
    // AI messages: full markdown rendering
    const raw = marked.parse(content) as string;
    return DOMPurify.sanitize(raw, {
      ADD_TAGS: ['button'],
      ADD_ATTR: ['data-code', 'target', 'rel'],
    });
  });

  function handleClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.classList.contains('code-copy-btn')) {
      const code = target.getAttribute('data-code');
      if (code) {
        navigator.clipboard.writeText(code);
        target.textContent = 'Copied!';
        setTimeout(() => { target.textContent = 'Copy'; }, 2000);
      }
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="md-content" class:md-user={isUser} onclick={handleClick}>
  {@html renderedHtml}
</div>

<style>
  .md-content {
    line-height: 1.65;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }
  .md-content :global(p) {
    margin: 0 0 0.5em;
  }
  .md-content :global(p:last-child) {
    margin-bottom: 0;
  }
  .md-content :global(ul),
  .md-content :global(ol) {
    margin: 0.4em 0;
    padding-left: 1.5em;
  }
  .md-content :global(li) {
    margin: 0.2em 0;
  }
  .md-content :global(blockquote) {
    margin: 0.6em 0;
    padding: 0.4em 0.8em;
    border-left: 3px solid var(--brand);
    background: var(--bg-3);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    color: var(--text-2);
  }
  .md-content :global(hr) {
    border: none;
    border-top: 1px solid var(--border);
    margin: 0.8em 0;
  }
  .md-content :global(h1),
  .md-content :global(h2),
  .md-content :global(h3),
  .md-content :global(h4) {
    margin: 0.8em 0 0.3em;
    font-weight: 600;
    line-height: 1.3;
  }
  .md-content :global(h1) { font-size: 1.2em; }
  .md-content :global(h2) { font-size: 1.1em; }
  .md-content :global(h3) { font-size: 1.0em; }
  .md-content :global(strong) { font-weight: 600; }
  .md-content :global(a) {
    color: var(--brand);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  .md-content :global(a:hover) {
    opacity: 0.8;
  }

  /* Inline code */
  .md-content :global(.inline-code) {
    background: var(--bg-3);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: 0.88em;
    font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
    border: 1px solid var(--border);
  }

  /* Code blocks */
  .md-content :global(.code-block) {
    margin: 0.6em 0;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    overflow: hidden;
    background: var(--bg-0);
  }
  .md-content :global(.code-header) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: var(--bg-3);
    border-bottom: 1px solid var(--border);
    font-size: 0.7rem;
  }
  .md-content :global(.code-lang) {
    color: var(--text-3);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .md-content :global(.code-copy-btn) {
    font-size: 0.68rem;
    padding: 2px 8px;
    border-radius: 4px;
    background: var(--bg-hover);
    border: 1px solid var(--border);
    color: var(--text-2);
    cursor: pointer;
    transition: all 0.1s;
    font-family: inherit;
  }
  .md-content :global(.code-copy-btn:hover) {
    background: var(--bg-selected);
    color: var(--brand);
  }
  .md-content :global(pre) {
    margin: 0;
    padding: 12px 14px;
    overflow-x: auto;
    font-size: 0.82rem;
    line-height: 1.55;
  }
  .md-content :global(code) {
    font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
  }

  /* Tables */
  .md-content :global(.table-wrapper) {
    margin: 0.6em 0;
    overflow-x: auto;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }
  .md-content :global(table) {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }
  .md-content :global(th) {
    text-align: left;
    padding: 8px 12px;
    background: var(--bg-3);
    border-bottom: 1px solid var(--border);
    font-weight: 600;
    font-size: 0.75rem;
    color: var(--text-2);
  }
  .md-content :global(td) {
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
  }
  .md-content :global(tr:last-child td) {
    border-bottom: none;
  }

  /* highlight.js token colors */
  .md-content :global(.hljs-keyword) { color: #c678dd; }
  .md-content :global(.hljs-string) { color: #98c379; }
  .md-content :global(.hljs-number) { color: #d19a66; }
  .md-content :global(.hljs-built_in) { color: #e6c07b; }
  .md-content :global(.hljs-comment) { color: #5c6370; font-style: italic; }
  .md-content :global(.hljs-function) { color: #61afef; }
  .md-content :global(.hljs-title) { color: #61afef; }
  .md-content :global(.hljs-params) { color: #abb2bf; }
  .md-content :global(.hljs-attr) { color: #d19a66; }
  .md-content :global(.hljs-type) { color: #e6c07b; }
  .md-content :global(.hljs-meta) { color: #56b6c2; }
  .md-content :global(.hljs-literal) { color: #56b6c2; }
  .md-content :global(.hljs-symbol) { color: #61afef; }
  .md-content :global(.hljs-selector-class) { color: #d19a66; }
  .md-content :global(.hljs-selector-tag) { color: #e06c75; }
  .md-content :global(.hljs-template-variable) { color: #e06c75; }
  .md-content :global(.hljs-variable) { color: #e06c75; }
  .md-content :global(.hljs-addition) { color: #98c379; }
  .md-content :global(.hljs-deletion) { color: #e06c75; }

  /* Light theme token overrides */
  :global([data-theme="light"]) .md-content :global(.hljs-keyword) { color: #a626a4; }
  :global([data-theme="light"]) .md-content :global(.hljs-string) { color: #50a14f; }
  :global([data-theme="light"]) .md-content :global(.hljs-number) { color: #986801; }
  :global([data-theme="light"]) .md-content :global(.hljs-built_in) { color: #c18401; }
  :global([data-theme="light"]) .md-content :global(.hljs-comment) { color: #a0a1a7; font-style: italic; }
  :global([data-theme="light"]) .md-content :global(.hljs-function) { color: #4078f2; }
  :global([data-theme="light"]) .md-content :global(.hljs-title) { color: #4078f2; }
  :global([data-theme="light"]) .md-content :global(.hljs-type) { color: #c18401; }
  :global([data-theme="light"]) .md-content :global(.hljs-selector-tag) { color: #e45649; }
  :global([data-theme="light"]) .md-content :global(.hljs-variable) { color: #e45649; }

  /* User messages: keep it simple */
  .md-user {
    white-space: pre-wrap;
  }
</style>
