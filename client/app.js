import initCrypto, { CryptoDevice } from "/crypto-pkg/larptrix_crypto_wasm.js";
import {
  LarptrixMatrixCrypto,
  getOrCreateMatrixDeviceId,
} from "/matrix-crypto.js";

await initCrypto();

const statusEl = document.getElementById("status");
const usersEl = document.getElementById("users");
const logEl = document.getElementById("log");
const composer = document.getElementById("composer");
const bodyInput = document.getElementById("body");
const gate = document.getElementById("gate");
const authForm = document.getElementById("auth");
const authError = document.getElementById("auth-error");
const accessKeyInput = document.getElementById("access-key");
const emailInput = document.getElementById("email");
const passwordInput = document.getElementById("password");
const legacyCredentials = document.getElementById("legacy-credentials");
const legacyLoginButton = document.getElementById("legacy-login");
const registerPasswordToggle = document.getElementById("register-password-toggle");
const nameInput = document.getElementById("display-name");
const authSubmit = document.getElementById("auth-submit");
const tabLogin = document.getElementById("tab-login");
const tabRegister = document.getElementById("tab-register");
const keyDialog = document.getElementById("key-dialog");
const generatedKey = document.getElementById("generated-key");
const keySaved = document.getElementById("key-saved");
const keyContinue = document.getElementById("key-continue");
const imageViewer = document.getElementById("image-viewer");
const imageViewerImage = document.getElementById("image-viewer-image");
const logoutBtn = document.getElementById("logout");
const meLabel = document.getElementById("me-label");
const meUsername = document.getElementById("me-username");
const meAvatar = document.getElementById("me-avatar");
const avatarFile = document.getElementById("avatar-file");
const profileOpen = document.getElementById("profile-open");
const profileDialog = document.getElementById("profile-dialog");
const profileForm = document.getElementById("profile-form");
const profileName = document.getElementById("profile-name");
const profileUsername = document.getElementById("profile-username");
const profileAbout = document.getElementById("profile-about");
const profileEmail = document.getElementById("profile-email");
const profileEmailLabel = document.getElementById("profile-email-label");
const profileActivity = document.getElementById("profile-activity");
const showMusicActivity = document.getElementById("show-music-activity");
const forgetE2eDeviceButton = document.getElementById("forget-e2e-device");
const profileCardName = document.getElementById("profile-card-name");
const profileCardHandle = document.getElementById("profile-card-handle");
const profileCardActivity = document.getElementById("profile-card-activity");
const profileError = document.getElementById("profile-error");
const issueAccessKeyButton = document.getElementById("issue-access-key");
const enableE2eButton = document.getElementById("enable-e2e");
const cryptoProfileStatus = document.getElementById("crypto-profile-status");
const cryptoOwnFingerprint = document.getElementById("crypto-own-fingerprint");
const cryptoDialog = document.getElementById("crypto-dialog");
const cryptoDialogTitle = document.getElementById("crypto-dialog-title");
const cryptoDialogDescription = document.getElementById("crypto-dialog-description");
const cryptoRecoveryDisplay = document.getElementById("crypto-recovery-key");
const cryptoRecoveryInput = document.getElementById("crypto-recovery-input");
const rememberCryptoDevice = document.getElementById("remember-crypto-device");
const rememberCryptoDeviceLabel = document.getElementById("remember-crypto-device-label");
const cryptoConfirmLabel = document.getElementById("crypto-confirm-label");
const cryptoConfirm = document.getElementById("crypto-confirm");
const cryptoError = document.getElementById("crypto-error");
const cryptoContinue = document.getElementById("crypto-continue");
const verifyDeviceDialog = document.getElementById("verify-device-dialog");
const verifyDeviceDescription = document.getElementById("verify-device-description");
const verifyDeviceFingerprint = document.getElementById("verify-device-fingerprint");
const verifyDeviceConfirm = document.getElementById("verify-device-confirm");
const verifyDeviceContinue = document.getElementById("verify-device-continue");
const photoInput = document.getElementById("photo");
const gifInput = document.getElementById("gif");
const fileInput = document.getElementById("file");
const audioFileInput = document.getElementById("audio-file");
const attachmentPreview = document.getElementById("attachment-preview");
const recordAudioButton = document.getElementById("record-audio");
const emptyEl = document.getElementById("empty");
const peerName = document.getElementById("peer-name");
const peerVerified = document.getElementById("peer-verified");
const chatTitlebar = document.getElementById("chat-titlebar");
const callStage = document.getElementById("call-stage");
const callStatus = document.getElementById("call-status");
const localVideo = document.getElementById("local-video");
const localScreenVideo = document.getElementById("local-screen-video");
const remoteVideo = document.getElementById("remote-video");
const remoteAudio = document.getElementById("remote-audio");
const enableCallAudio = document.getElementById("enable-call-audio");
const incomingCallDialog = document.getElementById("incoming-call-dialog");
const incomingCallTitle = document.getElementById("incoming-call-title");
const incomingCallKind = document.getElementById("incoming-call-kind");
const screenQuality = document.getElementById("screen-quality");
const chatBackgroundInput = document.getElementById("chat-background");
const peerProfileDialog = document.getElementById("peer-profile-dialog");
const createGroupDialog = document.getElementById("create-group-dialog");
const groupMemberList = document.getElementById("group-member-list");
const userSearchInput = document.getElementById("user-search");
const emojiPicker = document.getElementById("emoji-picker");
const menuOpenButton = document.getElementById("menu-open");
const menuCloseButton = document.getElementById("menu-close");
const menuBackdrop = document.getElementById("menu-backdrop");
const appMenu = document.getElementById("app-menu");
const menuAvatar = document.getElementById("menu-avatar");
const menuName = document.getElementById("menu-name");
const menuUsername = document.getElementById("menu-username");
const menuServer = document.getElementById("menu-server");
const menuChats = document.getElementById("menu-chats");
const menuMusic = document.getElementById("menu-music");
const menuNewGroup = document.getElementById("menu-new-group");
const menuSettings = document.getElementById("menu-settings");
const settingsDialog = document.getElementById("settings-dialog");
const settingsServer = document.getElementById("settings-server");
const settingsName = document.getElementById("settings-name");
const settingsUsername = document.getElementById("settings-username");
const settingsE2eStatus = document.getElementById("settings-e2e-status");
const settingsE2eFingerprint = document.getElementById("settings-e2e-fingerprint");
const settingsLayoutStatus = document.getElementById("settings-layout-status");


let socket = null;
let me = null;
let peerId = null;
let users = [];
let groups = [];
let reconnect = false;
let mode = "login";
let legacyLogin = false;
let registerWithPassword = false;
let pendingKeyUser = null;
let pendingAttachment = null;
let previewUrl = null;
let recorder = null;
let recordingStream = null;
let recordedChunks = [];
let cryptoDevice = null;
let cryptoDeviceBundle = null;
let cryptoStoredState = null;
let cryptoRecoveryKey = null;
let cryptoDialogMode = null;
let cryptoEnabled = false;
let cryptoReady = Promise.resolve();
let cryptoLoadResolve = null;
let customActivity = "";
let musicActivityEnabled = localStorage.getItem("larptrix_show_music_activity") !== "0";
let matrixCrypto = null;
let matrixServerName = null;
let matrixCryptoReady = Promise.resolve(false);

let cryptoStateQueue = Promise.resolve();

function withCryptoStateLock(task) {
  const run = cryptoStateQueue.then(task, task);
  cryptoStateQueue = run.catch(() => {});
  return run;
}
let peerVerificationResolve = null;

const sentPlaintextByCiphertext = new Map();
const cryptoRecoveryLastAttempt = new Map();
const cryptoRecoveryPending = new Map();
const recoveredBodiesByMessageId = new Map();
const cryptoRecoveryResponsesByMessageId = new Map();
const messageBodyElementsById = new Map();
const messagesById = new Map();

async function sentPlaintextCacheId(ciphertext) {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(ciphertext));
  return `sent-plaintext:${[...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("")}`;
}

async function sentPlaintextCacheKey() {
  if (!cryptoRecoveryKey) return null;
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(cryptoRecoveryKey));
  return crypto.subtle.importKey("raw", digest, { name: "AES-GCM" }, false, ["encrypt", "decrypt"]);
}

async function cacheSentPlaintext(ciphertext, payload) {
  try {
    const key = await sentPlaintextCacheKey();
    if (!key) return;
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv },
      key,
      new TextEncoder().encode(payload),
    );
    await writeLocalCryptoRecord({
      id: await sentPlaintextCacheId(ciphertext),
      iv: Array.from(iv),
      ciphertext: Array.from(new Uint8Array(encrypted)),
    });
  } catch {}
}

async function loadCachedSentPlaintext(ciphertext) {
  try {
    const key = await sentPlaintextCacheKey();
    if (!key) return null;
    const record = await readLocalCryptoRecord(await sentPlaintextCacheId(ciphertext));
    if (!record) return null;
    const plaintext = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: new Uint8Array(record.iv) },
      key,
      new Uint8Array(record.ciphertext),
    );
    return new TextDecoder().decode(plaintext);
  } catch {
    return null;
  }
}

let peerConnection = null;
let localMediaStream = null;
let screenMediaStream = null;
let pendingIncomingCall = null;
let callPeerId = null;
let callMediaKind = null;
let callMediaNotice = "";
let pendingIceCandidates = [];
const iceCandidatesBeforeOffer = new Map();

function closeAppMenu() {
  appMenu.hidden = true;
  menuBackdrop.hidden = true;
  menuOpenButton.setAttribute("aria-expanded", "false");
  document.body.classList.remove("menu-open");
}

function openAppMenu() {
  if (!me) return;
  renderMenuAccount();
  appMenu.hidden = false;
  menuBackdrop.hidden = false;
  menuOpenButton.setAttribute("aria-expanded", "true");
  document.body.classList.add("menu-open");
}

function renderMenuAccount() {
  if (!me) return;
  menuName.textContent = me.display_name || "";
  menuUsername.textContent = me.username ? "@" + me.username : "";
  menuAvatar.replaceChildren();
  if (me.avatar_url) {
    const image = document.createElement("img");
    image.src = me.avatar_url;
    image.alt = "";
    menuAvatar.append(image);
  } else {
    menuAvatar.textContent = (me.display_name || "?").trim().charAt(0).toUpperCase() || "?";
  }
  menuServer.textContent = location.host;
}

function openSettings() {
  if (!me) return;
  settingsServer.value = location.host;
  settingsName.textContent = me.display_name || "—";
  settingsUsername.textContent = me.username ? "@" + me.username : "No username";
  const unlocked = cryptoEnabled && Boolean(cryptoDevice);
  settingsE2eStatus.textContent = !cryptoEnabled
    ? "E2E is not configured."
    : unlocked
      ? "E2E is enabled and unlocked on this device."
      : "E2E is enabled, but this device is locked.";
  settingsE2eFingerprint.hidden = !cryptoDeviceBundle?.fingerprint;
  settingsE2eFingerprint.textContent = cryptoDeviceBundle?.fingerprint || "";
  settingsLayoutStatus.textContent = "";
  settingsDialog.showModal();
  closeAppMenu();
}

function resetChatListWidth() {
  localStorage.removeItem("larptrix_chat_list_width");
  document.documentElement.style.setProperty("--chat-list-width", "280px");
  settingsLayoutStatus.textContent = "Chat list width reset to 280px.";
}

menuOpenButton.addEventListener("click", () => {
  if (appMenu.hidden) openAppMenu();
  else closeAppMenu();
});
menuCloseButton.addEventListener("click", closeAppMenu);
menuBackdrop.addEventListener("click", closeAppMenu);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !appMenu.hidden) {
    event.preventDefault();
    closeAppMenu();
  }
});
menuChats.addEventListener("click", () => {
  document.getElementById("chat-view-open").click();
  closeAppMenu();
});
menuMusic.addEventListener("click", () => {
  document.getElementById("music-library-open").click();
  closeAppMenu();
});
menuNewGroup.addEventListener("click", () => {
  document.getElementById("create-group-open").click();
  closeAppMenu();
});
profileOpen.addEventListener("click", async () => {
  closeAppMenu();
  try {
    const profile = await api("GET", `/api/users/${encodeURIComponent(me.user_id)}/profile`);
    profileName.value = profile.display_name;
    profileUsername.value = profile.username;
    profileAbout.value = profile.about;
    updateOwnProfileCard(profile);
    customActivity = profile.activity?.startsWith("Listening to ") ? "" : (profile.activity || "");
    profileActivity.value = customActivity;
    showMusicActivity.checked = musicActivityEnabled;
    profileDialog.showModal();
  } catch (err) {
    profileError.textContent = err.message;
    profileError.hidden = false;
  }
});
menuSettings.addEventListener("click", openSettings);
document.getElementById("settings-close").addEventListener("click", () => settingsDialog.close());
document.getElementById("settings-done").addEventListener("click", () => settingsDialog.close());
document.getElementById("settings-open-profile").addEventListener("click", () => {
  settingsDialog.close();
  profileOpen.click();
});
document.getElementById("settings-open-profile-e2e").addEventListener("click", () => {
  settingsDialog.close();
  profileOpen.click();
  setTimeout(() => enableE2eButton.click(), 0);
});
document.getElementById("settings-reset-layout").addEventListener("click", resetChatListWidth);

