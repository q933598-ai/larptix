(() => {
  "use strict";

  function formatDate(value) {
    if (!Number.isFinite(Number(value))) return "Unknown time";
    try {
      return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(Number(value)));
    } catch {
      return new Date(Number(value)).toLocaleString();
    }
  }

  function init() {
    const dialog = document.getElementById("settings-dialog");
    if (!dialog || dialog.dataset.devicesSectionReady === "1") return;
    dialog.dataset.devicesSectionReady = "1";

    const footer = dialog.querySelector(".settings-footer");
    const section = document.createElement("section");
    section.className = "settings-section settings-devices-section";
    section.innerHTML =
      "<div class=\"settings-section-heading\">" +
        "<div><h3>Devices</h3><p class=\"settings-help\">Larptrix devices with encrypted chat keys for this account.</p></div>" +
        "<button type=\"button\" class=\"ghost settings-devices-refresh\">Refresh</button>" +
      "</div>" +
      "<div class=\"settings-device-list\" aria-live=\"polite\"><p class=\"settings-help\">Loading devices…</p></div>";
    footer ? dialog.insertBefore(section, footer) : dialog.append(section);

    const list = section.querySelector(".settings-device-list");
    const refresh = section.querySelector(".settings-devices-refresh");

    async function load() {
      list.innerHTML = "<p class=\"settings-help\">Loading devices…</p>";
      try {
        const response = await fetch("/api/me/crypto-devices", { credentials: "same-origin", headers: { Accept: "application/json" } });
        if (!response.ok) throw new Error("Could not load devices.");
        const data = await response.json();
        const devices = Array.isArray(data.devices) ? data.devices : [];
        const meResponse = await fetch("/api/me", { credentials: "same-origin", headers: { Accept: "application/json" } });
        const me = meResponse.ok ? await meResponse.json() : null;
        const activeId = me?.user_id ? currentDeviceId(me.user_id) : "";

        list.replaceChildren();
        if (!devices.length) {
          const empty = document.createElement("p");
          empty.className = "settings-help";
          empty.textContent = "No encrypted devices are registered yet.";
          list.append(empty);
          return;
        }

        for (const device of devices) {
          const row = document.createElement("div");
          row.className = "settings-device-row";

          const icon = document.createElement("div");
          icon.className = "settings-device-icon";
          icon.textContent = device.device_id === activeId ? "●" : "○";

          const info = document.createElement("div");
          info.className = "settings-device-info";
          const name = document.createElement("strong");
          const current = device.device_id === activeId;
          name.textContent = current ? "This device" : "Encrypted device " + String(device.device_id || "").slice(0, 10);
          const meta = document.createElement("span");
          meta.textContent = current
            ? "Active here · updated " + formatDate(device.updated_at)
            : "Added " + formatDate(device.created_at) + " · updated " + formatDate(device.updated_at);
          info.append(name, meta);

          const actions = document.createElement("div");
          actions.className = "settings-device-actions";
          if (!current) {
            const revoke = document.createElement("button");
            revoke.type = "button";
            revoke.className = "ghost danger-item";
            revoke.textContent = "Revoke";
            revoke.addEventListener("click", async () => {
              if (!confirm("Revoke this encrypted device? It will need to set up E2E again.")) return;
              revoke.disabled = true;
              try {
                const response = await fetch("/api/me/crypto-devices/" + encodeURIComponent(device.device_id), { method: "DELETE", credentials: "same-origin" });
                if (!response.ok) throw new Error("Could not revoke device.");
                await load();
              } catch (error) {
                revoke.disabled = false;
                alert(error.message || "Could not revoke device.");
              }
            });
            actions.append(revoke);
          }
          row.append(icon, info, actions);
          list.append(row);
        }
      } catch (error) {
        list.innerHTML = "";
        const message = document.createElement("p");
        message.className = "error";
        message.textContent = error.message || "Could not load devices.";
        list.append(message);
      }
    }

    refresh.addEventListener("click", () => void load());
    const observer = new MutationObserver(() => {
      if (dialog.open) void load();
    });
    observer.observe(dialog, { attributes: true, attributeFilter: ["open"] });
    if (dialog.open) void load();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init, { once: true });
  else init();
})();