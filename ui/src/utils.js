import { t } from "./i18n.js";

export function normalizeError(error, locale) {
  if (typeof error === "string") {
    return error;
  }
  if (error?.message) {
    return error.message;
  }
  return t(locale, "status.genericError");
}

export function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function renderMarkdown(input, options = {}) {
  const lines = String(input ?? "").replaceAll("\r\n", "\n").split("\n");
  const blocks = [];
  let paragraph = [];
  let unorderedItems = [];
  let orderedItems = [];
  let codeFence = null;
  let tableLines = [];

  function flushParagraph() {
    if (paragraph.length === 0) {
      return;
    }
    blocks.push(`<p>${renderInline(paragraph.join(" "), options)}</p>`);
    paragraph = [];
  }

  function flushUnorderedList() {
    if (unorderedItems.length === 0) {
      return;
    }
    blocks.push(
      `<ul>${unorderedItems.map((item) => `<li>${renderInline(item, options)}</li>`).join("")}</ul>`,
    );
    unorderedItems = [];
  }

  function flushOrderedList() {
    if (orderedItems.length === 0) {
      return;
    }
    blocks.push(
      `<ol>${orderedItems.map((item) => `<li>${renderInline(item, options)}</li>`).join("")}</ol>`,
    );
    orderedItems = [];
  }

  function flushTable() {
    if (tableLines.length < 2) {
      tableLines = [];
      return;
    }

    const rows = tableLines.map(parseTableRow).filter((row) => row.length > 0);
    if (rows.length < 2 || !isTableDivider(rows[1])) {
      const fallbackLines = [...tableLines];
      tableLines = [];
      flushUnorderedList();
      flushOrderedList();
      paragraph.push(...fallbackLines);
      return;
    }

    const header = rows[0];
    const body = rows.slice(2);
    blocks.push(
      `<table><thead><tr>${header.map((cell) => `<th>${renderInline(cell, options)}</th>`).join("")}</tr></thead><tbody>${body
        .map((row) => `<tr>${row.map((cell) => `<td>${renderInline(cell, options)}</td>`).join("")}</tr>`)
        .join("")}</tbody></table>`,
    );
    tableLines = [];
  }

  function pushCodeBlock(lines) {
    const codeLines = [...lines];
    if (codeLines[0] && /^[A-Za-z0-9_+-]+$/.test(codeLines[0])) {
      codeLines.shift();
    }
    blocks.push(`<pre class="readme-pre"><code>${escapeHtml(codeLines.join("\n"))}</code></pre>`);
  }

  for (const line of lines) {
    if (codeFence) {
      if (line.startsWith("```")) {
        pushCodeBlock(codeFence);
        codeFence = null;
      } else {
        codeFence.push(line);
      }
      continue;
    }

    if (line.startsWith("```")) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      flushTable();
      codeFence = [line.slice(3).trim()].filter(Boolean);
      continue;
    }

    const trimmed = line.trim();
    if (!trimmed) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      flushTable();
      continue;
    }

    const headingMatch = trimmed.match(/^(#{1,6})\s+(.*)$/);
    if (headingMatch) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      flushTable();
      const level = headingMatch[1].length;
      blocks.push(`<h${level}>${renderInline(headingMatch[2], options)}</h${level}>`);
      continue;
    }

    if (/^(-{3,}|\*{3,}|_{3,})$/.test(trimmed)) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      flushTable();
      blocks.push("<hr>");
      continue;
    }

    const quoteMatch = trimmed.match(/^>\s?(.*)$/);
    if (quoteMatch) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      flushTable();
      blocks.push(`<blockquote>${renderInline(quoteMatch[1], options)}</blockquote>`);
      continue;
    }

    const unorderedMatch = trimmed.match(/^[-*]\s+(.*)$/);
    if (unorderedMatch) {
      flushParagraph();
      flushOrderedList();
      flushTable();
      unorderedItems.push(unorderedMatch[1]);
      continue;
    }

    const orderedMatch = trimmed.match(/^\d+\.\s+(.*)$/);
    if (orderedMatch) {
      flushParagraph();
      flushUnorderedList();
      flushTable();
      orderedItems.push(orderedMatch[1]);
      continue;
    }

    if (trimmed.includes("|")) {
      flushParagraph();
      flushUnorderedList();
      flushOrderedList();
      tableLines.push(trimmed);
      continue;
    }

    flushTable();
    flushUnorderedList();
    flushOrderedList();
    paragraph.push(trimmed);
  }

  flushParagraph();
  flushUnorderedList();
  flushOrderedList();
  flushTable();

  if (codeFence) {
    pushCodeBlock(codeFence);
  }

  return `<article class="markdown-body">${blocks.join("")}</article>`;
}

function renderInline(text, options) {
  let html = escapeHtml(text);
  html = html.replace(/!\[([^\]]*)\]\((.+?)\)/g, (_match, alt, url) => {
    const resolved = resolveMarkdownUrl(url, options, true);
    return `<img src="${escapeHtml(resolved)}" alt="${escapeHtml(alt)}">`;
  });
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
  html = html.replace(
    /\[([^\]]+)\]\((.+?)\)/g,
    (_match, label, url) => {
      const resolved = resolveMarkdownUrl(url, options, false);
      return `<a href="${escapeHtml(resolved)}" target="_blank" rel="noreferrer">${escapeHtml(label)}</a>`;
    },
  );
  html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/__([^_]+)__/g, "<strong>$1</strong>");
  html = html.replace(/~~([^~]+)~~/g, "<del>$1</del>");
  html = html.replace(/(^|[^\*])\*([^*]+)\*/g, "$1<em>$2</em>");
  html = html.replace(/(^|[^_])_([^_]+)_/g, "$1<em>$2</em>");
  return html;
}

function resolveMarkdownUrl(url, options, isImage) {
  if (/^(https?:|data:|blob:|mailto:|#)/i.test(url)) {
    return url;
  }

  if (options.baseUrl) {
    try {
      return new URL(url, options.baseUrl).href;
    } catch {
      return url;
    }
  }

  if (options.localBasePath) {
    const fullPath = joinLocalPath(options.localBasePath, url);
    if (isImage) {
      const converted = window.__TAURI__?.core?.convertFileSrc?.(fullPath);
      return converted || fullPath;
    }
    return fullPath;
  }

  return url;
}

function joinLocalPath(basePath, relativePath) {
  const normalizedBase = String(basePath).replace(/[\\\/]+$/, "");
  const normalizedRelative = String(relativePath).replace(/^\.?[\\\/]+/, "");
  return `${normalizedBase}/${normalizedRelative}`.replaceAll("\\", "/");
}

function parseTableRow(line) {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

function isTableDivider(row) {
  return row.every((cell) => /^:?-{3,}:?$/.test(cell));
}

export function repositoryHref(repositoryUrl) {
  const value = String(repositoryUrl || "").trim().replace(/^\/+|\/+$/g, "");
  if (!value) {
    return "";
  }
  return `https://github.com/${value}`;
}