tabLogin.addEventListener("click", () => setMode("login"));
tabRegister.addEventListener("click", () => setMode("register"));
legacyLoginButton.addEventListener("click", toggleLegacyLogin);
registerPasswordToggle.addEventListener("click", () => {
  registerWithPassword = !registerWithPassword;
  accessKeyInput.hidden = true;
  legacyCredentials.hidden = !registerWithPassword;
  emailInput.required = registerWithPassword;
  passwordInput.required = registerWithPassword;
  registerPasswordToggle.textContent = registerWithPassword
    ? "Create account with an access key instead"
    : "Create account with email and password";
  authSubmit.textContent = registerWithPassword ? "Create account" : "Create account with key";
});
keySaved.addEventListener("change", () => {
  keyContinue.disabled = !keySaved.checked;
});
keyDialog.addEventListener("cancel", (event) => event.preventDefault());
document.getElementById("copy-key").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(generatedKey.textContent);
    document.getElementById("copy-key").textContent = "Copied";
  } catch {
    generatedKey.focus?.();
    appendSystem("Copy failed. Select and copy the key manually.");
  }
});
keyContinue.addEventListener("click", async () => {
  if (!pendingKeyUser || !keySaved.checked) return;
  keyContinue.disabled = true;
  const keyError = document.getElementById("key-error");
  keyError.hidden = true;
  try {
    const user = await api("POST", "/api/login", {
      access_key: generatedKey.textContent,
    });
    const alreadyConnected = socket?.readyState === WebSocket.OPEN && me?.user_id === user.user_id;
    keyDialog.close();
    generatedKey.textContent = "";
    pendingKeyUser = null;
    if (alreadyConnected) {
      me = user;
      renderMe();
    } else {
      signedIn(user);
    }
  } catch (err) {
    keyError.textContent = err.message;
    keyError.hidden = false;
    keyContinue.disabled = false;
  }
});
document.getElementById("image-viewer-close").addEventListener("click", () => imageViewer.close());
imageViewer.addEventListener("click", (event) => {
  if (event.target === imageViewer) imageViewer.close();
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && imageViewer.open) {
    event.preventDefault();
    imageViewer.close();
  }
});
document.getElementById("peer-profile-open").addEventListener("click", () => {
  if (peerId) void showPeerProfile(peerId);
});
document.getElementById("peer-profile-close").addEventListener("click", () => peerProfileDialog.close());
forgetE2eDeviceButton.addEventListener("click", async () => {
  if (!me) return;
  await forgetRememberedCryptoKey(me.user_id);
  cryptoProfileStatus.textContent = "This device will ask for the recovery key at the next sign-in.";
  forgetE2eDeviceButton.hidden = true;
});
document.getElementById("create-group-open").addEventListener("click", () => {
  renderGroupMemberChoices();
  document.getElementById("group-create-error").hidden = true;
  document.getElementById("group-name").value = "";
  createGroupDialog.showModal();
});
document.getElementById("create-group-cancel").addEventListener("click", () => createGroupDialog.close());
document.getElementById("create-group-form").addEventListener("submit", createGroup);
userSearchInput.addEventListener("input", renderUsers);
document.getElementById("emoji-picker-toggle").addEventListener("click", () => {
  emojiPicker.hidden = !emojiPicker.hidden;
});
emojiPicker.addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (!button) return;
  const start = bodyInput.selectionStart;
  const end = bodyInput.selectionEnd;
  bodyInput.setRangeText(button.textContent, start, end, "end");
  bodyInput.focus();
  emojiPicker.hidden = true;
});
chatBackgroundInput.addEventListener("change", () => {
  const file = chatBackgroundInput.files[0];
  if (file && peerId) void saveChatWallpaper(file, peerId);
  chatBackgroundInput.value = "";
});
document.getElementById("chat-background-clear").addEventListener("click", () => {
  if (!peerId) return;
  localStorage.removeItem(chatWallpaperKey(peerId));
  applyChatWallpaper(peerId);
});
document.getElementById("profile-close").addEventListener("click", () => profileDialog.close());
issueAccessKeyButton.addEventListener("click", async () => {
  issueAccessKeyButton.disabled = true;
  profileError.hidden = true;
  try {
    const result = await api("POST", "/api/me/access-key", {});
    showKeyDialog(result.access_key, me);
  } catch (err) {
    profileError.textContent = err.message;
    profileError.hidden = false;
  } finally {
    issueAccessKeyButton.disabled = false;
  }
});
enableE2eButton.addEventListener("click", () => {
  if (cryptoEnabled) {
    if (cryptoDevice) {
      cryptoProfileStatus.textContent = "E2E is already unlocked on this device.";
      return;
    }
    openCryptoDialog("unlock");
    return;
  }
  try {
    cryptoRecoveryKey = createRecoveryKey();
    cryptoDevice = new CryptoDevice(cryptoRecoveryKey);
    cryptoDeviceBundle = JSON.parse(cryptoDevice.public_bundle_json());
    openCryptoDialog("enable");
  } catch (err) {
    profileError.textContent = err.message || "Could not initialize E2E encryption.";
    profileError.hidden = false;
  }
});
cryptoConfirm.addEventListener("change", () => {
  cryptoContinue.disabled = !cryptoConfirm.checked;
});
cryptoRecoveryInput.addEventListener("input", () => {
  cryptoContinue.disabled = cryptoRecoveryInput.value.trim().length < 40;
});
document.getElementById("crypto-cancel").addEventListener("click", () => {
  cryptoDialog.close();
  if (cryptoDialogMode === "unlock") {
    cryptoLoadResolve?.(false);
    cryptoLoadResolve = null;
  } else if (cryptoDialogMode === "enable") {
    cryptoDevice?.free();
    cryptoDevice = null;
    cryptoDeviceBundle = null;
    cryptoRecoveryKey = null;
    cryptoLoadResolve?.(false);
    cryptoLoadResolve = null;
  }
  cryptoDialogMode = null;
});
cryptoDialog.addEventListener("cancel", (event) => {
  event.preventDefault();
  document.getElementById("crypto-cancel").click();
});
cryptoContinue.addEventListener("click", completeCryptoDialog);
document.getElementById("copy-crypto-recovery").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(cryptoRecoveryDisplay.textContent);
    document.getElementById("copy-crypto-recovery").textContent = "Copied";
  } catch {
    cryptoError.textContent = "Copy failed. Select the recovery key and copy it manually.";
    cryptoError.hidden = false;
  }
});
verifyDeviceConfirm.addEventListener("change", () => {
  verifyDeviceContinue.disabled = !verifyDeviceConfirm.checked;
});
document.getElementById("verify-device-cancel").addEventListener("click", () => {
  verifyDeviceDialog.close();
  peerVerificationResolve?.(false);
  peerVerificationResolve = null;
});
verifyDeviceDialog.addEventListener("cancel", (event) => {
  event.preventDefault();
  document.getElementById("verify-device-cancel").click();
});
verifyDeviceContinue.addEventListener("click", () => {
  if (!verifyDeviceConfirm.checked) return;
  const peerIdToVerify = verifyDeviceDialog.dataset.peerId;
  const peerDeviceIdToVerify = verifyDeviceDialog.dataset.peerDeviceId;

  if (!peerIdToVerify || !peerDeviceIdToVerify) return;

  localStorage.setItem(
    verifiedFingerprintKey(peerIdToVerify, peerDeviceIdToVerify),
    verifyDeviceFingerprint.textContent
  );

  setPeerVerified(peerIdToVerify, true);
  verifyDeviceDialog.close();
  peerVerificationResolve?.(true);
  peerVerificationResolve = null;
});

profileForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  profileError.hidden = true;
  try {
    me = await api("PATCH", "/api/me", {
      display_name: profileName.value.trim(),
      username: profileUsername.value.trim(),
      about: profileAbout.value.trim(),
    });
    customActivity = profileActivity.value.trim();
    musicActivityEnabled = showMusicActivity.checked;
    localStorage.setItem("larptrix_show_music_activity", musicActivityEnabled ? "1" : "0");
    await setMyActivity(customActivity);
    window.larptixMusicStatus?.refresh?.();
    renderMe();
    updateOwnProfileCard({ ...me, activity: profileCardActivity.textContent });
    profileDialog.close();
  } catch (err) {
    profileError.textContent = err.message;
    profileError.hidden = false;
  }
});

authForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  showAuthError("");
  try {
    if (mode === "register") {
      if (registerWithPassword) {
        const user = await api("POST", "/api/register/password", {
          display_name: nameInput.value.trim(),
          email: emailInput.value.trim(),
          password: passwordInput.value,
        });
        signedIn(user);
        return;
      }
      const result = await api("POST", "/api/register", {
        display_name: nameInput.value.trim(),
      });
      gate.close();
      showKeyDialog(result.access_key, result.user);
      return;
    }
    const payload = legacyLogin
      ? { email: emailInput.value.trim(), password: passwordInput.value }
      : { access_key: accessKeyInput.value.trim() };
    const user = await api("POST", "/api/login", payload);
    signedIn(user);
  } catch (err) {
    showAuthError(err.message);
  }
});

logoutBtn.addEventListener("click", async () => {
  reconnect = false;
  if (socket) socket.close();
  const signedOutUserId = me?.user_id;
  if (signedOutUserId) await api("POST", "/api/me/activity", { activity: "" }).catch(() => {});
  await api("POST", "/api/logout", {});
  me = null;
  peerId = null;
  logoutBtn.hidden = true;
  composer.hidden = true;
  closeAppMenu();
  meLabel.textContent = "";
  meUsername.textContent = "";
  meAvatar.textContent = "?";
  meAvatar.replaceChildren();
  await matrixCrypto?.close().catch(() => {});
  matrixCrypto = null;
  matrixServerName = null;
  matrixCryptoReady = Promise.resolve(false);
  cryptoDevice?.free();
  cryptoDevice = null;
  cryptoDeviceBundle = null;
  cryptoStoredState = null;
  cryptoRecoveryKey = null;
  cryptoEnabled = false;
  cryptoProfileStatus.textContent = "End-to-end encryption is required for all messages.";
  cryptoOwnFingerprint.hidden = true;
  setMode("login");
  gate.showModal();
});

composer.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!peerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  const body = bodyInput.value.trim();
  const file = pendingAttachment;
  if (cryptoEnabled) {
    try {
      await cryptoReady;
      if (!cryptoDevice) throw new Error("Unlock E2E with your separate recovery key before sending messages.");
      if (!body && !file) return;
      const peer = [...users, ...groups].find((item) => item.user_id === peerId);
      if (!peer) throw new Error("Chat peer is not in the current list.");
      let attachment_id = null;
      let encryptedFile = null;
      if (file) {
        const encrypted = await encryptAttachment(file);
        const uploaded = await uploadFile("/api/upload", encrypted.file);
        attachment_id = uploaded.id;
        encryptedFile = encrypted.metadata;
      }
      const payload = JSON.stringify({ text: body, file: encryptedFile });
      let encryptedBody;

      await withCryptoStateLock(async () => {
        if (peer.is_group) {
          await matrixCryptoReady;
          if (!matrixCrypto) {
            throw new Error("Matrix E2E is not initialized for this device.");
          }

          const roomId = matrixCrypto.groupRoomId(peer.user_id);
          await matrixCrypto.prepareRoom(roomId, peer.group_member_ids);
          const ciphertext = await matrixCrypto.encrypt(roomId, payload);

          encryptedBody = JSON.stringify({
            version: 3,
            message_type: "matrix",
            sender_device_id: matrixCrypto.deviceId,
            room_id: roomId,
            ciphertext,
          });
        } else {
          await matrixCryptoReady;

          if (matrixCrypto) {
            const roomId = await matrixCrypto.roomIdForDm(peerId);
            await matrixCrypto.prepareRoom(roomId, [peerId]);
            const ciphertext = await matrixCrypto.encrypt(roomId, payload);

            encryptedBody = JSON.stringify({
              version: 3,
              message_type: "matrix",
              sender_device_id: matrixCrypto.deviceId,
              room_id: roomId,
              ciphertext,
            });
          } else {
            const result = await api(
              "GET",
              `/api/users/${encodeURIComponent(peerId)}/crypto-devices`
            );

            const devices = Array.isArray(result?.devices) ? result.devices : [];

            if (devices.length === 0) {
              throw new Error("Peer has no E2E devices.");
            }

            const ciphertexts = {};

            for (const bundle of devices) {
              if (!bundle || typeof bundle.device_id !== "string" || !bundle.device_id) {
                throw new Error("Peer has an invalid E2E device bundle.");
              }

              if (!(await ensurePeerFingerprint(peer, bundle))) {
                throw new Error(`Device ${bundle.device_id} could not be verified.`);
              }

              const deviceId = bundle.device_id;

              if (!cryptoDevice.has_session(deviceId)) {
                const claimedBundle = await claimPeerOneTimeKey(peerId, deviceId);
                if (!(await ensurePeerFingerprint(peer, claimedBundle))) {
                  throw new Error(`Device ${deviceId} could not be verified.`);
                }
                cryptoDevice.establish_session(
                  deviceId,
                  JSON.stringify(claimedBundle),
                  claimedBundle.fingerprint
                );
              }

              ciphertexts[deviceId] = cryptoDevice.encrypt(
                deviceId,
                payload
              );
            }

            encryptedBody = JSON.stringify({
              version: 2,
              message_type: "message",
              sender_device_id: cryptoDevice.device_id(),
              ciphertexts,
            });
          }
        }

        await persistCryptoState();
      });
      sentPlaintextByCiphertext.set(encryptedBody, payload);
      void cacheSentPlaintext(encryptedBody, payload);
      socket.send(JSON.stringify({
        type: "send",
        peer_id: peerId,
        body: encryptedBody,
        attachment_id,
      }));
      bodyInput.value = "";
      clearAttachment();
    } catch (err) {
      appendSystem(err.message || "Could not encrypt the message.");
    }
    return;
  }
  appendSystem("Set up E2E before sending messages.");
});

