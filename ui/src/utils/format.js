import { messages } from "../services/messages.js";

export function resolveLocale(language) {
  return messages[language] ?? messages.en;
}

export function t(locale, path) {
  return path.split(".").reduce((value, key) => value?.[key], locale) ?? path;
}

export function tf(locale, path, vars = {}) {
  let text = t(locale, path);
  Object.entries(vars).forEach(([key, value]) => {
    text = text.replaceAll(`{${key}}`, String(value));
  });
  return text;
}

export function normalizeError(error, locale) {
  if (typeof error === "string") {
    return error;
  }
  if (error?.message) {
    return error.message;
  }
  return t(locale, "status.genericError");
}

export function externalHref(url) {
  if (!url) {
    return "";
  }
  if (/^https?:\/\//i.test(url)) {
    return url;
  }
  return "";
}

export function isHttpUrl(url) {
  if (!url) {
    return false;
  }

  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export function initials(value) {
  return String(value ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");
}
