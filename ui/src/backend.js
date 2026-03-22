function tauri() {
  return window.__TAURI__;
}

export function hasTauri() {
  return Boolean(tauri()?.core?.invoke);
}

export async function invoke(command, payload) {
  return tauri().core.invoke(command, payload);
}

export async function chooseDirectory() {
  return tauri().dialog.open({
    directory: true,
    multiple: false,
  });
}
