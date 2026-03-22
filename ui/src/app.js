import { elements } from "./dom.js";
import { hasTauri, invoke, chooseDirectory } from "./backend.js";
import { loadLocale, tf } from "./i18n.js";
import {
  countUpdates,
  createState,
  pickSelectedPath,
  replaceInstalled,
  saveActiveTab,
  saveLanguage,
} from "./state.js";
import { renderApp } from "./render.js";
import { normalizeError } from "./utils.js";

const state = createState();
state.onOpenModal = openModal;
state.onOpenAvailableModal = openAvailableModal;
state.onInstallAvailable = installAvailable;
state.onRemoveSource = removeSource;

bindEvents();
init();

async function init() {
  await ensureLocale();
  await loadAppSettings();
  await loadAppInfo();
  render();
  if (state.rootPath) {
    await refreshAll();
  } else if (state.sources.length > 0) {
    await refreshAvailable();
  }
}

function bindEvents() {
  elements.rootPath.addEventListener("input", () => {
    state.rootPath = elements.rootPath.value.trim();
    render();
  });

  elements.sourceInput.addEventListener("input", () => {
    state.sourceInput = elements.sourceInput.value;
    render();
  });

  elements.languageSelect.addEventListener("change", async () => {
    state.language = elements.languageSelect.value;
    saveLanguage(state.language);
    await ensureLocale();
    render();
  });

  elements.chooseFolder.addEventListener("click", chooseFolderAction);
  elements.scanButton.addEventListener("click", refreshAll);
  elements.checkButton.addEventListener("click", refreshAll);
  elements.updateAllButton.addEventListener("click", updateAll);
  elements.updateBannerDismiss.addEventListener("click", dismissAppUpdate);
  elements.updateBannerInstall.addEventListener("click", installAppUpdate);
  elements.addSourceButton.addEventListener("click", addSource);
  elements.appUpdateButton.addEventListener("click", installAppUpdate);
  elements.modalUpdateButton.addEventListener("click", updateSelectedFromModal);
  elements.modalRollbackButton.addEventListener("click", rollbackSelectedFromModal);
  elements.modalInstallButton.addEventListener("click", requestInstallFromModal);
  elements.closeModal.addEventListener("click", closeModal);
  elements.installConfirmCancel.addEventListener("click", closeInstallPlan);
  elements.installConfirmAccept.addEventListener("click", confirmInstallPlan);
  elements.modal.addEventListener("click", (event) => {
    const openButton = event.target.closest("[data-open-external]");
    if (openButton) {
      event.preventDefault();
      void openExternalUrl(openButton.dataset.openExternal);
      return;
    }

    const link = event.target.closest("a[href]");
    if (link) {
      const url = link.getAttribute("href") || "";
      if (/^(https?:|mailto:)/i.test(url)) {
        event.preventDefault();
        void openExternalUrl(url);
        return;
      }
    }

    if (event.target.dataset.closeModal === "true") {
      closeModal();
    }
  });
  elements.modalTabButtons.forEach((button) => {
    button.addEventListener("click", () => {
      state.modalTab = button.dataset.modalTab;
      render();
    });
  });
  elements.tabButtons.forEach((button) => {
    button.addEventListener("click", () => {
      state.activeTab = button.dataset.tab;
      saveActiveTab(state.activeTab);
      render();
    });
  });

  window.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && state.modalOpen) {
      closeModal();
    }
  });
}

async function ensureLocale() {
  try {
    state.locale = await loadLocale(state.language);
  } catch {
    if (state.language !== "en") {
      state.language = "en";
      saveLanguage("en");
      state.locale = await loadLocale("en");
      return;
    }

    throw new Error("Failed to load locale");
  }
}

async function loadAppSettings() {
  if (!hasTauri()) {
    return;
  }

  try {
    applySettings(await invoke("load_settings"));
  } catch (error) {
    applySettings(null);
    setSourcesStatusText(normalizeError(error, state.locale));
  }
}

async function loadAppInfo() {
  if (!hasTauri()) {
    return;
  }

  try {
    const info = await invoke("load_app_info");
    state.appVersion = info.version ?? "";
    state.updaterEnabled = Boolean(info.updaterEnabled);
    state.appUpdate = info.update ?? null;
    state.appUpdateDismissed = false;
  } catch (error) {
    state.appVersion = "";
    state.updaterEnabled = false;
    state.appUpdate = null;
    setStatusText(normalizeError(error, state.locale));
  }
}

async function chooseFolderAction() {
  if (!hasTauri()) {
    setStatus("status.embeddedOnly");
    return;
  }

  const chosen = await chooseDirectory();
  if (!chosen) {
    return;
  }

  state.rootPath = Array.isArray(chosen) ? chosen[0] : chosen;
  await persistRootPath();
  await refreshAll();
}

async function refreshAll() {
  await refreshInstalled();
  await refreshAvailable();
}

