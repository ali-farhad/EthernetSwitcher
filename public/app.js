(() => {
  const bootScreen = document.getElementById("boot-screen");

  const state = {
    adapters: [],
    query: "",
    loading: true,
    switching: false,
  };

  const icon = (paths, className = "") =>
    `<svg class="${className}" viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`;

  function errorText(error) {
    if (typeof error === "string") return error;
    if (error && typeof error.message === "string") return error.message;
    return "Windows returned an unexpected error.";
  }

  function createShell() {
    const shell = document.createElement("div");
    shell.className = "app-shell";
    shell.innerHTML = `
      <div class="ambient ambient-one"></div>
      <div class="ambient ambient-two"></div>
      <header class="topbar">
        <a class="brand" href="#" aria-label="Ethernet Switcher home">
          <span class="brand-mark">${icon('<path d="M7 8V5h10v3M5 11v6h4v-6H5Zm10 0v6h4v-6h-4ZM10 11v6h4v-6h-4ZM12 8v3M7 8h10"/>')}</span>
          <span><strong>Ethernet</strong><small>SWITCHER</small></span>
        </a>
        <div class="system-state"><span class="pulse"></span><span id="network-state">Checking network</span></div>
      </header>
      <main>
        <section class="control-panel">
          <div class="panel-heading">
            <div><h2>Ethernet adapters</h2><p id="adapter-summary">Checking Windows...</p></div>
            <div class="tools">
              <label class="search">
                ${icon('<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>')}
                <input id="adapter-search" type="search" placeholder="Search adapters" aria-label="Search adapters">
              </label>
              <button id="refresh-button" class="icon-button" title="Refresh adapters" aria-label="Refresh adapters">
                ${icon('<path d="M20 12a8 8 0 1 1-2.34-5.66L20 8"/><path d="M20 3v5h-5"/>', 'refresh-icon')}
              </button>
            </div>
          </div>
          <div id="messages"></div>
          <div id="adapter-list" class="adapter-list" aria-live="polite"></div>
        </section>
        <footer><span>A local Windows utility</span><span>No network data leaves this device</span></footer>
      </main>
      `;
    document.body.appendChild(shell);

    document.getElementById("adapter-search").addEventListener("input", (event) => {
      state.query = event.target.value.toLowerCase();
      renderAdapters();
    });
    document.getElementById("refresh-button").addEventListener("click", reloadAdapters);
  }

  function showMessage(kind, title, message) {
    const host = document.getElementById("messages");
    host.replaceChildren();
    const alert = document.createElement("div");
    alert.className = `alert ${kind === "error" ? "error-alert" : "success-alert"}`;
    alert.setAttribute("role", kind === "error" ? "alert" : "status");

    const symbol = document.createElement("span");
    symbol.textContent = kind === "error" ? "!" : "OK";
    const copy = document.createElement("div");
    const heading = document.createElement("strong");
    heading.textContent = title;
    const detail = document.createElement("p");
    detail.textContent = message;
    copy.append(heading, detail);
    alert.append(symbol, copy);

    if (kind !== "error") {
      const close = document.createElement("button");
      close.setAttribute("aria-label", "Dismiss");
      close.textContent = "x";
      close.addEventListener("click", () => host.replaceChildren());
      alert.appendChild(close);
    }
    host.appendChild(alert);
  }

  function renderEmpty(title, message, loading = false) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    const symbol = document.createElement("span");
    symbol.className = loading ? "loader" : "empty-icon";
    if (!loading) symbol.textContent = "~";
    const heading = document.createElement("h3");
    heading.textContent = title;
    const detail = document.createElement("p");
    detail.textContent = message;
    empty.append(symbol, heading, detail);
    return empty;
  }

  function metadata(label, value) {
    const item = document.createElement("span");
    const heading = document.createElement("small");
    heading.textContent = label;
    item.append(heading, document.createTextNode(value));
    return item;
  }

  function renderAdapters() {
    const list = document.getElementById("adapter-list");
    list.replaceChildren();

    if (state.loading && state.adapters.length === 0) {
      list.appendChild(renderEmpty("Finding Ethernet adapters...", "Asking Windows for physical network interfaces", true));
      return;
    }

    const visible = state.adapters.filter((adapter) => {
      const haystack = `${adapter.name} ${adapter.description} ${adapter.ipv4Address || ""}`.toLowerCase();
      return haystack.includes(state.query);
    });

    if (visible.length === 0) {
      list.appendChild(renderEmpty(
        state.query ? "No matching adapters" : "No Ethernet adapters found",
        state.query ? "Try a different name or IP address." : "Connect a wired network adapter, then refresh."
      ));
      return;
    }

    const activeCount = state.adapters.filter((adapter) => adapter.status.toLowerCase() === "up").length;
    for (const adapter of visible) {
      const isActive = adapter.status.toLowerCase() === "up";
      const isExclusive = isActive && activeCount === 1;
      const card = document.createElement("article");
      card.className = `adapter-card${isActive ? " active" : ""}`;

      const adapterIcon = document.createElement("div");
      adapterIcon.className = "adapter-icon";
      adapterIcon.innerHTML = icon('<path d="M6 4h12v7l-3 3H9l-3-3V4Z"/><path d="M9 4v4m3-4v4m3-4v4M8 17h8m-4-3v3"/>');

      const content = document.createElement("div");
      content.className = "adapter-main";
      const titleRow = document.createElement("div");
      titleRow.className = "adapter-title";
      const name = document.createElement("h3");
      name.textContent = adapter.name;
      const status = document.createElement("span");
      status.className = `status${isActive ? " online" : ""}`;
      status.innerHTML = "<i></i>";
      status.appendChild(document.createTextNode(isActive ? "Connected" : adapter.status === "Disabled" ? "Disabled" : "Disconnected"));
      titleRow.append(name, status);

      const description = document.createElement("p");
      description.className = "description";
      description.textContent = adapter.description;
      const details = document.createElement("div");
      details.className = "metadata";
      details.append(
        metadata("IP ADDRESS", adapter.ipv4Address || "Not assigned"),
        metadata("LINK SPEED", adapter.linkSpeed),
        metadata("MAC", adapter.macAddress)
      );
      content.append(titleRow, description, details);

      const button = document.createElement("button");
      button.className = `switch-button${isExclusive ? " current" : ""}`;
      button.disabled = isExclusive || state.switching;
      button.textContent = isExclusive ? "Active" : isActive ? "Use only" : "Switch";
      button.addEventListener("click", () => switchAdapter(adapter));
      card.append(adapterIcon, content, button);
      list.appendChild(card);
    }
  }

  function updateSummary() {
    const count = state.adapters.length;
    document.getElementById("adapter-summary").textContent = `${count} physical adapter${count === 1 ? "" : "s"} detected`;
    const online = state.adapters.some((adapter) => adapter.status.toLowerCase() === "up");
    document.getElementById("network-state").textContent = online ? "Network online" : "No active link";
  }

  async function reloadAdapters() {
    state.loading = true;
    const refresh = document.querySelector(".refresh-icon");
    const button = document.getElementById("refresh-button");
    refresh.classList.add("spinning");
    button.disabled = true;
    renderAdapters();
    try {
      state.adapters = await window.__TAURI__.core.invoke("list_ethernet_adapters");
      document.getElementById("messages").replaceChildren();
      updateSummary();
    } catch (error) {
      showMessage("error", "Couldn't load adapters", errorText(error));
    } finally {
      state.loading = false;
      refresh.classList.remove("spinning");
      button.disabled = false;
      renderAdapters();
    }
  }

  async function switchAdapter(adapter) {
    state.switching = true;
    document.getElementById("messages").replaceChildren();
    renderAdapters();
    try {
      await window.__TAURI__.core.invoke("switch_adapter", { adapterGuid: adapter.guid });
      await reloadAdapters();
      showMessage("success", "Connection switched", `${adapter.name} is now active.`);
    } catch (error) {
      showMessage("error", "Couldn't switch connection", errorText(error));
    } finally {
      state.switching = false;
      renderAdapters();
    }
  }

  try {
    createShell();
    bootScreen.remove();
    reloadAdapters();
  } catch (error) {
    const detail = bootScreen.querySelector("span");
    detail.textContent = `Startup failed: ${errorText(error)}`;
  }
})();
