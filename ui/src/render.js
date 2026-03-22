import { t, tf } from "./i18n.js";
import { countUpdates } from "./state.js";
import { escapeHtml, renderMarkdown, repositoryHref } from "./utils.js";

export function renderApp(state, elements) {
  renderStaticText(state, elements);
  renderTabs(state, elements);
  elements.statusText.textContent = state.statusMessage;
  elements.sourcesStatusText.textContent = state.sourcesStatusMessage;

  const updates = countUpdates(state.installedAddons);
  const appUpdate = state.appUpdate;
  elements.updatesPill.hidden = updates === 0;
  if (updates > 0) {
    elements.updatesPill.textContent = tf(state.locale, "addons.updatesShort", { count: updates });
  }

  elements.updateBanner.hidden = !appUpdate || state.appUpdateDismissed;
  if (appUpdate && !state.appUpdateDismissed) {
    elements.updateBannerTitle.textContent = tf(state.locale, "app.updateTitle", {
      version: appUpdate.version,
    });
    elements.updateBannerText.textContent = appUpdate.notes || t(state.locale, "app.updateReady");
  }

  elements.checkButton.disabled = state.busy || state.installedAddons.length === 0;
  elements.updateAllButton.hidden = updates === 0;
  elements.updateAllButton.disabled = state.busy || updates === 0;
  elements.sourceInput.disabled = state.busy;
  elements.addSourceButton.disabled = state.busy || !state.sourceInput.trim();
  elements.languageSelect.disabled = state.busy;
  elements.updateBannerInstall.disabled = state.appUpdating;
  elements.appUpdateButton.hidden = !appUpdate;
  elements.appUpdateButton.disabled = state.appUpdating;

  renderInstalledGrid(state, elements);
  renderAvailableGrid(state, elements);
  renderSourceList(state, elements);
  renderModal(state, elements);
}

function renderStaticText(state, elements) {
  const updates = countUpdates(state.installedAddons);

  document.title = "GLuaManager";
  document.documentElement.lang = state.language;
  elements.rootPath.value = state.rootPath;
  elements.sourceInput.value = state.sourceInput;
  elements.languageSelect.value = state.language;
  elements.rootPath.placeholder = t(state.locale, "workspace.placeholder");
  elements.sourceInput.placeholder = t(state.locale, "sources.placeholder");
  elements.chooseFolderLabel.textContent = t(state.locale, "workspace.folder");
  elements.scanButtonLabel.textContent = t(state.locale, "workspace.scan");
  elements.workspaceHint.textContent = t(state.locale, "workspace.hint");
  elements.tabAddons.textContent = t(state.locale, "tabs.addons");
  elements.tabSettings.textContent = t(state.locale, "tabs.settings");
  elements.addonsTitle.textContent = t(state.locale, "addons.title");
  elements.installedTitle.textContent = tf(state.locale, "addons.installedCount", {
    count: state.installedAddons.length,
  });
  elements.availableTitle.textContent = tf(state.locale, "addons.availableCount", {
    count: state.availableAddons.length,
  });
  elements.checkButton.textContent = t(state.locale, "addons.check");
  elements.updateAllButton.textContent =
    updates > 0
      ? tf(state.locale, "addons.updateAllCount", { count: updates })
      : t(state.locale, "addons.updateAll");
  elements.sourcesTitle.textContent = t(state.locale, "sources.title");
  elements.addSourceButton.textContent = t(state.locale, "sources.add");
  elements.settingsTitle.textContent = t(state.locale, "settings.title");
  elements.languageLabel.textContent = t(state.locale, "settings.language");
  elements.appVersionLabel.textContent = t(state.locale, "settings.application");
  elements.appVersionText.textContent = t(state.locale, "settings.currentVersion");
  elements.appUpdateLabel.textContent = t(state.locale, "settings.availableUpdate");
  elements.appVersionValue.textContent = state.appVersion || t(state.locale, "common.notSpecified");
  elements.appUpdateValue.textContent = state.appUpdate
    ? tf(state.locale, "settings.updateVersion", { version: state.appUpdate.version })
    : state.updaterEnabled
      ? t(state.locale, "settings.noAppUpdate")
      : t(state.locale, "settings.updaterDisabled");
  elements.appUpdateButton.textContent = t(state.locale, "app.installUpdate");
  elements.updateBannerDismiss.textContent = t(state.locale, "app.dismissUpdate");
  elements.updateBannerInstall.textContent = t(state.locale, "app.installUpdate");
  elements.modalTabInfo.textContent = t(state.locale, "modal.infoTab");
  elements.modalTabReadme.textContent = t(state.locale, "modal.readmeTab");

  elements.labelInstalledVersion.textContent = t(state.locale, "modal.installedVersion");
  elements.labelRemoteVersion.textContent = t(state.locale, "modal.remoteVersion");
  elements.labelState.textContent = t(state.locale, "modal.state");
  elements.labelDescription.textContent = t(state.locale, "modal.description");
  elements.labelFolder.textContent = t(state.locale, "modal.folder");
  elements.labelPreserve.textContent = t(state.locale, "modal.preserve");
  elements.labelDependencies.textContent = t(state.locale, "modal.dependencies");
  elements.modalInstallButton.textContent = t(state.locale, "addons.install");
  elements.modalRollbackButton.textContent = t(state.locale, "addons.rollback");
  elements.installConfirmTitle.textContent = t(state.locale, "install.confirmTitle");
  elements.installConfirmCancel.textContent = t(state.locale, "install.cancel");
  elements.installConfirmAccept.textContent = t(state.locale, "install.accept");
}

