(() => {
  "use strict";

  function formatDate(value) {
    if (!Number.isFinite(Number(value)) || Number(value) <= 0) return "Unknown";
    try {
      return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(Number(value)));
    } catch {
      return new Date(Number(value)).toLocaleString();
    }
  }

  function init() {
    const dialog = document.getElementById("settings-dialog");
    if (!dialog || dialog.dataset.sessionsSectionReady === "1") return;
    dialog.dataset.sessionsSectionReady = "1";

    const footer = dialog.querySelector(".settings-footer");
    const section = document.createElement("section");
    section.className = "settings-section settings-devices-section";
    section.innerHTML =
      "<div class=\"settings-section-heading\">" +
        "<div><h3>Logged-in devices</h3><p class=\"settings-help\">Devices that currently have an active Larptrix sign-in session.</p></div>" +
        "<button type=\"button\" class=\"ghost settings-devices-refresh\">Refresh</button>" +
      "</div>" +
      "<div class=\"settings-device-list\" aria-live=\"polite\"><p class=\"settings-help\">Loading devices…</p></div>";
    footer ? dialog.insertBefore(section, footer) : dialog.append(section);

    const list = section.querySelector(".settings-device-list");
    const refresh = section.querySelector(".settings-devices-refresh");

    async function load() {
      list.innerHTML = "<p class=\"settings-help\">Loading devices…</p>";
      try {
        const response = await fetch("/api/me/sessions", {
          credentials: "same-origin",
          headers: { Accept: "application/json" },
        });
        const data = await response.json().catch(() => ({}));
        if (!response.ok) throw new Error(data.error || "Could not load logged-in devices.");
        const sessions = Array.isArray(data.sessions) ? data.sessions : [];

        list.replaceChildren();
        if (!sessions.length) {
          const empty = document.createElement("p");
          empty.className = "settings-help";
          empty.textContent = "No active Larptrix sessions.";
          list.append(empty);
          return;
        }

        for (const session of sessions) {
          const row = document.createElement("div");
          row.className = "settings-device-row" + (session.current ? " is-current" : "");

          const icon = document.createElement("div");
          icon.className = "settings-device-icon";
          icon.textContent = session.current ? "●" : "○";

          const info = document.createElement("div");
          info.className = "settings-device-info";

          const name = document.createElement("strong");
          name.textContent = session.current ? "This device" : (session.device_name || "Larptrix device");

          const meta = document.createElement("span");
          const lastSeen = Number(session.last_seen_at) > 0 ? formatDate(session.last_seen_at) : "unknown";
          const created = Number(session.created_at) > 0 ? formatDate(session.created_at) : "unknown";
          meta.textContent = session.current
            ? "Active now · last seen " + lastSeen
            : "Added " + created + " · last seen " + lastSeen;

          const agent = document.createElement("small");
          agent.className = "settings-device-agent";
          agent.textContent = session.user_agent || "Browser information unavailable";

          info.append(name, meta, agent);

          const actions = document.createElement("div");
          actions.className = "settings-device-actions";

          if (session.current) {
            const current = document.createElement("span");
            current.className = "settings-device-current";
            current.textContent = "Current";
            actions.append(current);
          } else {
            const revoke = document.createElement("button");
            revoke.type = "button";
            revoke.className = "ghost danger-item";
            revoke.textContent = "Log out";
            revoke.addEventListener("click", async () => {
              if (!confirm("Log out this device?")) return;
              revoke.disabled = true;
              try {
                const response = await fetch("/api/me/sessions/" + encodeURIComponent(session.session_id), {
                  method: "DELETE",
                  credentials: "same-origin",
                });
                const data = await response.json().catch(() => ({}));
                if (!response.ok) throw new Error(data.error || "Could not close this session.");
                await load();
              } catch (error) {
                revoke.disabled = false;
                alert(error.message || "Could not close this session.");
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
        message.textContent = error.message || "Could not load logged-in devices.";
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