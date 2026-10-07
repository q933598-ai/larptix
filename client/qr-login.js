(() => {
  "use strict";

  const ACCESS_KEY_RE = /^(?:[0-9a-f]{8}-){7}[0-9a-f]{8}$/i;

  function extractAccessKey(raw) {
    const text = String(raw || "").trim();
    if (!text) return "";
    try {
      const url = new URL(text, location.href);
      const hash = url.hash.startsWith("#") ? url.hash.slice(1) : url.hash;
      const params = new URLSearchParams(hash);
      const key = params.get("access_key")?.trim() || "";
      if (ACCESS_KEY_RE.test(key)) return key;
    } catch {}
    return ACCESS_KEY_RE.test(text) ? text : "";
  }

  async function jsonRequest(path, options = {}) {
    const response = await fetch(path, {
      credentials: "same-origin",
      headers: { "Content-Type": "application/json", Accept: "application/json", ...(options.headers || {}) },
      ...options,
    });
    const data = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(data.error || data.message || "Request failed");
    return data;
  }

  function init() {
    const accessKey = document.getElementById("access-key");
    const authForm = document.getElementById("auth");
    if (!accessKey || !authForm || document.getElementById("scan-qr-login")) return;

    const button = document.createElement("button");
    button.type = "button";
    button.id = "scan-qr-login";
    button.className = "ghost";
    button.textContent = "▣ Scan QR code";
    button.style.width = "100%";
    button.style.marginTop = "0.5rem";
    accessKey.insertAdjacentElement("afterend", button);

    const dialog = document.createElement("dialog");
    dialog.id = "qr-login-dialog";
    dialog.className = "qr-login-dialog";
    dialog.innerHTML =
      "<form method=\"dialog\">" +
        "<header class=\"settings-header\">" +
          "<div><p class=\"eyebrow\">LARPTRIX</p><h2 id=\"qr-login-title\">Scan sign-in QR</h2></div>" +
          "<button type=\"button\" id=\"qr-login-close\" class=\"ghost\" aria-label=\"Close\">×</button>" +
        "</header>" +
        "<div id=\"qr-login-scan-view\">" +
          "<p class=\"settings-help\">Open the sign-in QR on another Larptrix device and point your camera at it.</p>" +
          "<div class=\"qr-login-camera\"><video id=\"qr-login-video\" autoplay playsinline muted></video><div class=\"qr-login-frame\" aria-hidden=\"true\"></div></div>" +
          "<p id=\"qr-login-status\" class=\"settings-help\" aria-live=\"polite\">Starting camera…</p>" +
          "<p id=\"qr-login-error\" class=\"error\" hidden></p>" +
        "</div>" +
        "<div id=\"qr-login-confirm\" hidden>" +
          "<div class=\"qr-login-account-card\">" +
            "<div id=\"qr-login-avatar\" class=\"qr-login-avatar\">L</div>" +
            "<div><strong id=\"qr-login-account-name\">Larptrix account</strong><span id=\"qr-login-account-meta\">Ready to sign in</span></div>" +
          "</div>" +
          "<p class=\"settings-help\">This QR contains the sign-in key for this account. Continue only if you trust the device that displayed it.</p>" +
          "<div class=\"profile-actions\"><button id=\"qr-login-cancel\" type=\"button\" class=\"ghost\">Cancel</button><button id=\"qr-login-confirm-button\" type=\"button\">Sign in</button></div>" +
        "</div>" +
      "</form>";
    document.body.appendChild(dialog);

    const video = dialog.querySelector("#qr-login-video");
    const status = dialog.querySelector("#qr-login-status");
    const error = dialog.querySelector("#qr-login-error");
    const scanView = dialog.querySelector("#qr-login-scan-view");
    const confirmView = dialog.querySelector("#qr-login-confirm");
    const accountName = dialog.querySelector("#qr-login-account-name");
    const accountMeta = dialog.querySelector("#qr-login-account-meta");
    const avatar = dialog.querySelector("#qr-login-avatar");
    const cancel = dialog.querySelector("#qr-login-cancel");
    const confirmButton = dialog.querySelector("#qr-login-confirm-button");
    const closeButton = dialog.querySelector("#qr-login-close");

    let stream = null;
    let timer = null;
    let detector = null;
    let pendingKey = "";

    function stopCamera() {
      if (timer) clearInterval(timer);
      timer = null;
      detector = null;
      if (stream) stream.getTracks().forEach((track) => track.stop());
      stream = null;
      if (video) video.srcObject = null;
    }

    function showScanError(message) {
      error.textContent = message;
      error.hidden = false;
      status.hidden = true;
    }

    function resetDialog() {
      stopCamera();
      pendingKey = "";
      scanView.hidden = false;
      confirmView.hidden = true;
      error.hidden = true;
      status.hidden = false;
      status.textContent = "Starting camera…";
    }

    async function confirmKey(key) {
      pendingKey = key;
      stopCamera();
      status.hidden = true;
      try {
        const account = await jsonRequest("/api/login/preview", {
          method: "POST",
          body: JSON.stringify({ access_key: key }),
        });
        accountName.textContent = account.display_name || "Larptrix account";
        accountMeta.textContent = account.user_id ? "@" + String(account.user_id).slice(0, 8) + " · sign in to this device" : "Sign in to this device";
        avatar.textContent = (account.display_name || "L").trim().charAt(0).toUpperCase() || "L";
      } catch {
        accountName.textContent = "Larptrix account";
        accountMeta.textContent = "Sign-in key detected";
        avatar.textContent = "L";
      }
      scanView.hidden = true;
      confirmView.hidden = false;
    }

    async function finishLogin() {
      if (!pendingKey) return;
      confirmButton.disabled = true;
      try {
        await jsonRequest("/api/login", {
          method: "POST",
          body: JSON.stringify({ access_key: pendingKey }),
        });
        dialog.close();
        location.reload();
      } catch (err) {
        confirmButton.disabled = false;
        scanView.hidden = false;
        confirmView.hidden = true;
        showScanError("QR sign-in failed: " + (err.message || "invalid sign-in code"));
      }
    }

    async function startScanner() {
      error.hidden = true;
      status.hidden = false;
      status.textContent = "Starting camera…";

      if (!window.isSecureContext) {
        showScanError("Camera access requires HTTPS. Use the normal HTTPS Larptrix address.");
        return;
      }
      if (!navigator.mediaDevices?.getUserMedia) {
        showScanError("This browser does not provide camera access.");
        return;
      }
      if (!("BarcodeDetector" in window)) {
        showScanError("This browser cannot scan QR codes inside the app. On Android, try Chrome or another browser with QR camera support, or use the phone camera.");
        return;
      }

      try {
        const formats = await BarcodeDetector.getSupportedFormats();
        if (!formats.includes("qr_code")) {
          showScanError("This browser has a camera scanner, but QR codes are not supported.");
          return;
        }

        detector = new BarcodeDetector({ formats: ["qr_code"] });
        stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: { ideal: "environment" } },
          audio: false,
        });
        video.srcObject = stream;
        await video.play();
        status.textContent = "Point the camera at the sign-in QR…";

        timer = setInterval(async () => {
          if (!detector || video.readyState < HTMLMediaElement.HAVE_CURRENT_DATA) return;
          try {
            const results = await detector.detect(video);
            for (const result of results) {
              const key = extractAccessKey(result.rawValue);
              if (key) {
                await confirmKey(key);
                return;
              }
            }
          } catch {}
        }, 250);
      } catch (err) {
        stopCamera();
        if (err?.name === "NotAllowedError") showScanError("Camera permission was denied. Allow camera access and try again.");
        else if (err?.name === "NotFoundError") showScanError("No camera was found on this device.");
        else showScanError("Could not start the camera.");
      }
    }

    button.addEventListener("click", () => {
      resetDialog();
      dialog.showModal();
      void startScanner();
    });
    cancel.addEventListener("click", () => {
      resetDialog();
      void startScanner();
    });
    confirmButton.addEventListener("click", () => void finishLogin());
    closeButton.addEventListener("click", () => dialog.close());
    dialog.addEventListener("close", stopCamera);
    dialog.addEventListener("cancel", stopCamera);
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init, { once: true });
  else init();
})();