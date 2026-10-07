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
    } catch {
      // Fall through and try the raw value.
    }

    return ACCESS_KEY_RE.test(text) ? text : "";
  }

  function init() {
    const accessKey = document.getElementById("access-key");
    const authForm = document.getElementById("auth");
    if (!accessKey || !authForm) return;

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
    dialog.style.maxWidth = "520px";
    dialog.style.width = "min(92vw, 520px)";

    dialog.innerHTML = `
      <form method="dialog" style="margin:0">
        <h2>Scan Larptrix QR code</h2>
        <p class="settings-help">
          Open the sign-in QR code on another Larptrix device and point your camera at it.
        </p>
        <div style="position:relative;overflow:hidden;border-radius:14px;background:#000;aspect-ratio:1/1">
          <video id="qr-login-video" autoplay playsinline muted
            style="width:100%;height:100%;object-fit:cover"></video>
          <div style="position:absolute;inset:18%;border:2px solid currentColor;border-radius:18px;pointer-events:none"></div>
        </div>
        <p id="qr-login-status" class="settings-help" aria-live="polite">Starting camera…</p>
        <p id="qr-login-error" class="error" hidden></p>
        <div class="profile-actions">
          <button id="qr-login-cancel" type="submit" class="ghost">Cancel</button>
        </div>
      </form>
    `;

    document.body.appendChild(dialog);

    const video = dialog.querySelector("#qr-login-video");
    const status = dialog.querySelector("#qr-login-status");
    const error = dialog.querySelector("#qr-login-error");
    let stream = null;
    let timer = null;
    let detector = null;

    function stopCamera() {
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
      if (stream) {
        for (const track of stream.getTracks()) track.stop();
        stream = null;
      }
      video.srcObject = null;
    }

    function showError(message) {
      error.textContent = message;
      error.hidden = false;
      status.hidden = true;
    }

    function finishWithKey(key) {
      stopCamera();
      dialog.close();

      accessKey.value = key;
      accessKey.dispatchEvent(new Event("input", { bubbles: true }));
      accessKey.dispatchEvent(new Event("change", { bubbles: true }));

      if (typeof authForm.requestSubmit === "function") {
        authForm.requestSubmit();
      } else {
        authForm.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
      }
    }

    async function startScanner() {
      error.hidden = true;
      status.hidden = false;
      status.textContent = "Starting camera…";

      if (!window.isSecureContext) {
        showError("Camera access requires HTTPS. Use the normal HTTPS Larptrix address.");
        return;
      }

      if (!navigator.mediaDevices?.getUserMedia) {
        showError("This browser does not provide camera access.");
        return;
      }

      if (!("BarcodeDetector" in window)) {
        showError(
          "This browser does not support built-in QR scanning. " +
          "You can still scan this QR with your phone camera; it will open Larptrix and sign in automatically."
        );
        return;
      }

      try {
        const formats = await BarcodeDetector.getSupportedFormats();
        if (!formats.includes("qr_code")) {
          showError("This browser has a barcode scanner, but QR codes are not supported.");
          return;
        }

        detector = new BarcodeDetector({ formats: ["qr_code"] });
        stream = await navigator.mediaDevices.getUserMedia({
          video: { facingMode: { ideal: "environment" } },
          audio: false
        });

        video.srcObject = stream;
        await video.play();
        status.textContent = "Point the camera at the Larptrix sign-in QR code…";

        timer = setInterval(async () => {
          if (!detector || video.readyState < HTMLMediaElement.HAVE_CURRENT_DATA) return;

          try {
            const results = await detector.detect(video);
            for (const result of results) {
              const key = extractAccessKey(result.rawValue);
              if (key) {
                finishWithKey(key);
                return;
              }
            }
          } catch {
            // Keep scanning; transient camera frames can fail to decode.
          }
        }, 250);
      } catch (err) {
        stopCamera();
        if (err?.name === "NotAllowedError") {
          showError("Camera permission was denied. Allow camera access and try again.");
        } else if (err?.name === "NotFoundError") {
          showError("No camera was found on this device.");
        } else {
          showError("Could not start the camera.");
        }
      }
    }

    button.addEventListener("click", () => {
      dialog.showModal();
      startScanner();
    });

    dialog.addEventListener("close", stopCamera);
    dialog.addEventListener("cancel", stopCamera);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init, { once: true });
  } else {
    init();
  }
})();
