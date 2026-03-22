export async function loadLocale(language) {
  const response = await fetch(`./locales/${language}.json`);
  if (!response.ok) {
    throw new Error(`Failed to load locale ${language}`);
  }
  return response.json();
}

export function t(locale, path) {
  return path.split(".").reduce((value, key) => value?.[key], locale) ?? path;
}

export function tf(locale, path, vars) {
  let text = t(locale, path);
  for (const [key, value] of Object.entries(vars)) {
    text = text.replaceAll(`{${key}}`, String(value));
  }
  return text;
}