photoInput.addEventListener("change", () => queueAttachment(photoInput.files[0]));
gifInput.addEventListener("change", () => queueAttachment(gifInput.files[0]));
fileInput.addEventListener("change", () => queueAttachment(fileInput.files[0]));
audioFileInput.addEventListener("change", () => queueAttachment(audioFileInput.files[0]));
recordAudioButton.addEventListener("click", toggleRecording);
document.getElementById("start-audio-call").addEventListener("click", () => startCall("audio"));
document.getElementById("start-video-call").addEventListener("click", () => startCall("video"));
document.getElementById("accept-call").addEventListener("click", acceptIncomingCall);
document.getElementById("reject-call").addEventListener("click", rejectIncomingCall);
document.getElementById("end-call").addEventListener("click", () => endCall(true));
enableCallAudio.addEventListener("click", () => {
  remoteAudio.play().then(() => {
    enableCallAudio.hidden = true;
  }).catch((err) => appendSystem(err.message || "Could not play call audio."));
});
document.getElementById("toggle-microphone").addEventListener("click", toggleMicrophone);
document.getElementById("call-collapse").addEventListener("click", (event) => {
  callStage.classList.toggle("call-collapsed");
  const collapsed = callStage.classList.contains("call-collapsed");
  event.currentTarget.textContent = collapsed ? "□" : "−";
  event.currentTarget.title = collapsed ? "Restore call" : "Minimize call";
});
document.getElementById("toggle-camera").addEventListener("click", toggleCamera);
document.getElementById("toggle-screen-share").addEventListener("click", toggleScreenShare);
document.getElementById("toggle-call-fullscreen").addEventListener("click", async () => {
  try {
    if (document.fullscreenElement) await document.exitFullscreen();
    else await document.getElementById("call-videos").requestFullscreen();
  } catch (err) {
    appendSystem(err.message || "Fullscreen is not available in this browser context.");
  }
});

avatarFile.addEventListener("change", async () => {
  const file = avatarFile.files[0];
  if (!file) return;
  try {
    const user = await uploadFile("/api/me/avatar", file);
    me = {
      ...user,
      avatar_url: user.avatar_url
        ? `${user.avatar_url}?v=${Date.now()}`
        : user.avatar_url,
    };
    renderMe();
  } catch (err) {
    appendSystem(err.message);
  }
  avatarFile.value = "";
});

bootstrap();

async function bootstrap() {
  try {
    const user = await api("GET", "/api/me");
    signedIn(user);
  } catch {
    gate.showModal();
  }
}

function showKeyDialog(accessKey, user) {
  pendingKeyUser = user;
  generatedKey.textContent = accessKey;
  document.getElementById("key-error").hidden = true;
  keySaved.checked = false;
  keyContinue.disabled = true;
  document.getElementById("copy-key").textContent = "Copy key";
  if (profileDialog.open) profileDialog.close();
  keyDialog.showModal();
}

function createRecoveryKey() {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  return btoa(String.fromCharCode(...bytes)).replace(/=+$/, "");
}

function openCryptoDialog(mode) {
  cryptoDialogMode = mode;
  cryptoError.hidden = true;
  cryptoConfirm.checked = false;
  cryptoRecoveryInput.value = "";
  cryptoRecoveryDisplay.hidden = mode !== "enable";
  const copyRecoveryButton = document.getElementById("copy-crypto-recovery");
  copyRecoveryButton.hidden = mode !== "enable";
  copyRecoveryButton.textContent = "Copy recovery key";
  cryptoRecoveryInput.hidden = mode !== "unlock";
  rememberCryptoDeviceLabel.hidden = mode !== "unlock";
  rememberCryptoDevice.checked = true;
  cryptoConfirmLabel.hidden = mode !== "enable";
  if (mode === "enable") {
    cryptoDialogTitle.textContent = "Save your E2E recovery key";
    cryptoDialogDescription.textContent = "E2E is required for messaging. Save this separate recovery key: it unlocks your encrypted chats on other devices.";
    cryptoRecoveryDisplay.textContent = cryptoRecoveryKey;
    cryptoContinue.textContent = "Enable E2E";
    cryptoContinue.disabled = false;
  } else {
    cryptoDialogTitle.textContent = "Unlock encrypted chats";
    cryptoDialogDescription.textContent = "Enter the separate recovery key you saved when enabling E2E. It is not your sign-in key.";
    cryptoContinue.textContent = "Unlock chats";
    cryptoContinue.disabled = true;
  }
  cryptoDialog.showModal();
}

async function completeCryptoDialog() {
  cryptoContinue.disabled = true;
  cryptoError.hidden = true;
  try {
    if (cryptoDialogMode === "enable") {
      await persistCryptoState();
      cryptoEnabled = true;
      cryptoProfileStatus.textContent = "E2E is required for every chat. Verify each peer fingerprint before messaging.";
      enableE2eButton.textContent = "E2E enabled";
      cryptoOwnFingerprint.textContent = cryptoDeviceBundle.fingerprint;
      cryptoOwnFingerprint.hidden = false;
      cryptoDialog.close();
      cryptoDialogMode = null;
      try {
        await rememberRecoveryKey(me.user_id, cryptoRecoveryKey);
        forgetE2eDeviceButton.hidden = false;
      } catch {
        cryptoProfileStatus.textContent = "E2E is enabled, but this device could not be remembered.";
      }
      cryptoLoadResolve?.(true);
      cryptoLoadResolve = null;
      if (peerId) openChat(peerId);
      return;
    }

    const recoveryKey = cryptoRecoveryInput.value.trim();

    await withCryptoStateLock(async () => {
      const restored = CryptoDevice.restore(recoveryKey, cryptoStoredState.encrypted_state);
      const bundle = JSON.parse(cryptoStoredState.bundle_json);
      const restoredBundle = JSON.parse(restored.public_bundle_json());
      if (restoredBundle.fingerprint !== bundle.fingerprint) {
        restored.free();
        throw new Error("Recovery key restored a different device identity. Check the recovery key and backup.");
      }
      cryptoDevice?.free();
      cryptoDevice = restored;
      cryptoRecoveryKey = recoveryKey;
      cryptoDeviceBundle = restoredBundle;
      cryptoOwnFingerprint.textContent = restoredBundle.fingerprint;
      cryptoOwnFingerprint.hidden = false;
      cryptoProfileStatus.textContent = "E2E enabled and unlocked on this device.";
      cryptoDialog.close();
      cryptoDialogMode = null;
      await persistCryptoState();
    });
    try {
      if (rememberCryptoDevice.checked) {
        await rememberRecoveryKey(me.user_id, recoveryKey);
        forgetE2eDeviceButton.hidden = false;
      } else {
        await forgetRememberedCryptoKey(me.user_id);
        forgetE2eDeviceButton.hidden = true;
      }
    } catch {
      cryptoProfileStatus.textContent = "E2E is unlocked, but this device could not be remembered.";
    }
    cryptoLoadResolve?.(true);
    cryptoLoadResolve = null;
    if (peerId) openChat(peerId);
  } catch (err) {
    cryptoError.textContent = err.message || "Could not enable or unlock E2E.";
    cryptoError.hidden = false;
    cryptoContinue.disabled = false;
  }
}

async function loadCryptoStatus() {
  try {
    cryptoStoredState = await api("GET", "/api/me/crypto-device");
  } catch (err) {
    if (err.status === 404) {
      cryptoEnabled = false;
      cryptoProfileStatus.textContent = "Set up E2E before sending or reading messages.";
      enableE2eButton.textContent = "Set up E2E";
      cryptoRecoveryKey = createRecoveryKey();
      cryptoDevice = new CryptoDevice(cryptoRecoveryKey);
      cryptoDeviceBundle = JSON.parse(cryptoDevice.public_bundle_json());
      openCryptoDialog("enable");
      return new Promise((resolve) => {
        cryptoLoadResolve = resolve;
      });
    }
    throw err;
  }
  cryptoEnabled = true;
  const bundle = JSON.parse(cryptoStoredState.bundle_json);
  cryptoDeviceBundle = bundle;
  cryptoOwnFingerprint.textContent = bundle.fingerprint;
  cryptoOwnFingerprint.hidden = false;
  let rememberedKey = null;
  try {
    rememberedKey = await loadRememberedRecoveryKey(me.user_id);
  } catch {
    await forgetRememberedCryptoKey(me.user_id);
  }
  if (rememberedKey) {
    try {
      const restored = CryptoDevice.restore(rememberedKey, cryptoStoredState.encrypted_state);
      const restoredBundle = JSON.parse(restored.public_bundle_json());
      if (restoredBundle.fingerprint !== bundle.fingerprint) {
        restored.free();
        throw new Error("Remembered key does not match this device.");
      }
      cryptoDevice?.free();
      cryptoDevice = restored;
      cryptoRecoveryKey = rememberedKey;
      cryptoDeviceBundle = restoredBundle;
      cryptoProfileStatus.textContent = "E2E enabled and unlocked on this device.";
      enableE2eButton.textContent = "E2E enabled";
      forgetE2eDeviceButton.hidden = false;
      return true;
    } catch {
      await forgetRememberedCryptoKey(me.user_id);
    }
  }
  cryptoProfileStatus.textContent = "E2E enabled. Enter your recovery key to unlock encrypted chats.";
  enableE2eButton.textContent = "Unlock E2E device";
  cryptoDialogMode = "unlock";
  cryptoLoadResolve = null;
  openCryptoDialog("unlock");
  return new Promise((resolve) => {
    cryptoLoadResolve = resolve;
  });
}

async function loadMatrixCryptoStatus() {
  if (!me || !cryptoRecoveryKey) return false;

  const config = await api("GET", "/api/matrix/config");
  matrixServerName = config.server_name;
  if (!matrixServerName || typeof matrixServerName !== "string") {
    throw new Error("Matrix server name is unavailable.");
  }

  const deviceId = getOrCreateMatrixDeviceId(me.user_id);
  const next = new LarptrixMatrixCrypto({
    api,
    userId: me.user_id,
    serverName: matrixServerName,
    deviceId,
    storePassphrase: cryptoRecoveryKey,
  });

  try {
    await next.initialize();
  } catch (err) {
    await next.close().catch(() => {});
    throw err;
  }

  if (matrixCrypto) await matrixCrypto.close().catch(() => {});
  matrixCrypto = next;
  return true;
}

async function persistCryptoState() {
  if (!cryptoDevice) return;

  // Build the public bundle before publishing so all currently
  // available unpublished OTKs are included in the bundle.
  const bundleJson = cryptoDevice.public_bundle_json();

  // Move those OTKs into the published public-key cache.
  // The private OTK material remains inside AccountPickle.
  cryptoDevice.mark_one_time_keys_as_published();

  // Persist the exact public bundle together with the updated
  // encrypted state. This keeps the published OTK metadata and
  // the private Account state synchronized.
  cryptoDeviceBundle = JSON.parse(bundleJson);
  cryptoOwnFingerprint.textContent = cryptoDeviceBundle.fingerprint;

  const encryptedState = cryptoDevice.encrypted_state_json();
  const deviceId = cryptoDevice.device_id();

  if (!cryptoStoredState) {
    const created = await api("POST", "/api/me/crypto-devices", {
      device_id: deviceId,
      bundle_json: bundleJson,
      encrypted_state: encryptedState,
    });

    cryptoStoredState = {
      device_id: created.device_id,
      bundle_json: bundleJson,
      encrypted_state: encryptedState,
      state_version: created.state_version,
    };
    return;
  }

  const updated = await api(
    "PUT",
    `/api/me/crypto-devices/${encodeURIComponent(deviceId)}`,
    {
      bundle_json: bundleJson,
      encrypted_state: encryptedState,
      expected_version: cryptoStoredState.state_version,
    }
  );

  cryptoStoredState = {
    ...cryptoStoredState,
    bundle_json: bundleJson,
    encrypted_state: encryptedState,
    state_version: updated.state_version,
    updated_at: updated.updated_at,
  };
}

function openLocalCryptoDb() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("larptrix-e2e-device", 1);
    request.addEventListener("upgradeneeded", () => {
      request.result.createObjectStore("items", { keyPath: "id" });
    });
    request.addEventListener("success", () => resolve(request.result), { once: true });
    request.addEventListener("error", () => reject(request.error), { once: true });
  });
}

async function readLocalCryptoRecord(id) {
  const db = await openLocalCryptoDb();
  try {
    const request = db.transaction("items", "readonly").objectStore("items").get(id);
    return await new Promise((resolve, reject) => {
      request.addEventListener("success", () => resolve(request.result), { once: true });
      request.addEventListener("error", () => reject(request.error), { once: true });
    });
  } finally {
    db.close();
  }
}

async function writeLocalCryptoRecord(record) {
  const db = await openLocalCryptoDb();
  try {
    await new Promise((resolve, reject) => {
      const transaction = db.transaction("items", "readwrite");
      transaction.objectStore("items").put(record);
      transaction.addEventListener("complete", resolve, { once: true });
      transaction.addEventListener("error", () => reject(transaction.error), { once: true });
      transaction.addEventListener("abort", () => reject(transaction.error), { once: true });
    });
  } finally {
    db.close();
  }
}

function rememberedKeyOptOut(userId) {
  return `larptrix_e2e_remember_disabled_${userId}`;
}