function renderTabs(state, elements) {
  elements.tabButtons.forEach((button) => {
    const active = button.dataset.tab === state.activeTab;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-selected", active ? "true" : "false");
  });
  elements.tabPanels.forEach((panel) => {
    const active = panel.dataset.panel === state.activeTab;
    panel.classList.toggle("is-active", active);
    panel.hidden = !active;
  });
}

function renderInstalledGrid(state, elements) {
  if (state.installedAddons.length === 0) {
    const message = state.rootPath
      ? t(state.locale, "addons.emptyWithFolder")
      : t(state.locale, "addons.empty");
    elements.installedAddonList.innerHTML = renderEmptyState(
      t(state.locale, "addons.emptyTitle"),
      message,
    );
    return;
  }

  elements.installedAddonList.innerHTML = state.installedAddons
    .map((addon) => {
      const description = addon.description || t(state.locale, "addons.noDescription");
      const name = addon.name || t(state.locale, "common.notSpecified");
      const version = addon.version || t(state.locale, "common.notSpecified");
      return `
        <article class="addon-tile addon-tile--installed" data-addon-path="${escapeHtml(addon.addonPath)}">
          <div class="addon-tile__top">
            <div class="addon-tile__title">
              <h3>${escapeHtml(name)}</h3>
            </div>
            ${renderInstalledBadge(state, addon)}
          </div>
          <p>${escapeHtml(description)}</p>
          <div class="addon-tile__meta">
            <span>${escapeHtml(tf(state.locale, "addons.version", { version }))}</span>
          </div>
        </article>
      `;
    })
    .join("");

  for (const tile of elements.installedAddonList.querySelectorAll(".addon-tile")) {
    tile.addEventListener("click", () => state.onOpenModal(tile.dataset.addonPath));
  }
}

function renderAvailableGrid(state, elements) {
  if (state.sources.length === 0) {
    elements.availableAddonList.innerHTML = renderEmptyState(
      t(state.locale, "sources.emptyTitle"),
      t(state.locale, "sources.empty"),
    );
    return;
  }

  if (state.availableAddons.length === 0) {
    elements.availableAddonList.innerHTML = renderEmptyState(
      t(state.locale, "addons.availableEmptyTitle"),
      t(state.locale, "addons.availableEmpty"),
    );
    return;
  }

  elements.availableAddonList.innerHTML = state.availableAddons
    .map((addon) => {
      const description = addon.description || t(state.locale, "addons.noDescription");
      const installText = addon.installed
        ? t(state.locale, "addons.installedState")
        : t(state.locale, "addons.install");

      return `
        <article
          class="addon-tile addon-tile--available"
          data-repository-url="${escapeHtml(addon.repositoryUrl)}"
          data-branch="${escapeHtml(addon.branch || "")}"
        >
          <div class="addon-tile__top">
            <div class="addon-tile__title">
              <h3>${escapeHtml(addon.name || t(state.locale, "common.notSpecified"))}</h3>
            </div>
            ${
              addon.installed
                ? `<span class="badge" data-kind="ok">${escapeHtml(t(state.locale, "addons.installedState"))}</span>`
                : `<span class="badge" data-kind="update">${escapeHtml(t(state.locale, "addons.availableState"))}</span>`
            }
          </div>
          <p>${escapeHtml(description)}</p>
          <div class="addon-tile__meta">
            <span>${escapeHtml(tf(state.locale, "addons.version", { version: addon.version || t(state.locale, "common.notSpecified") }))}</span>
          </div>
          <div class="addon-tile__actions">
            <button
              class="button ${addon.installed ? "button--ghost" : "button--accent"}"
              data-install-url="${escapeHtml(addon.repositoryUrl)}"
              data-install-branch="${escapeHtml(addon.branch || "")}"
              ${addon.installed || state.busy ? "disabled" : ""}
            >${escapeHtml(installText)}</button>
          </div>
        </article>
      `;
    })
    .join("");

  for (const button of elements.availableAddonList.querySelectorAll("[data-install-url]")) {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      state.onInstallAvailable(button.dataset.installUrl, button.dataset.installBranch || "");
    });
  }

  for (const tile of elements.availableAddonList.querySelectorAll(".addon-tile")) {
    tile.addEventListener("click", () => {
      state.onOpenAvailableModal(
        tile.dataset.repositoryUrl,
        tile.dataset.branch || "",
      );
    });
  }
}