async function refreshInstalled() {
  if (!state.rootPath) {
    setStatus("status.selectFolderFirst");
    return;
  }
  if (!hasTauri()) {
    setStatus("status.runtimeRequired");
    return;
  }

  try {
    setBusy(true);
    await persistRootPath();
    setStatus("status.scanning");
    const scanned = await invoke("scan_root", { rootPath: state.rootPath });
    state.installedAddons = scanned;
    state.selectedPath = pickSelectedPath(state);
    render();

    if (state.installedAddons.length === 0) {
      setStatus("status.nothingFound");
      return;
    }

    setStatus("status.checkingUpdates");
    const checked = await invoke("check_updates", { rootPath: state.rootPath });
    state.installedAddons = checked;
    state.selectedPath = pickSelectedPath(state);

    const updates = countUpdates(state.installedAddons);
    setStatusText(updates > 0 ? format("status.updatesFound", { count: updates }) : "");
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

async function refreshAvailable() {
  if (!state.rootPath || state.sources.length === 0 || !hasTauri()) {
    state.availableAddons = [];
    setSourcesStatusText("");
    render();
    return;
  }

  try {
    setBusy(true);
    setSourcesStatus("status.loadingSources");
    state.availableAddons = await invoke("list_available_addons", {
      rootPath: state.rootPath,
      sourceUrls: state.sources,
    });
    setSourcesStatusText(format("status.sourcesLoaded", { count: state.availableAddons.length }));
  } catch (error) {
    state.availableAddons = [];
    setSourcesStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

async function updateAll() {
  const updateTargets = state.installedAddons.filter((addon) => addon.hasUpdate);
  if (updateTargets.length === 0) {
    setStatus("status.noUpdatesToInstall");
    return;
  }
  if (!hasTauri()) {
    setStatus("status.runtimeRequired");
    return;
  }

  try {
    setBusy(true);
    for (let index = 0; index < updateTargets.length; index += 1) {
      const addon = updateTargets[index];
      setStatusText(
        format("status.updatingProgress", {
          current: index + 1,
          total: updateTargets.length,
          name: addon.name,
        }),
      );
      await invoke("update_addon", { addonPath: addon.addonPath });
    }

    await refreshInstalled();
    await refreshAvailable();
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

async function installAvailable(repositoryUrl, branch) {
  await openAvailableModal(repositoryUrl, branch);
}

async function requestInstallFromModal() {
  if (!state.rootPath) {
    setStatus("status.selectFolderFirst");
    return;
  }
  if (!hasTauri()) {
    setStatus("status.runtimeRequired");
    return;
  }

  const addon = state.modalAddon;
  if (!addon?.repositoryUrl || !addon?.branch) {
    return;
  }

  try {
    state.installPlanLoading = true;
    render();
    state.installPlan = await invoke("preview_install", {
      rootPath: state.rootPath,
      repositoryUrl: addon.repositoryUrl,
      branch: addon.branch,
    });
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    state.installPlanLoading = false;
    render();
  }
}

async function updateSelectedFromModal() {
  const selected = state.modalTargetType === "installed" ? state.modalAddon : null;
  if (!selected || !selected.hasUpdate || !hasTauri()) {
    return;
  }

  try {
    setBusy(true);
    setStatusText(format("status.updatingOne", { name: selected.name }));
    await invoke("update_addon", { addonPath: selected.addonPath });
    await refreshInstalled();
    await refreshAvailable();
    if (state.selectedPath) {
      await openModal(state.selectedPath);
    }
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

async function rollbackSelectedFromModal() {
  const selected = state.modalTargetType === "installed" ? state.modalAddon : null;
  if (!selected || !selected.addonPath || !hasTauri()) {
    return;
  }

  try {
    setBusy(true);
    setStatusText(format("status.rollbackOne", { name: selected.name }));
    await invoke("rollback_addon", { addonPath: selected.addonPath });
    await refreshInstalled();
    await refreshAvailable();
    if (state.selectedPath) {
      await openModal(state.selectedPath);
    }
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

async function addSource() {
  const normalized = state.sourceInput.trim();
  if (!normalized) {
    return;
  }
  if (state.sources.includes(normalized)) {
    setSourcesStatus("sources.exists");
    return;
  }

  state.sources = [...state.sources, normalized];
  state.sourceInput = "";
  await persistSources();
  setSourcesStatus("sources.added");
  await refreshAvailable();
}

async function removeSource(url) {
  state.sources = state.sources.filter((item) => item !== url);
  await persistSources();
  setSourcesStatus("sources.removed");
  await refreshAvailable();
}

async function openModal(addonPath) {
  const addon = state.installedAddons.find((item) => item.addonPath === addonPath);
  if (!addon) {
    return;
  }

  state.selectedPath = addonPath;
  state.modalOpen = true;
  state.modalTargetType = "installed";
  state.modalAddon = addon;
  resetModalState({ checking: true, readmeLoading: true });
  render();

  if (!hasTauri()) {
    state.modalChecking = false;
    render();
    return;
  }

  try {
    const [addonResult, readmeResult] = await Promise.allSettled([
      invoke("check_addon", { addonPath }),
      invoke("load_addon_readme", { addonPath }),
    ]);

    if (addonResult.status === "fulfilled") {
      state.modalAddon = addonResult.value;
      replaceInstalled(state, addonResult.value);
    } else {
      const index = state.installedAddons.findIndex((addon) => addon.addonPath === addonPath);
      if (index !== -1) {
        const fallbackAddon = {
          ...state.installedAddons[index],
          remoteVersion: null,
          hasUpdate: false,
          hasError: true,
          status: normalizeError(addonResult.reason, state.locale),
        };
        state.installedAddons[index] = fallbackAddon;
        state.modalAddon = fallbackAddon;
      }
    }

    if (readmeResult.status === "fulfilled") {
      state.modalReadme = readmeResult.value;
    } else {
      state.modalReadme = null;
    }
  } finally {
    state.modalChecking = false;
    state.modalReadmeLoading = false;
    render();
  }
}

async function openAvailableModal(repositoryUrl, branch) {
  const addon =
    state.availableAddons.find(
      (item) => item.repositoryUrl === repositoryUrl && item.branch === branch,
    ) ?? null;
  if (!addon) {
    return;
  }

  state.modalOpen = true;
  state.modalTargetType = "available";
  state.modalAddon = {
    ...addon,
    remoteVersion: addon.version,
    status: format("addons.availableState"),
  };
  resetModalState({ checking: true, readmeLoading: true });
  render();

  try {
    const [addonResult, readmeResult] = await Promise.allSettled([
      invoke("load_available_addon", {
        rootPath: state.rootPath,
        repositoryUrl,
        branch,
      }),
      invoke("load_available_addon_readme", {
        repositoryUrl,
        branch,
      }),
    ]);

    if (addonResult.status === "fulfilled") {
      state.modalAddon = {
        ...addonResult.value,
        remoteVersion: addonResult.value.version,
        status: addonResult.value.installed
          ? format("addons.installedState")
          : format("addons.availableState"),
      };
    } else {
      state.modalAddon = {
        ...state.modalAddon,
        status: normalizeError(addonResult.reason, state.locale),
      };
    }

    state.modalReadme = readmeResult.status === "fulfilled" ? readmeResult.value : null;
  } finally {
    state.modalChecking = false;
    state.modalReadmeLoading = false;
    render();
  }
}

function closeModal() {
  state.modalOpen = false;
  state.modalTargetType = null;
  state.modalAddon = null;
  resetModalState();
  render();
}

function resetModalState({ checking = false, readmeLoading = false } = {}) {
  state.modalChecking = checking;
  state.modalTab = "info";
  state.modalReadme = null;
  state.modalReadmeLoading = readmeLoading;
  state.installPlan = null;
  state.installPlanLoading = false;
}

function closeInstallPlan() {
  state.installPlan = null;
  render();
}

function dismissAppUpdate() {
  state.appUpdateDismissed = true;
  render();
}

async function installAppUpdate() {
  if (!hasTauri() || !state.appUpdate || state.appUpdating) {
    return;
  }

  try {
    state.appUpdating = true;
    setStatus("app.installing");
    render();
    const installed = await invoke("install_app_update");
    if (!installed) {
      state.appUpdate = null;
      state.appUpdateDismissed = false;
      setStatus("app.noUpdate");
    }
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    state.appUpdating = false;
    render();
  }
}

async function openExternalUrl(url) {
  if (!url) {
    return;
  }

  if (!hasTauri()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }

  try {
    await invoke("open_external_url", { url });
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  }
}

async function confirmInstallPlan() {
  const addon = state.modalAddon;
  if (!addon?.repositoryUrl || !addon?.branch || !hasTauri()) {
    return;
  }

  try {
    setBusy(true);
    setStatusText(format("status.installingAddon", { url: addon.repositoryUrl }));
    const installed = await invoke("install_addon", {
      rootPath: state.rootPath,
      repositoryUrl: addon.repositoryUrl,
      branch: addon.branch,
    });
    replaceInstalled(state, installed);
    closeInstallPlan();
    closeModal();
    await refreshInstalled();
    await refreshAvailable();
  } catch (error) {
    setStatusText(normalizeError(error, state.locale));
  } finally {
    setBusy(false);
  }
}

function setBusy(value) {
  state.busy = value;
  render();
}

function setStatus(path) {
  state.statusMessage = format(path);
  render();
}

function setStatusText(message) {
  state.statusMessage = message;
  render();
}

function setSourcesStatus(path) {
  state.sourcesStatusMessage = format(path);
  render();
}

function setSourcesStatusText(message) {
  state.sourcesStatusMessage = message;
  render();
}

function format(path, vars = {}) {
  return tf(state.locale, path, vars);
}

function render() {
  renderApp(state, elements);
}

async function persistRootPath() {
  if (!hasTauri()) {
    return;
  }

  applySettings(await invoke("save_root_path", { rootPath: state.rootPath }));
}

async function persistSources() {
  if (!hasTauri()) {
    return;
  }

  applySettings(await invoke("save_sources", { sources: state.sources }));
}

function applySettings(settings) {
  state.rootPath = settings?.rootPath ?? "";
  state.sources = Array.isArray(settings?.sources) ? settings.sources : [];
}