async function rememberRecoveryKey(userId, recoveryKey) {
  let wrappingKey = (await readLocalCryptoRecord("wrapping-key"))?.value;
  if (!wrappingKey) {
    wrappingKey = await crypto.subtle.generateKey(
      { name: "AES-GCM", length: 256 },
      false,
      ["encrypt", "decrypt"],
    );
    await writeLocalCryptoRecord({ id: "wrapping-key", value: wrappingKey });
  }
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    wrappingKey,
    new TextEncoder().encode(recoveryKey),
  );
  await writeLocalCryptoRecord({
    id: `recovery-key:${userId}`,
    iv: bytesToBase64(iv),
    ciphertext: bytesToBase64(new Uint8Array(ciphertext)),
  });
  localStorage.removeItem(rememberedKeyOptOut(userId));
}

async function loadRememberedRecoveryKey(userId) {
  if (typeof indexedDB === "undefined" || localStorage.getItem(rememberedKeyOptOut(userId)) === "1") {
    return null;
  }
  const [keyRecord, savedRecord] = await Promise.all([
    readLocalCryptoRecord("wrapping-key"),
    readLocalCryptoRecord(`recovery-key:${userId}`),
  ]);
  if (!keyRecord?.value || !savedRecord?.iv || !savedRecord?.ciphertext) return null;
  const plaintext = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: base64ToBytes(savedRecord.iv) },
    keyRecord.value,
    base64ToBytes(savedRecord.ciphertext),
  );
  return new TextDecoder().decode(plaintext);
}

async function forgetRememberedCryptoKey(userId) {
  localStorage.setItem(rememberedKeyOptOut(userId), "1");
  if (typeof indexedDB === "undefined") return;
  let db;
  try {
    db = await openLocalCryptoDb();
    await new Promise((resolve, reject) => {
      const transaction = db.transaction("items", "readwrite");
      transaction.objectStore("items").delete(`recovery-key:${userId}`);
      transaction.addEventListener("complete", resolve, { once: true });
      transaction.addEventListener("error", () => reject(transaction.error), { once: true });
      transaction.addEventListener("abort", () => reject(transaction.error), { once: true });
    });
  } catch {
    return;
  } finally {
    db?.close();
  }
}

function verifiedFingerprintKey(userId, deviceId) {
  return `larptrix_verified_device_${me.user_id}_${userId}_${deviceId}`;
}

async function claimPeerOneTimeKey(userId, deviceId) {
  const result = await api(
    "POST",
    `/api/users/${encodeURIComponent(userId)}/crypto-devices/${encodeURIComponent(deviceId)}/claim-one-time-key`,
    {}
  );
  if (!result?.bundle) {
    throw new Error(`No one-time key is available for device ${deviceId}.`);
  }
  return result.bundle;
}

async function ensurePeerFingerprint(peerUser, bundle) {
  if (!bundle || typeof bundle.device_id !== "string" || !bundle.device_id) {
    return false;
  }

  const key = verifiedFingerprintKey(peerUser.user_id, bundle.device_id);

  if (localStorage.getItem(key) === bundle.fingerprint) {
    setPeerVerified(peerUser.user_id, true);
    return true;
  }

  verifyDeviceDescription.textContent =
    `Compare this full device fingerprint with ${peerUser.display_name} through another trusted channel, then confirm. This verifies device ${bundle.device_id}. A mismatch may mean the server substituted a device key.`;

  verifyDeviceFingerprint.textContent = bundle.fingerprint;
  verifyDeviceDialog.dataset.peerId = peerUser.user_id;
  verifyDeviceDialog.dataset.peerDeviceId = bundle.device_id;
  verifyDeviceConfirm.checked = false;
  verifyDeviceContinue.disabled = true;
  verifyDeviceDialog.showModal();

  return new Promise((resolve) => {
    peerVerificationResolve = resolve;
  });
}

function setPeerVerified(userId, verified) {
  if (peerId === userId) peerVerified.hidden = !verified;
}

async function refreshPeerVerification(userId) {
  peerVerified.hidden = true;

  try {
    const result = await api(
      "GET",
      `/api/users/${encodeURIComponent(userId)}/crypto-devices`
    );

    const devices = Array.isArray(result?.devices) ? result.devices : [];

    const verified = devices.length > 0 && devices.every((bundle) =>
      bundle
      && typeof bundle.device_id === "string"
      && localStorage.getItem(
        verifiedFingerprintKey(userId, bundle.device_id)
      ) === bundle.fingerprint
    );

    setPeerVerified(userId, verified);
  } catch {
    setPeerVerified(userId, false);
  }
}

function signedIn(user) {
  me = user;
  forgetE2eDeviceButton.hidden = true;
  gate.close();
  logoutBtn.hidden = false;
  renderMe();
  cryptoReady = loadCryptoStatus().catch((err) => {
    cryptoProfileStatus.textContent = `E2E setup error: ${err.message}`;
    return false;
  });
  matrixCryptoReady = cryptoReady.then(async (ready) => {
    if (!ready) return false;
    try {
      const matrixReady = await loadMatrixCryptoStatus();
      if (matrixReady) {
        cryptoProfileStatus.textContent = "E2E enabled with Matrix crypto.";
      }
      return matrixReady;
    } catch (err) {
      console.error("[E2E] Matrix crypto initialization failed", err);
      cryptoProfileStatus.textContent =
        "Classic E2E is available, but Matrix crypto could not initialize: " + err.message;
      return false;
    }
  });
  connect();
}

function setMode(next) {
  mode = next;
  legacyLogin = false;
  registerWithPassword = false;
  tabLogin.classList.toggle("active", next === "login");
  tabRegister.classList.toggle("active", next === "register");
  legacyLoginButton.textContent = "Use an older email/password account";
  accessKeyInput.hidden = next !== "login";
  accessKeyInput.required = next === "login";
  legacyLoginButton.hidden = next !== "login";
  registerPasswordToggle.hidden = next !== "register";
  legacyCredentials.hidden = true;
  emailInput.required = false;
  passwordInput.required = false;
  nameInput.hidden = next !== "register";
  nameInput.required = next === "register";
  authSubmit.textContent = next === "register" ? "Create account" : "Log in with key";
  registerPasswordToggle.textContent = "Create account with email and password";
}

function toggleLegacyLogin() {
  legacyLogin = !legacyLogin;
  legacyCredentials.hidden = !legacyLogin;
  emailInput.required = legacyLogin;
  passwordInput.required = legacyLogin;
  accessKeyInput.hidden = legacyLogin;
  accessKeyInput.required = !legacyLogin;
  legacyLoginButton.textContent = legacyLogin
    ? "Use access key"
    : "Use an older email/password account";
  authSubmit.textContent = legacyLogin ? "Log in with email/password" : "Log in with key";
}

function showAuthError(text, success = false) {
  authError.hidden = !text;
  authError.classList.toggle("notice", success);
  authError.classList.toggle("error", !success);
  authError.textContent = text;
}

function connect() {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  socket = new WebSocket(`${proto}://${location.host}/ws`);
  setStatus("connecting");
  reconnect = true;

  socket.addEventListener("open", () => setStatus("online"));

  socket.addEventListener("message", (event) => {
    const msg = JSON.parse(event.data);
    switch (msg.type) {
      case "welcome":
        me = msg.user;
        users = msg.users;
        renderMe();
        renderUsers();
        void loadGroups();
        const deepLink = new URLSearchParams(location.search);
        const deepLinkPeer = deepLink.get("peer");
        const deepLinkCall = deepLink.get("call");
        if (deepLinkPeer && users.some((user) => user.user_id === deepLinkPeer)) {
          history.replaceState(null, "", location.pathname);
          openChat(deepLinkPeer);
          if (["audio", "video"].includes(deepLinkCall)) {
            setTimeout(() => void startCall(deepLinkCall), 250);
          }
        } else if (peerId) {
          openChat(peerId);
        }
        break;
      case "directory":
        users = msg.users;
        renderUsers();
        break;
      case "groups":
        groups = msg.groups.map((group) => ({
          user_id: group.group_id,
          display_name: group.name,
          online: true,
          is_group: true,
          group_member_ids: group.member_ids,
        }));
        renderUsers();
        break;
      case "chat":
        emptyEl.hidden = true;
        chatTitlebar.hidden = false;
        peerName.hidden = false;
        composer.hidden = false;
        peerName.textContent = msg.peer.display_name;
        if (!msg.peer.is_group) void refreshPeerVerification(msg.peer.user_id);
        if (msg.peer.is_group && !groups.some((group) => group.user_id === msg.peer.user_id)) {
          groups.push({ ...msg.peer });
          renderUsers();
        } else if (msg.peer.is_group) {
          groups = groups.map((group) => group.user_id === msg.peer.user_id ? { ...group, ...msg.peer } : group);
        }
        document.getElementById("start-audio-call").hidden = Boolean(msg.peer.is_group);
        document.getElementById("start-video-call").hidden = Boolean(msg.peer.is_group);
        logEl.replaceChildren();
        messageBodyElementsById.clear();
        msg.history.forEach(appendMessage);
        break;
      case "message":
        if (isForOpenChat(msg.message)) appendMessage(msg.message);
        break;
      case "crypto_resync":
        void handleCryptoResyncRequest(msg);
        break;
      case "crypto_resync_response":
        void handleCryptoResyncResponse(msg);
        break;
      case "matrix_to_device":
        void matrixCryptoReady.then(async () => {
          await matrixCrypto?.handleLiveToDevice(msg);
          await retryVisibleMatrixMessages();
        }).catch((err) => {
          console.error("[E2E] Matrix to-device processing failed", err);
        });
        break;
      case "call_signal":
        handleCallSignal(msg).catch((err) => {
          appendSystem(`Call error: ${err.message}`);
          endCall(false);
        });
        break;
      case "error":
        appendSystem(msg.message);
        break;
      default:
        break;
    }
  });

  socket.addEventListener("close", () => {
    setStatus("offline");
    endCall(false);
    if (reconnect) setTimeout(connect, 1500);
  });
}

function openChat(id) {
  // Switching chats must not terminate an active call.
  // Calls live independently from the currently opened chat.
  peerId = id;
  const selected = [...users, ...groups].find((user) => user.user_id === id);
  peerVerified.hidden = true;
  if (!selected?.is_group) void refreshPeerVerification(id);
  document.getElementById("start-audio-call").hidden = Boolean(selected?.is_group);
  document.getElementById("start-video-call").hidden = Boolean(selected?.is_group);
  applyChatWallpaper(id);
  chatTitlebar.hidden = false;
  renderUsers();
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "open", peer_id: id }));
  }
}

