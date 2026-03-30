import DOMPurify from "dompurify";
import { marked } from "marked";

marked.setOptions({
  gfm: true,
  breaks: true,
});

export function renderMarkdown(input, options = {}) {
  const rendered = marked.parse(String(input ?? ""));
  const sanitized = DOMPurify.sanitize(rendered);
  const documentFragment = new DOMParser().parseFromString(
    `<article class="markdown-body">${sanitized}</article>`,
    "text/html",
  );

  documentFragment.querySelectorAll("a[href]").forEach((link) => {
    link.setAttribute("href", resolveMarkdownUrl(link.getAttribute("href"), options, false));
    link.setAttribute("target", "_blank");
    link.setAttribute("rel", "noreferrer");
  });

  documentFragment.querySelectorAll("img[src]").forEach((image) => {
    image.setAttribute("src", resolveMarkdownUrl(image.getAttribute("src"), options, true));
    image.setAttribute("loading", "lazy");
  });

  return documentFragment.body.innerHTML;
}

function resolveMarkdownUrl(url, options, isImage) {
  if (!url) {
    return "";
  }

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
