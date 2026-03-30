import { computed, inject, reactive } from "vue";

import { chooseDirectory, hasTauri, invoke } from "../services/tauri.js";
import { normalizeError, repositoryHref, resolveLocale, t, tf } from "../utils/format.js";

export const appStoreKey = Symbol("app-store");

const DEFAULT_LANGUAGE = localStorage.getItem("gluamanager.language") ?? "en";

function loadSavedThemeState() {
  return {
    language: DEFAULT_LANGUAGE,
  };
}

function createAvailableModalAddon(addon, translate) {
  return {
    ...addon,
    remoteVersion: addon.version,
    status: addon.installed ? translate("addons.installedState") : translate("addons.availableState"),
  };
}

export function createAppStore() {
  const state = reactive({
    initialized: false,
    busy: false,
    language: loadSavedThemeState().language,
    rootPath: "",
    installedAddons: [],
    availableAddons: [],
    sources: [],
    sourceInput: "",
    statusMessage: "",
    sourcesStatusMessage: "",
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
  });

  const locale = computed(() => resolveLocale(state.language));
  const updateCount = computed(() => state.installedAddons.filter((addon) => addon.hasUpdate).length);

  function translate(path) {
    return t(locale.value, path);
  }

  function translateFormat(path, vars = {}) {
    return tf(locale.value, path, vars);
  }

  function setBusy(value) {
    state.busy = value;
  }

  function setStatus(path, vars = {}) {
    state.statusMessage = translateFormat(path, vars);
  }

  function setStatusText(value) {
    state.statusMessage = value;
  }

  function setSourcesStatus(path, vars = {}) {
    state.sourcesStatusMessage = translateFormat(path, vars);
  }

  function setSourcesStatusText(value) {
    state.sourcesStatusMessage = value;
  }

  function persistLanguage() {
    localStorage.setItem("gluamanager.language", state.language);
  }

  function applySettings(settings) {
    state.rootPath = settings?.rootPath ?? "";
    state.sources = Array.isArray(settings?.sources) ? settings.sources : [];
  }

  async function initialize() {
    if (state.initialized) {
      return;
    }

    if (hasTauri()) {
      try {
        applySettings(await invoke("load_settings"));
      } catch (error) {
        setSourcesStatusText(normalizeError(error, locale.value));
      }

      try {
        const info = await invoke("load_app_info");
        state.appVersion = info.version ?? "";
        state.updaterEnabled = Boolean(info.updaterEnabled);
        state.appUpdate = info.update ?? null;
        state.appUpdateDismissed = false;
      } catch (error) {
        setStatusText(normalizeError(error, locale.value));
      }
    }

    if (state.rootPath) {
      await refreshAll();
    } else if (state.sources.length > 0) {
      await refreshAvailable();
    } else {
      setStatus("status.ready");
    }

    state.initialized = true;
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

      if (state.installedAddons.length === 0) {
        setStatus("status.nothingFound");
        return;
      }

      setStatus("status.checkingUpdates");
      const checked = await invoke("check_updates", { rootPath: state.rootPath });
      state.installedAddons = checked;

      if (updateCount.value > 0) {
        setStatus("status.updatesFound", { count: updateCount.value });
      } else {
        setStatus("status.ready");
      }
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  async function refreshAvailable() {
    if (!state.rootPath || !state.sources.length || !hasTauri()) {
      state.availableAddons = [];
      setSourcesStatusText("");
      return;
    }

    try {
      setBusy(true);
      setSourcesStatus("status.loadingSources");
      state.availableAddons = await invoke("list_available_addons", {
        rootPath: state.rootPath,
        sourceUrls: state.sources,
      });
      setSourcesStatus("status.sourcesLoaded", { count: state.availableAddons.length });
    } catch (error) {
      state.availableAddons = [];
      setSourcesStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  async function updateAll() {
    const targets = state.installedAddons.filter((addon) => addon.hasUpdate);
    if (!targets.length) {
      setStatus("status.noUpdatesToInstall");
      return;
    }
    if (!hasTauri()) {
      setStatus("status.runtimeRequired");
      return;
    }

    try {
      setBusy(true);
      for (let index = 0; index < targets.length; index += 1) {
        const addon = targets[index];
        setStatus("status.updatingProgress", {
          current: index + 1,
          total: targets.length,
          name: addon.name,
        });
        await invoke("update_addon", { addonPath: addon.addonPath });
      }
      await refreshCollections();
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  function resetModalState({ checking = false, readmeLoading = false } = {}) {
    state.modalChecking = checking;
    state.modalTab = "info";
    state.modalReadme = null;
    state.modalReadmeLoading = readmeLoading;
    state.installPlan = null;
    state.installPlanLoading = false;
  }

  function finishModalLoading() {
    state.modalChecking = false;
    state.modalReadmeLoading = false;
  }

  async function refreshCollections() {
    await refreshInstalled();
    await refreshAvailable();
  }

  async function openAddonModal(addonPath) {
    const addon = state.installedAddons.find((item) => item.addonPath === addonPath);
    if (!addon) {
      return;
    }

    state.modalOpen = true;
    state.modalTargetType = "installed";
    state.modalAddon = addon;
    resetModalState({ checking: true, readmeLoading: true });

    if (!hasTauri()) {
      finishModalLoading();
      return;
    }

    try {
      const [addonResult, readmeResult] = await Promise.allSettled([
        invoke("check_addon", { addonPath }),
        invoke("load_addon_readme", { addonPath }),
      ]);

      if (addonResult.status === "fulfilled") {
        replaceInstalled(addonResult.value);
        state.modalAddon = addonResult.value;
      } else {
        state.modalAddon = {
          ...addon,
          remoteVersion: null,
          hasUpdate: false,
          hasError: true,
          status: normalizeError(addonResult.reason, locale.value),
        };
      }

      state.modalReadme = readmeResult.status === "fulfilled" ? readmeResult.value : null;
    } finally {
      finishModalLoading();
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
    state.modalAddon = createAvailableModalAddon(addon, translate);
    resetModalState({ checking: true, readmeLoading: true });

    if (!hasTauri()) {
      finishModalLoading();
      return;
    }

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
        state.modalAddon = createAvailableModalAddon(addonResult.value, translate);
      } else {
        state.modalAddon = {
          ...state.modalAddon,
          status: normalizeError(addonResult.reason, locale.value),
        };
      }

      state.modalReadme = readmeResult.status === "fulfilled" ? readmeResult.value : null;
    } finally {
      finishModalLoading();
    }
  }

  function closeModal() {
    state.modalOpen = false;
    state.modalTargetType = null;
    state.modalAddon = null;
    resetModalState();
  }

  function setModalTab(tab) {
    state.modalTab = tab;
  }

  function closeInstallPlan() {
    state.installPlan = null;
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
      state.installPlan = await invoke("preview_install", {
        rootPath: state.rootPath,
        repositoryUrl: addon.repositoryUrl,
        branch: addon.branch,
      });
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      state.installPlanLoading = false;
    }
  }

  async function confirmInstallPlan() {
    const addon = state.modalAddon;
    if (!addon?.repositoryUrl || !addon?.branch || !hasTauri()) {
      return;
    }

    try {
      setBusy(true);
      setStatus("status.installingAddon", { url: addon.repositoryUrl });
      await invoke("install_addon", {
        rootPath: state.rootPath,
        repositoryUrl: addon.repositoryUrl,
        branch: addon.branch,
      });
      closeInstallPlan();
      closeModal();
      await refreshCollections();
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  async function updateSelectedFromModal() {
    const selected = state.modalTargetType === "installed" ? state.modalAddon : null;
    if (!selected || !selected.hasUpdate || !hasTauri()) {
      return;
    }

    try {
      setBusy(true);
      setStatus("status.updatingOne", { name: selected.name });
      await invoke("update_addon", { addonPath: selected.addonPath });
      await refreshCollections();
      await openAddonModal(selected.addonPath);
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  async function rollbackSelectedFromModal() {
    const selected = state.modalTargetType === "installed" ? state.modalAddon : null;
    if (!selected?.addonPath || !hasTauri()) {
      return;
    }

    try {
      setBusy(true);
      setStatus("status.rollbackOne", { name: selected.name });
      await invoke("rollback_addon", { addonPath: selected.addonPath });
      await refreshCollections();
      await openAddonModal(selected.addonPath);
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      setBusy(false);
    }
  }

  async function removeSelectedFromModal() {
    const selected = state.modalTargetType === "installed" ? state.modalAddon : null;
    if (!selected?.addonPath || !hasTauri()) {
      return;
    }

    try {
      setBusy(true);
      setStatus("status.removingOne", { name: selected.name });
      await invoke("remove_addon", { addonPath: selected.addonPath });
      closeInstallPlan();
      closeModal();
      await refreshCollections();
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
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

  async function removeSource(sourceUrl) {
    state.sources = state.sources.filter((item) => item !== sourceUrl);
    await persistSources();
    setSourcesStatus("sources.removed");
    await refreshAvailable();
  }

  function setSourceInput(value) {
    state.sourceInput = value;
  }

  function setRootPath(value) {
    state.rootPath = value;
  }

  async function saveTypedRootPath() {
    await persistRootPath();
  }

  async function setLanguage(language) {
    state.language = language;
    persistLanguage();
  }

  function dismissAppUpdate() {
    state.appUpdateDismissed = true;
  }

  async function installAppUpdate() {
    if (!hasTauri() || !state.appUpdate || state.appUpdating) {
      return;
    }

    try {
      state.appUpdating = true;
      setStatus("app.installing");
      const installed = await invoke("install_app_update");
      if (!installed) {
        state.appUpdate = null;
        state.appUpdateDismissed = false;
        setStatus("app.noUpdate");
      }
    } catch (error) {
      setStatusText(normalizeError(error, locale.value));
    } finally {
      state.appUpdating = false;
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
      setStatusText(normalizeError(error, locale.value));
    }
  }

  function replaceInstalled(updated) {
    const index = state.installedAddons.findIndex((addon) => addon.addonPath === updated.addonPath);
    if (index === -1) {
      state.installedAddons.push(updated);
      state.installedAddons.sort((left, right) => left.name.localeCompare(right.name));
      return;
    }
    state.installedAddons[index] = updated;
  }

  function repositoryLink(repositoryUrl) {
    return repositoryHref(repositoryUrl);
  }

  return {
    state,
    locale,
    updateCount,
    t: translate,
    tf: translateFormat,
    hasTauri,
    initialize,
    chooseFolderAction,
    refreshAll,
    refreshInstalled,
    refreshAvailable,
    updateAll,
    openAddonModal,
    openAvailableModal,
    closeModal,
    setModalTab,
    requestInstallFromModal,
    closeInstallPlan,
    confirmInstallPlan,
    updateSelectedFromModal,
    rollbackSelectedFromModal,
    removeSelectedFromModal,
    addSource,
    removeSource,
    setSourceInput,
    setRootPath,
    saveTypedRootPath,
    setLanguage,
    dismissAppUpdate,
    installAppUpdate,
    openExternalUrl,
    repositoryLink,
  };
}

export function useAppStore() {
  return inject(appStoreKey);
}