async function startCall(kind) {
  if (!peerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  if (typeof globalThis.RTCPeerConnection !== "function") {
    const handoff = await openCallInSystemBrowser(peerId, kind);
    if (handoff.opened) {
      appendSystem("This desktop WebKit has no WebRTC support. The chat opened in your browser; sign in there if asked, then retry the call.");
      return;
    }
    appendSystem(`This desktop WebKit has no WebRTC support, and browser handoff failed: ${handoff.error}. Open this server in Firefox or Chromium to call.`);
    return;
  }
  if (peerConnection) endCall(true);
  callPeerId = peerId;
  callMediaKind = kind;
  try {
    localMediaStream = await acquireCallMedia(kind);
    callStage.hidden = false;
    await attachLocalMediaPreview();
    callStatus.textContent = `Calling…${callMediaNotice}`;
    peerConnection = await createPeerConnection();
    for (const track of localMediaStream.getTracks()) {
      peerConnection.addTrack(track, localMediaStream);
    }
    if (!localMediaStream.getAudioTracks().length && peerConnection.addTransceiver) {
      peerConnection.addTransceiver("audio", { direction: "recvonly" });
    }
    applyCallCodecPreferences(peerConnection);
    const offer = await peerConnection.createOffer();
    await peerConnection.setLocalDescription(offer);
    sendCallSignal("offer", {
      description: peerConnection.localDescription,
      media: kind,
    });
  } catch (err) {
    appendSystem(`Call setup failed: ${err.message || "Check camera and microphone permissions."}`);
    endCall(false);
  }
}

async function openCallInSystemBrowser(targetPeerId, kind) {
  const invoke = globalThis.__TAURI__?.core?.invoke;
  if (typeof invoke !== "function") return { opened: false, error: "desktop bridge unavailable" };
  const url = new URL(location.href);
  url.search = new URLSearchParams({ peer: targetPeerId, call: kind }).toString();
  try {
    await invoke("open_call_in_browser", { url: url.toString() });
    return { opened: true, error: "" };
  } catch (err) {
    return { opened: false, error: err?.message || String(err) };
  }
}

async function createPeerConnection() {
  if (typeof globalThis.RTCPeerConnection !== "function") {
    throw new Error("Calls are unavailable in this desktop runtime. Install WebRTC support for WebKitGTK, including the GStreamer webrtc plugin (gst-plugins-bad), then restart the app.");
  }
  const config = await api("GET", "/api/rtc-config");
  if (!Array.isArray(config.ice_servers) || config.ice_servers.length === 0) {
    throw new Error("The server returned no ICE servers. Configure STUN/TURN and restart the server.");
  }
  const connection = new globalThis.RTCPeerConnection({ iceServers: config.ice_servers });
  connection.addEventListener("icecandidate", (event) => {
    if (event.candidate) sendCallSignal("ice_candidate", event.candidate.toJSON());
  });
  connection.addEventListener("icecandidateerror", (event) => {
    if (connection !== peerConnection) return;
    const server = event.url || "configured ICE server";
    const code = event.errorCode ? ` (${event.errorCode})` : "";
    callStatus.textContent = `Could not reach ${server}${code}; checking available network routes.${callMediaNotice}`;
  });
  connection.addEventListener("track", (event) => {
    if (event.track.kind === "audio") {
      remoteAudio.srcObject = new MediaStream([event.track]);
      remoteAudio.play().then(() => {
        enableCallAudio.hidden = true;
      }).catch(() => {
        enableCallAudio.hidden = false;
        callStatus.textContent = "Connected. Use Enable sound to hear the call.";
      });
    } else if (event.streams[0]) {
      remoteVideo.srcObject = event.streams[0];
      remoteVideo.play().catch(() => {});
    }
  });
  connection.addEventListener("connectionstatechange", () => {
    if (connection !== peerConnection) return;
    const states = {
      connecting: "Connecting…",
      connected: "Connected",
      disconnected: "Connection interrupted",
      failed: "Connection failed. A STUN/TURN server may be required on this network.",
      closed: "Call ended",
    };
    const baseStatus = states[connection.connectionState] || connection.connectionState;
    callStatus.textContent = `${baseStatus}${callMediaNotice}`;
    if (connection.connectionState === "failed" || connection.connectionState === "closed") {
      endCall(false);
    }
  });
  connection.addEventListener("iceconnectionstatechange", () => {
    if (connection !== peerConnection) return;
    if (connection.iceConnectionState === "checking") {
      callStatus.textContent = `Checking network path${callMediaNotice}`;
    } else if (connection.iceConnectionState === "failed") {
      callStatus.textContent = `ICE failed. This network may require TURN.${callMediaNotice}`;
    } else if (connection.iceConnectionState === "disconnected") {
      callStatus.textContent = `ICE connection interrupted${callMediaNotice}`;
    }
  });
  return connection;
}

function applyCallCodecPreferences(connection) {
  if (typeof RTCRtpReceiver === "undefined" || !RTCRtpReceiver.getCapabilities) return;
  for (const transceiver of connection.getTransceivers()) {
    const kind = transceiver.receiver.track?.kind || transceiver.sender.track?.kind;
    if (!kind || !transceiver.setCodecPreferences) continue;
    const codecs = RTCRtpReceiver.getCapabilities(kind)?.codecs;
    if (!codecs) continue;
    const compatibleCodecs = codecs.filter((codec) => {
      return !(kind === "audio" && codec.mimeType.toLowerCase() === "audio/telephone-event");
    });
    if (compatibleCodecs.length) transceiver.setCodecPreferences(compatibleCodecs);
  }
}

async function handleCallSignal(signal) {
  if (!me || signal.sender_id === me.user_id) return;
  if (signal.kind === "offer" && !peerConnection) {
    pendingIncomingCall = signal;
    callPeerId = signal.sender_id;
    pendingIceCandidates = iceCandidatesBeforeOffer.get(signal.sender_id) || [];
    iceCandidatesBeforeOffer.delete(signal.sender_id);
    callMediaKind = signal.payload.media === "video" ? "video" : "audio";
    const caller = users.find((user) => user.user_id === signal.sender_id);
    incomingCallTitle.textContent = `Call from ${caller?.display_name || "Larptrix user"}`;
    const requestedKind = callMediaKind === "video" ? "Video call" : "Voice call";
    incomingCallKind.textContent = typeof globalThis.RTCPeerConnection === "function"
      ? requestedKind
      : `${requestedKind} · open the browser client and ask the caller to retry`;
    document.getElementById("accept-call").textContent = typeof globalThis.RTCPeerConnection === "function"
      ? "Accept"
      : "Open browser";
    incomingCallDialog.showModal();
    return;
  }
  if (signal.kind === "ice_candidate" && !peerConnection) {
    if (pendingIncomingCall && signal.sender_id === callPeerId) {
      pendingIceCandidates.push(signal.payload);
    }

    // Ignore ICE candidates that arrive without a pending offer.
    // They may belong to a previous/ended ICE generation.
    return;
  }

  if (signal.sender_id !== callPeerId) return;
  if (!peerConnection) return;
  if (signal.kind === "answer") {
    await peerConnection.setRemoteDescription(signal.payload);
    await flushIceCandidates();
  } else if (signal.kind === "ice_candidate") {
    const candidate = signal.payload;
    if (peerConnection.remoteDescription) await peerConnection.addIceCandidate(candidate);
    else pendingIceCandidates.push(candidate);
  } else if (signal.kind === "offer") {
    await peerConnection.setRemoteDescription(signal.payload.description);
    await flushIceCandidates();
    applyCallCodecPreferences(peerConnection);
    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);
    sendCallSignal("answer", peerConnection.localDescription);
  } else if (signal.kind === "reject" || signal.kind === "hangup") {
    callStatus.textContent = signal.kind === "reject" ? "Call declined" : "Call ended";
    endCall(false);
  }
}

async function acceptIncomingCall() {
  if (!pendingIncomingCall) return;
  if (typeof globalThis.RTCPeerConnection !== "function") {
    const callerId = pendingIncomingCall.sender_id;
    const handoff = await openCallInSystemBrowser(callerId, "");
    callStatus.textContent = "This desktop cannot answer calls. Open the same chat in your browser and ask the caller to try again.";
    appendSystem(handoff.opened
      ? "The same chat opened in your browser. Sign in there if asked, then ask the caller to retry."
      : `Could not open the browser (${handoff.error}). Open this server in Firefox or Chromium and ask the caller to retry.`);
    return;
  }
  const incoming = pendingIncomingCall;
  incomingCallDialog.close();
  peerId = incoming.sender_id;
  renderUsers();
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "open", peer_id: peerId }));
  }
  try {
    localMediaStream = await acquireCallMedia(callMediaKind);
    callStage.hidden = false;
    await attachLocalMediaPreview();
    callStatus.textContent = `Connecting…${callMediaNotice}`;
    peerConnection = await createPeerConnection();
    for (const track of localMediaStream.getTracks()) peerConnection.addTrack(track, localMediaStream);
    applyCallCodecPreferences(peerConnection);
    await peerConnection.setRemoteDescription(incoming.payload.description);
    await flushIceCandidates();
    const answer = await peerConnection.createAnswer();
    await peerConnection.setLocalDescription(answer);
    pendingIncomingCall = null;
    sendCallSignal("answer", peerConnection.localDescription);
  } catch (err) {
    appendSystem(err.message || "Could not accept the call. Check camera and microphone permissions.");
    sendCallSignal("reject", {});
    endCall(false);
  }
}

function rejectIncomingCall() {
  if (pendingIncomingCall) {
    const rejectedPeerId = pendingIncomingCall.sender_id;
    callPeerId = pendingIncomingCall.sender_id;
    sendCallSignal("reject", {});
    iceCandidatesBeforeOffer.delete(rejectedPeerId);
  }
  pendingIncomingCall = null;
  incomingCallDialog.close();
  callPeerId = null;
}

async function flushIceCandidates() {
  const candidates = pendingIceCandidates;
  pendingIceCandidates = [];
  for (const candidate of candidates) await peerConnection.addIceCandidate(candidate);
}

async function acquireCallMedia(kind) {
  callMediaNotice = "";
  if (!navigator.mediaDevices?.getUserMedia) {
    throw new Error("This desktop runtime does not provide camera or microphone capture.");
  }
  let hasMicrophone = true;
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    hasMicrophone = devices.some((device) => device.kind === "audioinput");
  } catch {
    // Device enumeration can be blocked until capture permission is granted.
  }
  const videoAttempts = kind === "video"
    ? [{ width: { ideal: 1280 }, height: { ideal: 720 } }, true]
    : [false];
  const attempts = [];
  if (hasMicrophone) {
    for (const video of videoAttempts) attempts.push({ audio: true, video });
    if (kind === "video") attempts.push({ audio: true, video: false });
  }
  if (kind === "video") {
    for (const video of videoAttempts) attempts.push({ audio: false, video });
  }
  if (!attempts.length) {
    callMediaNotice = " · listen-only (no microphone detected)";
    return new MediaStream();
  }
  let lastError;
  for (const constraints of attempts) {
    try {
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      if (kind === "video" && !stream.getVideoTracks().length) {
        callMediaNotice = " · camera unavailable, audio-only";
      } else if (!stream.getAudioTracks().length) {
        callMediaNotice = kind === "video"
          ? " · video only (no microphone)"
          : " · listen-only (no microphone)";
      }
      return stream;
    } catch (err) {
      lastError = err;
    }
  }
  if (kind === "audio") {
    callMediaNotice = " · listen-only, microphone unavailable";
    return new MediaStream();
  }
  throw new Error(lastError?.message || "Could not access a camera or microphone.");
}

function sendCallSignal(kind, payload) {
  if (!callPeerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  socket.send(JSON.stringify({ type: "call_signal", peer_id: callPeerId, kind, payload }));
}

function toggleMicrophone() {
  const track = localMediaStream?.getAudioTracks()[0];
  if (!track) return;
  track.enabled = !track.enabled;
  const button = document.getElementById("toggle-microphone");
  button.textContent = track.enabled ? "🎙 Mute mic" : "🔇 Unmute mic";
  button.setAttribute("aria-pressed", String(!track.enabled));
}

function toggleCamera() {
  const track = localMediaStream?.getVideoTracks()[0];
  if (!track) return;
  track.enabled = !track.enabled;
  document.getElementById("toggle-camera").textContent = track.enabled ? "📷 Turn camera off" : "🚫 Turn camera on";
}

async function toggleScreenShare() {
  if (!peerConnection) return;
  if (screenMediaStream) {
    await stopScreenShare();
    return;
  }
  try {
    const highQuality = screenQuality.value === "high";
    const webkitGtk = navigator.platform.toLowerCase().includes("linux")
      && navigator.userAgent.includes("AppleWebKit")
      && !/(Chrome|Chromium)/.test(navigator.userAgent);
    if (!navigator.mediaDevices?.getDisplayMedia) {
      throw new Error("Screen capture is not supported by this desktop runtime.");
    }
    screenMediaStream = await navigator.mediaDevices.getDisplayMedia({
      video: true,
      audio: !webkitGtk,
    });
    const screenTrack = screenMediaStream.getVideoTracks()[0];
    localScreenVideo.srcObject = screenMediaStream;
    localScreenVideo.hidden = false;
    localScreenVideo.play().catch(() => {});
    let sender = peerConnection.getSenders().find((item) => item.track === localMediaStream?.getVideoTracks()[0]);
    let renegotiate = false;
    if (sender) {
      await sender.replaceTrack(screenTrack);
    } else {
      sender = peerConnection.addTrack(screenTrack, screenMediaStream);
      renegotiate = true;
    }
    const screenAudioTrack = screenMediaStream.getAudioTracks()[0];
    if (screenAudioTrack) {
      peerConnection.addTrack(screenAudioTrack, screenMediaStream);
      renegotiate = true;
    }
    if (renegotiate) {
      applyCallCodecPreferences(peerConnection);
      const offer = await peerConnection.createOffer();
      await peerConnection.setLocalDescription(offer);
      sendCallSignal("offer", { description: peerConnection.localDescription, media: callMediaKind });
    }
    try {
      const params = sender.getParameters();
      params.encodings = params.encodings?.length ? params.encodings : [{}];
      params.encodings[0].maxBitrate = highQuality ? 12_000_000 : 3_000_000;
      params.encodings[0].maxFramerate = highQuality ? 144 : 30;
      await sender.setParameters(params);
    } catch {
      // The browser may not expose encoder controls for this track.
    }
    const settings = screenTrack.getSettings();
    const audioStatus = screenAudioTrack
      ? " with shared audio"
      : webkitGtk
        ? " (screen audio unavailable in WebKitGTK)"
        : " (source audio unavailable)";
    callStatus.textContent = `Sharing ${settings.width || "?"}×${settings.height || "?"} at ${Math.round(settings.frameRate || 0)} fps${audioStatus}`;
    screenTrack.addEventListener("ended", stopScreenShare, { once: true });
    document.getElementById("toggle-screen-share").textContent = "Stop sharing";
  } catch (err) {
    if (err.name !== "NotAllowedError") appendSystem(err.message || "Could not start screen sharing.");
    screenMediaStream = null;
  }
}

async function stopScreenShare() {
  if (!screenMediaStream) return;
  const screenTrack = screenMediaStream.getVideoTracks()[0];
  const sender = peerConnection?.getSenders().find((item) => item.track === screenTrack);
  const cameraTrack = localMediaStream?.getVideoTracks()[0];
  if (sender) await sender.replaceTrack(cameraTrack || null);
  for (const audioTrack of screenMediaStream.getAudioTracks()) {
    const audioSender = peerConnection?.getSenders().find((item) => item.track === audioTrack);
    if (audioSender) await audioSender.replaceTrack(null);
  }
  screenMediaStream.getTracks().forEach((track) => track.stop());
  screenMediaStream = null;
  localScreenVideo.srcObject = null;
  localScreenVideo.hidden = true;
  document.getElementById("toggle-screen-share").textContent = "Share screen";
  if (callStatus.textContent.startsWith("Sharing ")) callStatus.textContent = "Connected";
}

function endCall(notifyPeer) {
  if (notifyPeer && callPeerId) sendCallSignal("hangup", {});
  const endedPeerId = callPeerId;
  if (incomingCallDialog.open) incomingCallDialog.close();
  screenMediaStream?.getTracks().forEach((track) => track.stop());
  localMediaStream?.getTracks().forEach((track) => track.stop());
  peerConnection?.close();
  peerConnection = null;
  localMediaStream = null;
  screenMediaStream = null;
  pendingIncomingCall = null;
  pendingIceCandidates = [];
  if (endedPeerId) iceCandidatesBeforeOffer.delete(endedPeerId);
  callPeerId = null;
  callMediaKind = null;
  localVideo.srcObject = null;
  localScreenVideo.srcObject = null;
  localScreenVideo.hidden = true;
  remoteVideo.srcObject = null;
  remoteAudio.srcObject = null;
  enableCallAudio.hidden = true;
  callStage.hidden = true;
  callStage.classList.remove("call-collapsed");
  const collapseButton = document.getElementById("call-collapse");
  collapseButton.textContent = "−";
  collapseButton.title = "Minimize call";
  document.getElementById("toggle-microphone").textContent = "🎙 Mute mic";
  document.getElementById("toggle-microphone").setAttribute("aria-pressed", "false");
  document.getElementById("toggle-camera").textContent = "📷 Turn camera off";
  document.getElementById("toggle-screen-share").textContent = "Share screen";
}

function isForOpenChat(message) {
  if (!peerId || !me) return false;
  const group = groups.find((item) => item.user_id === peerId);
  if (group?.is_group) return message.recipient_id === peerId;
  return (
    (message.sender_id === me.user_id && message.recipient_id === peerId) ||
    (message.sender_id === peerId && message.recipient_id === me.user_id)
  );
}

function setStatus(text) {
  statusEl.textContent = text;
  statusEl.classList.toggle("online", text === "online");
}

function renderMe() {
  if (!me) return;
  meLabel.textContent = me.display_name;
  meUsername.textContent = me.username ? `@${me.username}` : "";
  renderMenuAccount();
  profileName.value = me.display_name;
  profileEmail.value = me.email || "";
  profileEmail.hidden = !me.email;
  profileEmailLabel.hidden = !me.email;
  paintAvatar(meAvatar, me);
  updateOwnProfileCard(me);
}

function renderUsers() {
  usersEl.replaceChildren();
  const query = userSearchInput.value.trim().replace(/^@/, "").toLocaleLowerCase();
  const matches = [...users, ...groups].filter((user) => {
    if (!query) return true;
    return user.display_name.toLocaleLowerCase().includes(query)
      || (user.username || "").toLocaleLowerCase().includes(query);
  });
  for (const user of matches) {
    if (me && user.user_id === me.user_id) continue;
    const li = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.classList.toggle("active", user.user_id === peerId);
    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user);
    const dot = document.createElement("span");
    dot.className = user.online ? "dot on" : "dot";
    const name = document.createElement("span");
    name.className = "person-name";
    const displayName = document.createElement("span");
    displayName.textContent = user.is_group ? `👥 ${user.display_name}` : user.display_name;
    name.append(displayName);
    if (user.activity && !user.is_group) {
      const activity = document.createElement("small");
      activity.className = "person-activity";
      activity.textContent = user.activity;
      name.append(activity);
    }
    if (user.username && !user.is_group) {
      const handle = document.createElement("small");
      handle.textContent = `@${user.username}`;
      name.append(handle);
    }
    button.append(avatar, name, dot);
    button.addEventListener("click", () => openChat(user.user_id));
    li.append(button);
    usersEl.append(li);
  }
}