function renderSourceList(state, elements) {
  if (state.sources.length === 0) {
    elements.sourceList.innerHTML = renderEmptyState(
      t(state.locale, "sources.emptyTitle"),
      t(state.locale, "sources.empty"),
    );
    return;
  }

  elements.sourceList.innerHTML = state.sources
    .map(
      (source) => `
        <article class="source-item">
          <code>${escapeHtml(source)}</code>
          <button class="button button--ghost" data-remove-source="${escapeHtml(source)}">${escapeHtml(t(state.locale, "sources.remove"))}</button>
        </article>
      `,
    )
    .join("");

  for (const button of elements.sourceList.querySelectorAll("[data-remove-source]")) {
    button.addEventListener("click", () => state.onRemoveSource(button.dataset.removeSource));
  }
}

function renderModal(state, elements) {
  if (!state.modalOpen) {
    elements.modal.hidden = true;
    return;
  }

  const addon = state.modalAddon;
  if (!addon) {
    elements.modal.hidden = true;
    return;
  }

  const isInstalled = state.modalTargetType === "installed";
  const repoHref = repositoryHref(addon.repositoryUrl);
  elements.modal.hidden = false;
  elements.modalTitle.textContent = addon.name || t(state.locale, "common.notSpecified");
  elements.modalSubtitle.textContent = "";
  elements.modalInstalledVersion.textContent =
    addon.version || t(state.locale, "common.notSpecified");
  elements.modalRepositoryInline.innerHTML = addon.repositoryUrl
    ? repoHref
      ? `
          <button class="repo-link repo-link--inline" type="button" data-open-external="${escapeHtml(repoHref)}">
            <span class="repo-link__main">${escapeHtml(addon.repositoryUrl)}</span>
            <span class="repo-link__icon">
              <svg class="icon"><use href="#i-github" /></svg>
              <svg class="icon"><use href="#i-external" /></svg>
            </span>
          </button>
        `
      : `<span class="repo-inline-text">${escapeHtml(addon.repositoryUrl)}</span>`
    : "";
  elements.modalFolder.textContent = isInstalled
    ? addon.addonPath || t(state.locale, "common.notSpecified")
    : state.rootPath || t(state.locale, "common.notSpecified");

  const isChecking = state.modalChecking;
  const hasRemote = Boolean(addon.remoteVersion);
  const versionsEqual =
    isInstalled && hasRemote && addon.version.trim() === addon.remoteVersion.trim();
  const canUpdate = isInstalled && !isChecking && hasRemote && addon.hasUpdate && !versionsEqual;
  const canRollback = isInstalled && !isChecking;
  const canInstall = !isInstalled && !addon.installed && !state.busy;

  elements.modalRemoteVersion.textContent = isChecking
    ? t(state.locale, "modal.checking")
    : isInstalled
      ? addon.remoteVersion || t(state.locale, "modal.notChecked")
      : addon.version || t(state.locale, "common.notSpecified");
  elements.modalState.textContent = isChecking ? t(state.locale, "modal.checking") : addon.status;
  elements.modalDescription.innerHTML = renderMarkdown(
    addon.description || t(state.locale, "addons.noDescription"),
  );
  elements.modalVersionBand.textContent = isChecking
    ? t(state.locale, "modal.checking")
    : !isInstalled
      ? tf(state.locale, "modal.readyToInstall", { version: addon.version || t(state.locale, "common.notSpecified") })
      : !hasRemote
      ? t(state.locale, "modal.noRemoteVersion")
      : versionsEqual
        ? tf(state.locale, "modal.currentVersion", { version: addon.version })
        : tf(state.locale, "modal.updateRange", {
            from: addon.version,
            to: addon.remoteVersion,
          });

  elements.modalPreserve.innerHTML = addon.preserve.length
    ? addon.preserve.map((item) => `<span class="code-chip">${escapeHtml(item)}</span>`).join("")
    : `<span class="code-chip">${escapeHtml(t(state.locale, "modal.noProtectedPaths"))}</span>`;
  elements.modalDependencies.innerHTML = renderDependencies(state, addon.dependencies || []);

  elements.modalTabButtons.forEach((button) => {
    const active = button.dataset.modalTab === state.modalTab;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-selected", active ? "true" : "false");
  });

  elements.modalTabPanels.forEach((panel) => {
    const active = panel.dataset.modalPanel === state.modalTab;
    panel.classList.toggle("is-active", active);
    panel.hidden = !active;
  });

  if (state.modalReadmeLoading) {
    elements.modalReadme.innerHTML = `<div class="readme-state">${escapeHtml(t(state.locale, "modal.readmeLoading"))}</div>`;
  } else if (state.modalReadme) {
    elements.modalReadme.innerHTML = renderMarkdown(state.modalReadme.content, {
      baseUrl: state.modalReadme.baseUrl,
      localBasePath: state.modalReadme.localBasePath,
    });
  } else {
    elements.modalReadme.innerHTML = `<div class="readme-state">${escapeHtml(t(state.locale, "modal.readmeEmpty"))}</div>`;
  }

  elements.modalFooter.hidden = !canUpdate && !canInstall && !canRollback;
  elements.modalRollbackButton.hidden = !canRollback;
  elements.modalRollbackButton.disabled = state.busy || !canRollback;
  elements.modalInstallButton.hidden = !canInstall;
  elements.modalInstallButton.disabled = state.busy || state.installPlanLoading;
  elements.modalUpdateButton.hidden = !canUpdate;
  elements.modalUpdateButton.disabled = state.busy || !canUpdate;
  elements.modalUpdateButton.textContent = tf(state.locale, "modal.updateRange", {
    from: addon.version,
    to: addon.remoteVersion || addon.version,
  });

  elements.installConfirm.hidden = !state.installPlan;
  if (state.installPlan) {
    elements.installConfirmText.textContent = tf(state.locale, "install.confirmText", {
      root: state.installPlan.rootName,
    });
    elements.installConfirmList.innerHTML = state.installPlan.items
      .map(
        (item) => `
          <div class="install-confirm__item">
            <strong>${escapeHtml(item.name || item.repositoryUrl)}</strong>
            <span>${escapeHtml(item.branch)}</span>
          </div>
        `,
      )
      .join("");
  } else {
    elements.installConfirmText.textContent = "";
    elements.installConfirmList.innerHTML = "";
  }
}

