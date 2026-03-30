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

export function repositoryHref(repositoryUrl) {
  if (!repositoryUrl) {
    return "";
  }
  if (/^https?:\/\//i.test(repositoryUrl)) {
    return repositoryUrl;
  }
  if (/^[\w.-]+\/[\w.-]+$/.test(repositoryUrl)) {
    return `https://github.com/${repositoryUrl}`;
  }
  return "";
}

export function initials(value) {
  return String(value ?? "")
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");
}