async function loadGroups() {
  try {
    const savedGroups = await api("GET", "/api/groups");
    groups = savedGroups.map((group) => ({
      user_id: group.group_id,
      display_name: group.name,
      online: true,
      is_group: true,
      group_member_ids: group.member_ids,
    }));
    renderUsers();
  } catch (err) {
    appendSystem(`Could not load groups: ${err.message}`);
  }
}

function renderGroupMemberChoices() {
  groupMemberList.replaceChildren();
  for (const user of users.filter((item) => item.user_id !== me?.user_id && !item.is_group)) {
    const label = document.createElement("label");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.value = user.user_id;
    checkbox.disabled = !user.e2e_enabled;
    const name = document.createElement("span");
    name.textContent = user.e2e_enabled
      ? `${user.display_name} · E2E ready`
      : `${user.display_name} · sign in and set up E2E first`;
    if (!user.e2e_enabled) label.classList.add("member-needs-e2e");
    label.append(checkbox, name);
    groupMemberList.append(label);
  }
}

async function createGroup(event) {
  event.preventDefault();
  const error = document.getElementById("group-create-error");
  error.hidden = true;
  const memberIds = [...groupMemberList.querySelectorAll("input:checked")].map((input) => input.value);
  try {
    const created = await api("POST", "/api/groups", {
      name: document.getElementById("group-name").value.trim(),
      member_ids: memberIds,
    });
    const newGroup = {
      user_id: created.group_id,
      display_name: created.name,
      online: true,
      is_group: true,
      group_member_ids: created.member_ids,
    };
    groups = [...groups.filter((group) => group.user_id !== newGroup.user_id), newGroup];
    createGroupDialog.close();
    renderUsers();
    openChat(created.group_id);
  } catch (err) {
    error.textContent = err.message;
    error.hidden = false;
  }
}

function paintAvatar(el, user) {
  el.replaceChildren();
  el.classList.toggle("emoji-avatar", !user.avatar_url);
  if (user.avatar_url) {
    const img = document.createElement("img");
    const cacheKey = user.avatar_id || user.updated_at || user.avatar_version || Date.now();
    img.src = `${user.avatar_url}${user.avatar_url.includes("?") ? "&" : "?"}v=${encodeURIComponent(cacheKey)}`;
    img.alt = "";
    el.append(img);
  } else {
    const faces = ["🐸", "🦊", "🐙", "🐟", "🦉", "🐧", "🐢", "🦋"];
    const seed = [...(user.user_id || user.display_name || "")]
      .reduce((value, character) => value + character.charCodeAt(0), 0);
    el.classList.add("emoji-avatar");
    el.textContent = faces[seed % faces.length];
  }
}

function appendMessage(message) {
  messagesById.set(message.id, message);
  const li = document.createElement("li");
  if (me && message.sender_id === me.user_id) li.classList.add("me");

  const meta = document.createElement("div");
  meta.className = "meta";
  meta.textContent = `${message.sender_name} · ${new Date(message.created_at).toLocaleTimeString()}`;
  li.append(meta);

  let encryptedBodyElement = null;
  if (message.body) {
    const body = document.createElement("div");
    const envelope = parseCryptoEnvelope(message.body);
    if (envelope) {
      body.textContent = "Encrypted message";
      encryptedBodyElement = body;

    } else {
      body.textContent = cryptoEnabled
        ? `⚠️ Legacy message (not end-to-end encrypted): ${message.body}`
        : message.body;
    }
    li.append(body);
  }
  if (message.attachment && !parseCryptoEnvelope(message.body)) {
    if (message.attachment.mime.startsWith("image/")) {
      const img = document.createElement("img");
      img.className = "photo";
      img.src = message.attachment.url;
      img.alt = message.attachment.name;
      img.tabIndex = 0;
      img.setAttribute("role", "button");
      img.title = "Open image";
      img.addEventListener("click", () => openImageViewer(img.src, img.alt));
      img.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          openImageViewer(img.src, img.alt);
        }
      });
      li.append(img);
    } else if (message.attachment.mime.startsWith("audio/")) {
      const audio = document.createElement("audio");
      audio.controls = true;
      audio.preload = "metadata";
      audio.src = message.attachment.url;
      li.append(audio);
    } else {
      const link = document.createElement("a");
      link.href = message.attachment.url;
      link.download = message.attachment.name;
      link.textContent = `${message.attachment.name} (${formatSize(message.attachment.size_bytes)})`;
      li.append(link);
    }
  }

  logEl.append(li);
  if (encryptedBodyElement) {
    messageBodyElementsById.set(message.id, encryptedBodyElement);

    let effectiveMessage = message;
    const queuedResponse = cryptoRecoveryResponsesByMessageId.get(message.id);
    if (queuedResponse) {
      effectiveMessage =
        mergeCryptoRecoveryResponse(message, queuedResponse)
        || (
          parseCryptoEnvelope(message.body)?.version === 1
            ? { ...message, body: queuedResponse.ciphertext }
            : message
        );
      if (effectiveMessage !== message) {
        cryptoRecoveryResponsesByMessageId.delete(message.id);
      }
    }

    const recoveredBody = recoveredBodiesByMessageId.get(message.id);
    if (recoveredBody) {
      effectiveMessage = { ...effectiveMessage, body: recoveredBody };
      recoveredBodiesByMessageId.delete(message.id);
    }

    void displayEncryptedMessage(effectiveMessage, encryptedBodyElement).then((ok) => {
      if (ok && queuedResponse) {
        socket?.send(JSON.stringify({
          type: "crypto_resync_response_ack",
          peer_id: queuedResponse.sender_id,
          message_id: message.id,
          device_id: queuedResponse.device_id,
        }));
      }
    });
  }
  logEl.scrollTop = logEl.scrollHeight;
}

async function attachLocalMediaPreview() {
  localVideo.srcObject = localMediaStream;
  localVideo.hidden = !localMediaStream?.getVideoTracks().length;
  document.getElementById("toggle-microphone").disabled = !localMediaStream?.getAudioTracks().length;
  document.getElementById("toggle-camera").disabled = !localMediaStream?.getVideoTracks().length;
  if (!localVideo.hidden) {
    try {
      await localVideo.play();
    } catch {
      callStatus.textContent = "Camera is on. Click the preview to start local playback.";
    }
  }
}

function mergeCryptoRecoveryResponse(message, response) {
  const envelope = parseCryptoEnvelope(message?.body);
  if (
    !envelope
    || envelope.version !== 2
    || envelope.message_type !== "message"
    || !response?.device_id
    || typeof response.ciphertext !== "string"
  ) {
    return null;
  }

  return {
    ...message,
    body: JSON.stringify({
      ...envelope,
      ciphertexts: {
        ...envelope.ciphertexts,
        [response.device_id]: response.ciphertext,
      },
    }),
  };
}

function parseCryptoEnvelope(raw) {
  try {
    const envelope = JSON.parse(raw);

    if (
      envelope?.version === 2
      && envelope.message_type === "message"
      && typeof envelope.sender_device_id === "string"
      && envelope.sender_device_id
      && envelope.ciphertexts
      && typeof envelope.ciphertexts === "object"
      && !Array.isArray(envelope.ciphertexts)
    ) {
      return envelope;
    }

    if (
      envelope?.version === 1
      && envelope.message_type === "group"
      && envelope.ciphertexts
      && typeof envelope.ciphertexts === "object"
      && !Array.isArray(envelope.ciphertexts)
    ) {
      return envelope;
    }

    if (
      envelope?.version === 1
      && ["message", "prekey"].includes(envelope.message_type)
      && typeof envelope.ciphertext === "string"
    ) {
      return envelope;
    }

    if (
      envelope?.version === 3
      && envelope.message_type === "matrix"
      && typeof envelope.sender_device_id === "string"
      && envelope.sender_device_id
      && typeof envelope.room_id === "string"
      && envelope.room_id
      && envelope.ciphertext
      && typeof envelope.ciphertext === "object"
      && !Array.isArray(envelope.ciphertext)
    ) {
      return envelope;
    }

    return null;
  } catch {
    return null;
  }
}

console.log("[E2E] displayEncryptedMessage loaded");

async function retryVisibleMatrixMessages() {
  const entries = [...messageBodyElementsById.entries()];
  for (const [messageId, bodyElement] of entries) {
    const message = messagesById.get(messageId);
    if (!message || !parseCryptoEnvelope(message.body)) continue;
    if (!(bodyElement.textContent || "").startsWith("Could not decrypt Matrix message:")) continue;
    await displayEncryptedMessage(message, bodyElement, { allowRecovery: false });
  }
}

