const DEFAULT_TAB = "addons";

function loadActiveTab() {
  const value = localStorage.getItem("gluamanager.activeTab");
  return value === "settings" ? "settings" : DEFAULT_TAB;
}

export function createState() {
  return {
    rootPath: "",
    installedAddons: [],
    availableAddons: [],
    selectedPath: null,
    busy: false,
    language: localStorage.getItem("gluamanager.language") ?? "en",
    locale: null,
    activeTab: loadActiveTab(),
    appVersion: "",
    updaterEnabled: false,
    appUpdate: null,
    appUpdateDismissed: false,
    appUpdating: false,
    modalOpen: false,
    modalTargetType: null,
    modalAddon: null,
    modalChecking: false,
    modalTab: "info",
    modalReadme: null,
    modalReadmeLoading: false,
    installPlan: null,
    installPlanLoading: false,
    sources: [],
    sourceInput: "",
    statusMessage: "",
    sourcesStatusMessage: "",
  };
}

export function saveLanguage(language) {
  localStorage.setItem("gluamanager.language", language);
}

export function saveActiveTab(tab) {
  localStorage.setItem("gluamanager.activeTab", tab);
}

export function countUpdates(installedAddons) {
  return installedAddons.filter((addon) => addon.hasUpdate).length;
}

export function replaceInstalled(state, updated) {
  const index = state.installedAddons.findIndex((addon) => addon.addonPath === updated.addonPath);
  if (index !== -1) {
    state.installedAddons[index] = updated;
  } else {
    state.installedAddons.push(updated);
    state.installedAddons.sort((left, right) => left.name.localeCompare(right.name));
  }

  state.selectedPath = updated.addonPath;
}

export function selectedInstalledAddon(state) {
  if (state.modalAddon && state.modalTargetType === "installed") {
    return state.modalAddon;
  }
  return state.installedAddons.find((addon) => addon.addonPath === state.selectedPath) ?? null;
}

export function pickSelectedPath(state) {
  if (
    state.selectedPath &&
    state.installedAddons.some((addon) => addon.addonPath === state.selectedPath)
  ) {
    return state.selectedPath;
  }

  return state.installedAddons[0]?.addonPath ?? null;
}