function renderDependencies(state, dependencies) {
  if (!dependencies.length) {
    return `<span class="code-chip">${escapeHtml(t(state.locale, "modal.noDependencies"))}</span>`;
  }

  return dependencies
    .map(
      (dependency) => {
        const href = repositoryHref(dependency.url);
        const repository = dependency.url || t(state.locale, "common.notSpecified");
        return `
        <div class="dependency-item">
          ${
            href
              ? `<button class="dependency-link" type="button" data-open-external="${escapeHtml(href)}">
                  <span>${escapeHtml(repository)}</span>
                  <span class="dependency-link__icon">
                    <svg class="icon"><use href="#i-github" /></svg>
                    <svg class="icon"><use href="#i-external" /></svg>
                  </span>
                </button>`
              : `<code>${escapeHtml(repository)}</code>`
          }
          ${
            dependency.branch
              ? `<span class="dependency-meta">${escapeHtml(dependency.branch)}</span>`
              : ""
          }
        </div>
      `;
      },
    )
    .join("");
}

function renderInstalledBadge(state, addon) {
  if (addon.hasUpdate) {
    return `<span class="badge" data-kind="update">${escapeHtml(t(state.locale, "addons.badgeUpdate"))}</span>`;
  }
  if (addon.hasError) {
    return `<span class="badge" data-kind="error">${escapeHtml(t(state.locale, "addons.badgeError"))}</span>`;
  }
  return `<span class="badge" data-kind="ok">${escapeHtml(t(state.locale, "addons.badgeOk"))}</span>`;
}

function renderEmptyState(title, text) {
  return `
    <div class="empty-state">
      <div>
        <strong>${escapeHtml(title)}</strong>
        <p>${escapeHtml(text)}</p>
      </div>
    </div>
  `;
}