async function displayEncryptedMessage(message, bodyElement, { allowRecovery = true } = {}) {
  const matrixEnvelope = parseCryptoEnvelope(message.body);
  if (matrixEnvelope?.version === 3 && matrixEnvelope.message_type === "matrix") {
    try {
      await matrixCryptoReady;
      if (!matrixCrypto) throw new Error("Matrix E2E is not initialized.");
      const decrypted = await matrixCrypto.decrypt(matrixEnvelope.room_id, {
        id: message.id,
        sender_id: message.sender_id,
        created_at: message.created_at,
        body: matrixEnvelope,
      });
      const payload = decrypted?.content ?? decrypted;
      bodyElement.textContent =
        typeof payload?.text === "string" ? payload.text : JSON.stringify(payload);

      if (payload?.file && message.attachment) {
        try {
          await renderEncryptedAttachment(message.attachment, payload.file, bodyElement.parentElement);
        } catch (err) {
          bodyElement.textContent =
            `Could not open encrypted attachment: ${err?.message || String(err)}`;
        }
      }
      return true;
    } catch (err) {
      bodyElement.textContent =
        `Could not decrypt Matrix message: ${err?.message || String(err)}`;
      return false;
    }
  }

  if (message.sender_id === me?.user_id) {
    await cryptoReady;
    const cached = sentPlaintextByCiphertext.get(message.body)
      || await loadCachedSentPlaintext(message.body);
    if (cached) sentPlaintextByCiphertext.set(message.body, cached);
    const payload = parseEncryptedPayload(cached);
    bodyElement.textContent = payload?.text || "Encrypted message sent from this device";
    if (payload?.file && message.attachment) {
      try {
        await renderEncryptedAttachment(message.attachment, payload.file, bodyElement.parentElement);
      } catch (err) {
        bodyElement.textContent = `Could not open encrypted attachment: ${err?.message || String(err)}`;
      }
    }
    return true;
  }

  if (!cryptoEnabled) {
    bodyElement.textContent = "Encrypted. Set up E2E to read messages.";
    return false;
  }

  try {
    await cryptoReady;
    if (!cryptoDevice) {
      throw new Error("Unlock E2E with your recovery key to read messages.");
    }

    // The whole incoming crypto operation is serialized.
    // This is important for history because fetching crypto-key before
    // entering the queue can cause messages to reach the Olm ratchet
    // in a different order than the history itself.
    const result = await withCryptoStateLock(async () => {
      const envelope = parseCryptoEnvelope(message.body);

      if (!envelope) {
        throw new Error("Invalid encrypted message envelope.");
      }

      const sender = users.find((item) => item.user_id === message.sender_id)
        || { user_id: message.sender_id, display_name: message.sender_name };

      let encryptedBody;
      let senderDeviceId;
      let senderBundle;

      if (envelope.version === 2 && envelope.message_type === "message") {
        senderDeviceId = envelope.sender_device_id;
        encryptedBody = envelope.ciphertexts[cryptoDevice.device_id()];

        if (typeof encryptedBody !== "string") {
          throw new Error("No encrypted copy was addressed to this device.");
        }

        const senderDevices = await api(
          "GET",
          `/api/users/${encodeURIComponent(message.sender_id)}/crypto-devices`
        );

        const devices = Array.isArray(senderDevices?.devices)
          ? senderDevices.devices
          : [];

        senderBundle = devices.find(
          (bundle) => bundle?.device_id === senderDeviceId
        );

        if (!senderBundle) {
          throw new Error("Sender device was not found.");
        }

        if (!(await ensurePeerFingerprint(sender, senderBundle))) {
          throw new Error("Encrypted. Sender device was not verified; message was not decrypted.");
        }
      } else if (envelope.version === 1 && envelope.message_type === "group") {
        const senderBundleResponse = await api(
          "GET",
          `/api/users/${encodeURIComponent(message.sender_id)}/crypto-key`
        );
        senderBundle = Array.isArray(senderBundleResponse?.devices)
          ? senderBundleResponse.devices[0]
          : senderBundleResponse;

        if (!(await ensurePeerFingerprint(sender, senderBundle))) {
          throw new Error("Encrypted. Device was not verified; message was not decrypted.");
        }

        senderDeviceId = message.sender_id;
        encryptedBody = envelope.ciphertexts[me.user_id];

        if (typeof encryptedBody !== "string") {
          throw new Error("No encrypted copy was addressed to this account.");
        }
      } else {
        const senderBundleResponse = await api(
          "GET",
          `/api/users/${encodeURIComponent(message.sender_id)}/crypto-key`
        );
        const senderDevices = Array.isArray(senderBundleResponse?.devices)
          ? senderBundleResponse.devices
          : [senderBundleResponse];

        if (!senderDevices.length || !senderDevices[0]) {
          throw new Error("Sender E2E device was not found.");
        }

        // Historical v1 sessions were keyed by the sender account id, not
        // the sender device id. Keep that identity so old messages remain
        // decryptable after the Matrix migration.
        senderBundle = senderDevices[0];
        if (!(await ensurePeerFingerprint(sender, senderBundle))) {
          throw new Error("Encrypted. Device was not verified; message was not decrypted.");
        }

        senderDeviceId = message.sender_id;
        encryptedBody = message.body;
      }

      console.log("[E2E] incoming", {
        sender: message.sender_id,
        senderDevice: senderDeviceId,
        type: envelope.message_type,
        hasSession: cryptoDevice.has_session(senderDeviceId),
        sessionCount: cryptoDevice.session_count(senderDeviceId),
      });

      const sessionCountBefore = cryptoDevice.session_count(senderDeviceId);

      console.log("[E2E] decrypt start", {
        sender: message.sender_id,
        senderDevice: senderDeviceId,
        type: envelope.message_type,
        sessionCountBefore,
      });

      let plaintext;

      try {
        plaintext = cryptoDevice.decrypt(
          senderDeviceId,
          encryptedBody,
          JSON.stringify(senderBundle),
          senderBundle.fingerprint,
        );

        cryptoRecoveryPending.delete(message.id);
        console.log("[E2E] decrypt success", {
          sender: message.sender_id,
          senderDevice: senderDeviceId,
          type: envelope.message_type,
          sessionCountAfter: cryptoDevice.session_count(senderDeviceId),
        });
      } catch (err) {
        console.error("[E2E] decrypt FAILED", {
          sender: message.sender_id,
          senderDevice: senderDeviceId,
          type: envelope.message_type,
          sessionCountAfter: cryptoDevice.session_count(senderDeviceId),
          error: err?.message || String(err),
        });

        const recoveryCheck = {
          envelopeVersion: envelope.version,
          messageType: envelope.message_type,
          socketState: socket?.readyState ?? null,
          socketOpen: socket?.readyState === WebSocket.OPEN,
        };
        console.log("[E2E] recovery check", recoveryCheck);

        if (
          allowRecovery
          && (
            (
              envelope.version === 1
              && ["message", "prekey"].includes(envelope.message_type)
            )
            || (
              envelope.version === 2
              && envelope.message_type === "message"
            )
          )
          && socket?.readyState === WebSocket.OPEN
        ) {
          const recoveryDevice = cryptoDevice.device_id();
          const lastRecovery = cryptoRecoveryLastAttempt.get(message.id) || 0;
          const recoveryPending = cryptoRecoveryPending.has(message.id);
          const recoveryCooldownMs = 30000;
          const recoveryAllowed = Date.now() - lastRecovery >= recoveryCooldownMs;

          if (recoveryPending) {
            console.log("[E2E] automatic session recovery already pending", {
              message: message.id,
              sender: message.sender_id,
            });
          } else if (recoveryAllowed) {
            cryptoRecoveryLastAttempt.set(message.id, Date.now());
            cryptoRecoveryPending.set(message.id, {
              senderId: message.sender_id,
              senderDeviceId,
              deviceId: recoveryDevice,
              message: { ...message },
            });
            socket.send(JSON.stringify({
              type: "crypto_resync",
              peer_id: message.sender_id,
              message_id: message.id,
              body: message.body,
              device_id: recoveryDevice,
              attachment_id: message.attachment?.id || null,
            }));
            console.log("[E2E] requested automatic session recovery", {
              message: message.id,
              sender: message.sender_id,
              senderDevice: senderDeviceId,
              device: recoveryDevice,
            });
          } else {
            console.log("[E2E] automatic session recovery throttled", {
              message: message.id,
              sender: message.sender_id,
              senderDevice: senderDeviceId,
              cooldownMs: recoveryCooldownMs,
              remainingMs: recoveryCooldownMs - (Date.now() - lastRecovery),
            });
          }
        }
        throw err;
      }

      await persistCryptoState();

      return plaintext;
    });

    const payload = parseEncryptedPayload(result);
    bodyElement.textContent = payload?.text ?? result;

    if (payload?.file && message.attachment) {
      try {
        await renderEncryptedAttachment(
          message.attachment,
          payload.file,
          bodyElement.parentElement
        );
      } catch (err) {
        bodyElement.textContent =
          `Could not open encrypted attachment: ${err?.message || String(err)}`;
      }
    }
  } catch (err) {
    bodyElement.textContent = `Could not decrypt message: ${err?.message || String(err)}`;
    return false;
  }
  return true;
}

async function handleCryptoResyncResponse(response) {
  console.log("[E2E] recovery response received", {
    sender: response?.sender_id,
    message: response?.message_id,
    device: response?.device_id,
    hasCiphertext: typeof response?.ciphertext === "string",
    ciphertextLength: typeof response?.ciphertext === "string" ? response.ciphertext.length : 0,
  });

  if (
    !response?.sender_id
    || !response?.message_id
    || !response?.device_id
    || typeof response?.ciphertext !== "string"
  ) {
    console.warn("[E2E] recovery response ignored: malformed");
    return;
  }

  const pending = cryptoRecoveryPending.get(response.message_id);
  if (!pending) {
    const knownMessage = messagesById.get(response.message_id);
    const currentEnvelope = knownMessage ? parseCryptoEnvelope(knownMessage.body) : null;

    if (
      (
        (
          currentEnvelope?.version === 1
          && ["message", "prekey"].includes(currentEnvelope.message_type)
        )
        || (
          currentEnvelope?.version === 2
          && currentEnvelope.message_type === "message"
        )
      )
      && typeof response.device_id === "string"
    ) {
      cryptoRecoveryResponsesByMessageId.set(response.message_id, {
        sender_id: response.sender_id,
        device_id: response.device_id,
        ciphertext: response.ciphertext,
      });
      const bodyElement = messageBodyElementsById.get(response.message_id);
      if (bodyElement) {
        const recoveredMessage =
          currentEnvelope?.version === 1
            ? { ...knownMessage, body: response.ciphertext }
            : mergeCryptoRecoveryResponse(knownMessage, response);
        if (!recoveredMessage) return;
        const ok = await displayEncryptedMessage(
          recoveredMessage,
          bodyElement,
          { allowRecovery: false },
        );
        if (ok) {
          cryptoRecoveryResponsesByMessageId.delete(response.message_id);
          socket?.send(JSON.stringify({
            type: "crypto_resync_response_ack",
            peer_id: response.sender_id,
            message_id: response.message_id,
            device_id: response.device_id,
          }));
        }
      }
      return;
    }

    console.warn("[E2E] recovery response stored: no matching history message", {
      message: response.message_id,
    });
    cryptoRecoveryResponsesByMessageId.set(response.message_id, {
      sender_id: response.sender_id,
      device_id: response.device_id,
      ciphertext: response.ciphertext,
    });
    return;
  }

  if (
    pending.senderId !== response.sender_id
    || pending.deviceId !== response.device_id
  ) {
    console.warn("[E2E] recovery response ignored: request binding mismatch", {
      message: response.message_id,
      expectedSender: pending.senderId,
      actualSender: response.sender_id,
      expectedDevice: pending.deviceId,
      actualDevice: response.device_id,
    });
    return;
  }

  cryptoRecoveryPending.delete(response.message_id);

  let originalEnvelope = parseCryptoEnvelope(pending.message.body);
  const validV2Original =
    originalEnvelope?.version === 2
    && originalEnvelope.message_type === "message"
    && originalEnvelope.sender_device_id === pending.senderDeviceId;
  const validV1Original =
    originalEnvelope?.version === 1
    && ["message", "prekey"].includes(originalEnvelope.message_type)
    && pending.senderDeviceId === pending.senderId;

  if (!validV2Original && !validV1Original) {
    console.warn("[E2E] recovery response ignored: original message binding is invalid", {
      message: response.message_id,
    });
    return;
  }

  try {
    const recoveredCipherEnvelope = JSON.parse(response.ciphertext);
    if (
      recoveredCipherEnvelope?.version !== 1
      || !["message", "prekey"].includes(recoveredCipherEnvelope?.message_type)
      || typeof recoveredCipherEnvelope?.ciphertext !== "string"
    ) {
      throw new Error("Recovery response was not a valid v1 ciphertext.");
    }
  } catch (err) {
    console.warn("[E2E] recovery response ignored: invalid v1 ciphertext", {
      message: response.message_id,
      error: err?.message || String(err),
    });
    return;
  }

  let recoveredMessage;

  if (
    originalEnvelope.version === 1
    && ["message", "prekey"].includes(originalEnvelope.message_type)
  ) {
    recoveredMessage = {
      ...pending.message,
      body: response.ciphertext,
    };
  } else {
    originalEnvelope = {
      ...originalEnvelope,
      ciphertexts: {
        ...originalEnvelope.ciphertexts,
        [response.device_id]: response.ciphertext,
      },
    };

    recoveredMessage = {
      ...pending.message,
      body: JSON.stringify(originalEnvelope),
    };
  }
  const bodyElement = messageBodyElementsById.get(response.message_id);

  if (!bodyElement) {
    cryptoRecoveryResponsesByMessageId.set(response.message_id, {
      sender_id: response.sender_id,
      device_id: response.device_id,
      ciphertext: recoveredMessage.body,
      legacyBody: originalEnvelope.version === 1,
    });
    console.log("[E2E] recovery response stored until message is rendered", {
      message: response.message_id,
    });
    return;
  }

  try {
    const ok = await displayEncryptedMessage(
      recoveredMessage,
      bodyElement,
      { allowRecovery: false },
    );
    if (ok) {
      socket?.send(JSON.stringify({
        type: "crypto_resync_response_ack",
        peer_id: response.sender_id,
        message_id: response.message_id,
        device_id: response.device_id,
      }));
    }
    console.log("[E2E] recovery response decrypted", {
      message: response.message_id,
      device: response.device_id,
      ok,
    });
  } catch (err) {
    console.error("[E2E] recovery response decrypt failed", {
      message: response.message_id,
      error: err?.message || String(err),
    });
  }
}

async function handleCryptoResyncRequest(request) {
  console.log("[E2E] recovery request received", {
    requester: request?.requester_id,
    message: request?.message_id,
    device: request?.device_id,
    hasBody: typeof request?.body === "string",
    bodyLength: typeof request?.body === "string" ? request.body.length : 0,
  });

  if (!me || !request?.requester_id || request.requester_id === me.user_id) {
    console.warn("[E2E] recovery request ignored", {
      hasUser: Boolean(me),
      requester: request?.requester_id,
      ownUser: me?.user_id,
    });
    return false;
  }

  try {
    // Queued recovery requests can arrive immediately after reconnect,
    // before the local E2E device has finished unlocking. Wait instead of
    // dropping the request.
    await cryptoReady;
    if (!cryptoEnabled || !cryptoDevice) {
      console.warn("[E2E] recovery request ignored: E2E device is unavailable", {
        cryptoEnabled,
        hasCryptoDevice: Boolean(cryptoDevice),
      });
      return false;
    }

    await withCryptoStateLock(async () => {
      console.log("[E2E] recovery: looking up cached plaintext");

      const cached = sentPlaintextByCiphertext.get(request.body)
        || await loadCachedSentPlaintext(request.body);

      if (!cached) {
        console.warn("[E2E] recovery requested, but original plaintext is not cached locally");
        return;
      }

      console.log("[E2E] recovery: plaintext cache found");

      const original = parseCryptoEnvelope(request.body);
      if (
        !original
        || (
          original.version === 1
            ? !["message", "prekey"].includes(original.message_type)
            : !(original.version === 2 && original.message_type === "message")
        )
      ) {
        console.warn("[E2E] recovery: original envelope is invalid or unsupported");
        return;
      }

      const devicesResponse = await api(
        "GET",
        `/api/users/${encodeURIComponent(request.requester_id)}/crypto-devices`
      );
      const devices = Array.isArray(devicesResponse?.devices) ? devicesResponse.devices : [];
      const target = devices.find((device) => device?.device_id === request.device_id);

      console.log("[E2E] recovery: target device lookup", {
        device: request.device_id,
        found: Boolean(target),
        deviceCount: devices.length,
      });

      if (!target) throw new Error("The recovering device is no longer registered.");

      const peer = users.find((item) => item.user_id === request.requester_id)
        || { user_id: request.requester_id, display_name: request.requester_id };

      if (!(await ensurePeerFingerprint(peer, target))) {
        throw new Error("The recovering device is not verified.");
      }

      if (
        original?.version === 1
        && ["message", "prekey"].includes(original.message_type)
      ) {
        // Legacy v1 used the peer account id as the Olm session key.
        const legacySessionId = request.requester_id;
        console.log("[E2E] recovery: establishing fresh legacy v1 session");
        const devicesResponse = await api(
          "GET",
          `/api/users/${encodeURIComponent(request.requester_id)}/crypto-key`
        );
        const requesterDevices = Array.isArray(devicesResponse?.devices)
          ? devicesResponse.devices
          : [];

        const requesterBundle = requesterDevices.find(
          (device) => device?.device_id === request.device_id
        ) || requesterDevices[0];

        if (!requesterBundle) {
          throw new Error("The recovering device has no E2E bundle.");
        }

        if (!(await ensurePeerFingerprint(peer, requesterBundle))) {
          throw new Error(`Device ${request.device_id} could not be verified.`);
        }

        const claimedBundle = requesterDevices.length
          ? await claimPeerOneTimeKey(request.requester_id, requesterBundle.device_id)
          : requesterBundle;

        if (!(await ensurePeerFingerprint(peer, claimedBundle))) {
          throw new Error(`Device ${request.device_id} could not be verified.`);
        }

        cryptoDevice.establish_session(
          legacySessionId,
          JSON.stringify(claimedBundle),
          claimedBundle.fingerprint,
        );

        const recoveryCipher = cryptoDevice.encrypt(legacySessionId, cached);
        const recoveryCipherEnvelope = JSON.parse(recoveryCipher);
        if (
          recoveryCipherEnvelope?.version !== 1
          || !["message", "prekey"].includes(recoveryCipherEnvelope.message_type)
        ) {
          throw new Error("automatic legacy recovery did not produce a v1 ciphertext");
        }

        await persistCryptoState();

        socket?.send(JSON.stringify({
          type: "crypto_resync_response",
          peer_id: request.requester_id,
          message_id: request.message_id,
          device_id: request.device_id,
          ciphertext: recoveryCipher,
        }));

        console.log("[E2E] recovery: legacy v1 ciphertext sent", {
          peer: request.requester_id,
          message: request.message_id,
          session: legacySessionId,
          sessionCount: cryptoDevice.session_count(legacySessionId),
        });
        return;
      }

      console.log("[E2E] recovery: claiming one-time key");

      const claimedBundle = await claimPeerOneTimeKey(request.requester_id, target.device_id);

      console.log("[E2E] recovery: one-time key claimed", {
        device: claimedBundle?.device_id,
        hasOneTimeKey: Array.isArray(claimedBundle?.one_time_keys)
          && claimedBundle.one_time_keys.length > 0,
      });

      if (!(await ensurePeerFingerprint(peer, claimedBundle))) {
        throw new Error(`Device ${target.device_id} could not be verified.`);
      }

      cryptoDevice.establish_session(
        target.device_id,
        JSON.stringify(claimedBundle),
        claimedBundle.fingerprint,
      );

      console.log("[E2E] recovery: new outbound session established", {
        device: target.device_id,
        sessionCount: cryptoDevice.session_count(target.device_id),
      });

      const recoveryCipher = cryptoDevice.encrypt(target.device_id, cached);
      const recoveryCipherEnvelope = JSON.parse(recoveryCipher);
      if (recoveryCipherEnvelope?.message_type !== "prekey") {
        throw new Error("automatic recovery did not produce a pre-key message");
      }

      await persistCryptoState();

      socket?.send(JSON.stringify({
        type: "crypto_resync_response",
        peer_id: request.requester_id,
        message_id: request.message_id,
        device_id: request.device_id,
        ciphertext: recoveryCipher,
      }));

      console.log("[E2E] recovery: pre-key response sent", {
        peer: request.requester_id,
        message: request.message_id,
        device: target.device_id,
        sessionCount: cryptoDevice.session_count(target.device_id),
      });
    });
  } catch (err) {
    console.error("[E2E] automatic session recovery failed", {
      error: err?.message || String(err),
      stack: err?.stack || null,
    });
  }
}

function parseEncryptedPayload(raw) {
  if (typeof raw !== "string") return null;
  try {
    const payload = JSON.parse(raw);
    if (payload && typeof payload.text === "string") return payload;
  } catch {
    return { text: raw, file: null };
  }
  return null;
}

async function encryptAttachment(file) {
  const key = await crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, true, ["encrypt"]);
  const rawKey = await crypto.subtle.exportKey("raw", key);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, await file.arrayBuffer());
  return {
    file: new File([ciphertext], `${file.name}.encrypted`, { type: "application/octet-stream" }),
    metadata: {
      key: bytesToBase64(new Uint8Array(rawKey)),
      iv: bytesToBase64(iv),
      name: file.name,
      mime: file.type || "application/octet-stream",
      size: file.size,
    },
  };
}

async function renderEncryptedAttachment(attachment, metadata, container) {
  const response = await fetch(attachment.url, { credentials: "same-origin" });
  if (!response.ok) throw new Error("Encrypted attachment could not be loaded.");
  const ciphertext = await response.arrayBuffer();
  const key = await crypto.subtle.importKey("raw", base64ToBytes(metadata.key), "AES-GCM", false, ["decrypt"]);
  const plaintext = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: base64ToBytes(metadata.iv) },
    key,
    ciphertext,
  );
  const blob = new Blob([plaintext], { type: metadata.mime });
  const url = URL.createObjectURL(blob);
  if (metadata.mime.startsWith("image/")) {
    const image = document.createElement("img");
    image.className = "photo";
    image.src = url;
    image.alt = metadata.name;
    image.tabIndex = 0;
    image.setAttribute("role", "button");
    image.addEventListener("click", () => openImageViewer(url, metadata.name));
    container.append(image);
  } else if (metadata.mime.startsWith("audio/")) {
    const audio = document.createElement("audio");
    audio.controls = true;
    audio.preload = "metadata";
    audio.src = url;
    container.append(audio);
  } else {
    const link = document.createElement("a");
    link.href = url;
    link.download = metadata.name;
    link.textContent = `${metadata.name} (${formatSize(metadata.size)})`;
    container.append(link);
  }
}

function bytesToBase64(bytes) {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

function base64ToBytes(value) {
  return Uint8Array.from(atob(value), (character) => character.charCodeAt(0));
}

function queueAttachment(file) {
  if (!file) return;
  pendingAttachment = file;
  if (previewUrl) URL.revokeObjectURL(previewUrl);
  previewUrl = URL.createObjectURL(file);
  attachmentPreview.replaceChildren();
  if (file.type.startsWith("image/")) {
    const image = document.createElement("img");
    image.src = previewUrl;
    image.alt = "Selected image preview";
    attachmentPreview.append(image);
  } else if (file.type.startsWith("audio/")) {
    const audio = document.createElement("audio");
    audio.controls = true;
    audio.src = previewUrl;
    attachmentPreview.append(audio);
  }
  const details = document.createElement("span");
  details.textContent = `${file.name} · ${formatSize(file.size)}`;
  attachmentPreview.append(details);
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "ghost";
  remove.textContent = "Remove";
  remove.addEventListener("click", clearAttachment);
  attachmentPreview.append(remove);
  attachmentPreview.hidden = false;
}

function clearAttachment() {
  pendingAttachment = null;
  if (previewUrl) URL.revokeObjectURL(previewUrl);
  previewUrl = null;
  attachmentPreview.replaceChildren();
  attachmentPreview.hidden = true;
  photoInput.value = "";
  gifInput.value = "";
  fileInput.value = "";
  audioFileInput.value = "";
}

async function toggleRecording() {
  if (recorder && recorder.state === "recording") {
    recorder.stop();
    recordAudioButton.textContent = "Record";
    return;
  }
  if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") {
    appendSystem("Audio recording is not supported by this browser.");
    return;
  }
  try {
    recordingStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const mimeType = ["audio/webm;codecs=opus", "audio/ogg;codecs=opus"]
      .find((type) => MediaRecorder.isTypeSupported(type));
    recorder = new MediaRecorder(recordingStream, mimeType ? { mimeType } : undefined);
    recordedChunks = [];
    recorder.addEventListener("dataavailable", (event) => {
      if (event.data.size) recordedChunks.push(event.data);
    });
    recorder.addEventListener("stop", () => {
      const blob = new Blob(recordedChunks, { type: recorder.mimeType || "audio/webm" });
      const ext = blob.type.includes("ogg") ? "ogg" : "webm";
      queueAttachment(new File([blob], `voice-message.${ext}`, { type: blob.type }));
      recordingStream.getTracks().forEach((track) => track.stop());
      recordingStream = null;
    }, { once: true });
    recorder.start();
    recordAudioButton.textContent = "Stop recording";
  } catch (err) {
    appendSystem(err.message || "Could not access the microphone.");
    recordingStream?.getTracks().forEach((track) => track.stop());
    recordingStream = null;
  }
}

function formatSize(size) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

function appendSystem(text) {
  const li = document.createElement("li");
  li.textContent = text;
  logEl.append(li);
}

function openImageViewer(src, alt) {
  imageViewerImage.src = src;
  imageViewerImage.alt = alt;
  imageViewer.showModal();
}

function updateOwnProfileCard(profile) {
  profileCardName.textContent = profile.display_name || "";
  profileCardHandle.textContent = profile.username ? `@${profile.username}` : "";
  profileCardActivity.textContent = profile.activity || "No activity";
}

async function setMyActivity(activity) {
  try {
    const result = await api("POST", "/api/me/activity", { activity });
    profileCardActivity.textContent = result.activity || "No activity";
  } catch {
    return;
  }
}

function updateMusicActivity(trackName, playing) {
  if (!musicActivityEnabled || customActivity) return;
  void setMyActivity(playing ? `Listening to ${trackName}` : "");
}

window.larptixMusicStatus = {
  update: updateMusicActivity,
  refresh() {
    if (!musicActivityEnabled || customActivity) return;
    const player = document.getElementById("music-audio");
    const name = document.getElementById("music-player-name")?.textContent?.trim();
    if (player && name) updateMusicActivity(name, !player.paused);
  },
};

function chatWallpaperKey(id) {
  return `larptrix_chat_wallpaper_${me.user_id}_${id}`;
}

function applyChatWallpaper(id) {
  const wallpaper = localStorage.getItem(chatWallpaperKey(id));
  logEl.style.backgroundImage = wallpaper ? `url("${wallpaper}")` : "";
}

async function saveChatWallpaper(file, id) {
  if (!file.type.startsWith("image/")) return;
  try {
    const image = await createImageBitmap(file);
    const scale = Math.min(1, 1600 / image.width, 1000 / image.height);
    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(image.width * scale));
    canvas.height = Math.max(1, Math.round(image.height * scale));
    canvas.getContext("2d").drawImage(image, 0, 0, canvas.width, canvas.height);
    image.close();
    localStorage.setItem(chatWallpaperKey(id), canvas.toDataURL("image/jpeg", 0.78));
    applyChatWallpaper(id);
  } catch {
    appendSystem("Could not save this chat background. Try a smaller image.");
  }
}

async function showPeerProfile(id) {
  try {
    const profile = await api("GET", `/api/users/${encodeURIComponent(id)}/profile`);
    const avatar = document.getElementById("peer-profile-avatar");
    avatar.hidden = !profile.avatar_url;
    if (profile.avatar_url) avatar.src = profile.avatar_url;
    avatar.alt = `${profile.display_name} profile photo`;
    document.getElementById("peer-profile-name").textContent = profile.display_name;
    document.getElementById("peer-profile-username").textContent = profile.username ? `@${profile.username}` : "";
    document.getElementById("peer-profile-about").textContent = profile.about || "No profile description";
    document.getElementById("peer-profile-activity").textContent = profile.activity || "No activity";
    peerProfileDialog.showModal();
  } catch (err) {
    appendSystem(err.message || "Could not load profile.");
  }
}

async function api(method, path, body) {
  const options = { method, credentials: "same-origin", headers: {} };
  if (body !== undefined && method !== "GET") {
    options.headers["Content-Type"] = "application/json";
    options.body = JSON.stringify(body);
  }
  const response = await fetch(path, options);
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    const error = new Error(data.error || response.statusText);
    error.status = response.status;
    throw error;
  }
  return data;
}

async function uploadFile(path, file) {
  const form = new FormData();
  form.append("file", file);
  const response = await fetch(path, {
    method: "POST",
    credentials: "same-origin",
    body: form,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(data.error || response.statusText);
  return data;
}
