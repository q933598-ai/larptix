import initCrypto, { CryptoDevice } from "/crypto-pkg/larptrix_crypto_wasm.js";
import {
  LarptrixMatrixCrypto,
  getOrCreateMatrixDeviceId,
} from "/matrix-crypto.js";

const cryptoWasmReady = initCrypto();

const statusEl = document.getElementById("status");
const usersEl = document.getElementById("users");
const logEl = document.getElementById("log");
const composer = document.getElementById("composer");
const bodyInput = document.getElementById("body");
const gate = document.getElementById("gate");
const authForm = document.getElementById("auth");
const authError = document.getElementById("auth-error");
const accessKeyInput = document.getElementById("access-key");
const usernameInput = document.getElementById("username");
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
const gifOpenButton = document.getElementById("gif-open");
const gifDialog = document.getElementById("gif-dialog");
const gifCloseButton = document.getElementById("gif-close");
const gifResults = document.getElementById("gif-results");
const gifEmpty = document.getElementById("gif-empty");
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
const menuSaved = document.getElementById("menu-saved");
const menuNewGroup = document.getElementById("menu-new-group");
const menuSettings = document.getElementById("menu-settings");
const settingsDialog = document.getElementById("settings-dialog");
const settingsServer = document.getElementById("settings-server");
const settingsName = document.getElementById("settings-name");
const settingsUsername = document.getElementById("settings-username");
const settingsE2eStatus = document.getElementById("settings-e2e-status");
const settingsE2eFingerprint = document.getElementById("settings-e2e-fingerprint");
const settingsLayoutStatus = document.getElementById("settings-layout-status");
const settingsTheme = document.getElementById("settings-theme");
const customThemeEditor = document.getElementById("custom-theme-editor");
const themeBg = document.getElementById("theme-bg");
const themePanel = document.getElementById("theme-panel");
const themeText = document.getElementById("theme-text");
const themeMuted = document.getElementById("theme-muted");
const themeAccent = document.getElementById("theme-accent");
const themeMe = document.getElementById("theme-me");
const themeSaveCustom = document.getElementById("theme-save-custom");
const settingsPresence = document.getElementById("settings-presence");
const settingsCallSounds = document.getElementById("settings-call-sounds");
const settingsMessageSounds = document.getElementById("settings-message-sounds");
const settingsNoiseSuppression = document.getElementById("settings-noise-suppression");
const mobileChats = document.getElementById("mobile-chats");
const mobileSaved = document.getElementById("mobile-saved");
const mobileMusic = document.getElementById("mobile-music");
const mobileSettings = document.getElementById("mobile-settings");
const savedMessagesDialog = document.getElementById("saved-messages-dialog");
const savedMessagesClose = document.getElementById("saved-messages-close");
const savedMessagesList = document.getElementById("saved-messages-list");
const savedMessagesEmpty = document.getElementById("saved-messages-empty");
const groupCallStart = document.getElementById("group-call-start");
const groupCallBanner = document.getElementById("group-call-banner");
const groupCallBannerTitle = document.getElementById("group-call-banner-title");
const groupCallBannerMeta = document.getElementById("group-call-banner-meta");
const groupCallJoin = document.getElementById("group-call-join");
const callWindowTitle = document.getElementById("call-window-title");
const callRingtone = document.getElementById("call-ringtone");
const groupMembersOpen = document.getElementById("group-members-open");
const groupMembersDialog = document.getElementById("group-members-dialog");
const groupMembersClose = document.getElementById("group-members-close");
const groupMembersTitle = document.getElementById("group-members-title");
const groupMembersHelp = document.getElementById("group-members-help");
const groupMembersList = document.getElementById("group-members-list");
const groupCallInvite = document.getElementById("group-call-invite");
const groupCallCount = document.getElementById("group-call-count");


let socket = null;
let socketHeartbeatTimer = null;
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
let pendingGif = null;
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
let registrationE2eRequired = false;
let cryptoDialogRequired = false;

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
const deletedMessageIds = new Set();

function pinnedChatsKey() {
  return me ? `larptrix_pinned_chats_${me.user_id}` : null;
}

function getPinnedChats() {
  if (!me) return new Set();
  try {
    const value = JSON.parse(localStorage.getItem(pinnedChatsKey()) || "[]");
    return new Set(Array.isArray(value) ? value.filter((id) => typeof id === "string") : []);
  } catch {
    return new Set();
  }
}

function setPinnedChats(pinned) {
  const key = pinnedChatsKey();
  if (!key) return;
  localStorage.setItem(key, JSON.stringify([...pinned]));
}

function togglePinnedChat(id) {
  const pinned = getPinnedChats();
  if (pinned.has(id)) pinned.delete(id);
  else pinned.add(id);
  setPinnedChats(pinned);
  renderUsers();
}

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

let groupCallId = null;
let groupCallGroupId = null;
let groupCallMemberIds = [];
const groupCallJoinedMembers = new Set();
const groupPeerConnections = new Map();
const activeGroupCalls = new Map();
const presenceByUserId = new Map();
let groupCallInitiatorId = null;
let groupCallWindowDragging = false;
let groupCallWindowDragOffsetX = 0;
let groupCallWindowDragOffsetY = 0;
const groupPendingIceCandidates = new Map();

const THEME_KEY = "larptrix_theme";
const CUSTOM_THEME_KEY = "larptrix_custom_theme";
const PRESENCE_KEY = "larptrix_presence";
const CALL_SOUND_KEY = "larptrix_call_sounds";
const MESSAGE_SOUND_KEY = "larptrix_message_sounds";
const NOISE_SUPPRESSION_KEY = "larptrix_noise_suppression";
const SAVED_MESSAGES_KEY = "larptrix_saved_messages_v1";
function savedMessagesKey() {
  return me ? SAVED_MESSAGES_KEY + "_" + me.user_id : SAVED_MESSAGES_KEY;
}

function readStoredBool(key, fallback = true) {
  const value = localStorage.getItem(key);
  return value === null ? fallback : value === "1";
}

function applyTheme(name = localStorage.getItem(THEME_KEY) || "larptrix") {
  const root = document.documentElement;
  root.dataset.theme = name;
  if (name === "custom") {
    try {
      const custom = JSON.parse(localStorage.getItem(CUSTOM_THEME_KEY) || "null");
      for (const [key, value] of Object.entries(custom || {})) {
        if (typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value)) root.style.setProperty("--" + key, value);
      }
    } catch {}
  } else {
    for (const key of ["bg", "panel", "panel-2", "panel-3", "line", "line-bright", "text", "muted", "accent", "accent-strong", "me", "danger"]) {
      root.style.removeProperty("--" + key);
    }
  }
  localStorage.setItem(THEME_KEY, name);
}

function loadThemeEditor() {
  let custom = null;
  try { custom = JSON.parse(localStorage.getItem(CUSTOM_THEME_KEY) || "null"); } catch {}
  custom = custom || {
    bg: "#050b08", panel: "#08130e", "panel-2": "#0b1912", text: "#d7f3df", muted: "#6f9b7e",
    accent: "#35d47a", me: "#0d3020",
  };
  themeBg.value = custom.bg || "#050b08";
  themePanel.value = custom.panel || "#08130e";
  themeText.value = custom.text || "#d7f3df";
  themeMuted.value = custom.muted || "#6f9b7e";
  themeAccent.value = custom.accent || "#35d47a";
  themeMe.value = custom.me || "#0d3020";
}

function saveCustomTheme() {
  const custom = {
    bg: themeBg.value,
    panel: themePanel.value,
    "panel-2": themePanel.value,
    "panel-3": themePanel.value,
    line: mixThemeColor(themePanel.value, themeText.value, 0.16),
    "line-bright": mixThemeColor(themePanel.value, themeAccent.value, 0.34),
    text: themeText.value,
    muted: themeMuted.value,
    accent: themeAccent.value,
    "accent-strong": themeAccent.value,
    me: themeMe.value,
    danger: "#ef6d73",
  };
  localStorage.setItem(CUSTOM_THEME_KEY, JSON.stringify(custom));
  localStorage.setItem(THEME_KEY, "custom");
  applyTheme("custom");
  settingsTheme.value = "custom";
}

function mixThemeColor(a, b, amount) {
  const parse = (value) => [1, 3, 5].map((i) => parseInt(value.slice(i, i + 2), 16));
  const ca = parse(a);
  const cb = parse(b);
  return "#" + ca.map((v, i) => Math.round(v + (cb[i] - v) * amount).toString(16).padStart(2, "0")).join("");
}

function getPresence(user) {
  if (!user) return "offline";
  const status = presenceByUserId.get(user.user_id);
  if (!user.online) return "offline";
  return status === "dnd" || status === "invisible" || status === "online" ? status : "online";
}

function setStatus(text) {
  if (text === "online") {
    renderPresenceStatus(localStorage.getItem(PRESENCE_KEY) || "online");
    return;
  }
  renderPresenceStatus(text);
}

function renderPresenceStatus(status) {
  const value = status || "offline";
  statusEl.textContent = value === "dnd" ? "Do Not Disturb" : value === "invisible" ? "Invisible" : value;
  statusEl.classList.toggle("online", value === "online");
  statusEl.classList.toggle("dnd", value === "dnd");
}

function setPresence(status) {
  const value = ["online", "dnd", "invisible"].includes(status) ? status : "online";
  localStorage.setItem(PRESENCE_KEY, value);
  presenceByUserId.set(me?.user_id || "", value);
  renderPresenceStatus(value);
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "set_presence", status: value }));
  }
}

function playIncomingMessageSound() {
  if (!readStoredBool(MESSAGE_SOUND_KEY, true) || localStorage.getItem(PRESENCE_KEY) === "dnd") return;
  try {
    const context = new AudioContext();
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.frequency.value = 880;
    oscillator.type = "sine";
    gain.gain.setValueAtTime(0.0001, context.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.07, context.currentTime + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, context.currentTime + 0.11);
    oscillator.connect(gain).connect(context.destination);
    oscillator.start();
    oscillator.stop(context.currentTime + 0.12);
    setTimeout(() => context.close().catch(() => {}), 250);
  } catch {}
}

function startCallRingtone() {
  if (!readStoredBool(CALL_SOUND_KEY, true) || localStorage.getItem(PRESENCE_KEY) === "dnd") return;
  if (!callRingtone) return;
  callRingtone.currentTime = 0;
  callRingtone.loop = true;
  callRingtone.play().catch(() => {});
}

function stopCallRingtone() {
  if (!callRingtone) return;
  callRingtone.pause();
  callRingtone.currentTime = 0;
}

async function getSavedMessages() {
  if (!cryptoRecoveryKey) return [];
  try {
    const raw = localStorage.getItem(savedMessagesKey());
    if (!raw) return [];
    const record = JSON.parse(raw);
    const key = await sentPlaintextCacheKey();
    if (!key || !record?.iv || !record?.ciphertext) return [];
    const plaintext = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: new Uint8Array(record.iv) },
      key,
      new Uint8Array(record.ciphertext),
    );
    const parsed = JSON.parse(new TextDecoder().decode(plaintext));
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function setSavedMessages(items) {
  if (!cryptoRecoveryKey) return;
  const key = await sentPlaintextCacheKey();
  if (!key) return;
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    new TextEncoder().encode(JSON.stringify(items)),
  );
  localStorage.setItem(savedMessagesKey(), JSON.stringify({
    iv: Array.from(iv),
    ciphertext: Array.from(new Uint8Array(encrypted)),
  }));
}

async function toggleSavedMessage(message, textValue) {
  if (!cryptoRecoveryKey) {
    appendSystem("Unlock E2E before saving messages.");
    return;
  }
  if (!textValue || textValue === "Encrypted message" || textValue.startsWith("Could not decrypt")) {
    appendSystem("Wait for the message to decrypt before saving it.");
    return;
  }
  const items = await getSavedMessages();
  const index = items.findIndex((item) => item.id === message.id);
  if (index >= 0) {
    items.splice(index, 1);
  } else {
    const peer = message.sender_id === me?.user_id ? message.recipient_id : message.sender_id;
    items.unshift({
      id: message.id,
      peer_id: peer,
      sender_id: message.sender_id,
      sender_name: message.sender_name,
      text: textValue,
      created_at: message.created_at,
    });
  }
  await setSavedMessages(items.slice(0, 500));
  await renderSavedMessages();
}

async function renderSavedMessages() {
  if (!savedMessagesList || !savedMessagesEmpty) return;
  const items = await getSavedMessages();
  savedMessagesList.replaceChildren();
  savedMessagesEmpty.hidden = items.length > 0;
  for (const item of items) {
    const row = document.createElement("article");
    row.className = "saved-message-row";
    const meta = document.createElement("span");
    meta.className = "saved-message-meta";
    meta.textContent = item.sender_name + " · " + new Date(item.created_at).toLocaleString();
    const text = document.createElement("p");
    text.textContent = item.text;
    const actions = document.createElement("div");
    actions.className = "saved-message-actions";
    const open = document.createElement("button");
    open.type = "button";
    open.className = "ghost";
    open.textContent = "Open chat";
    open.addEventListener("click", () => {
      savedMessagesDialog?.close();
      if (item.peer_id) openChat(item.peer_id);
    });
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "ghost";
    remove.textContent = "Remove";
    remove.addEventListener("click", async () => {
      const next = (await getSavedMessages()).filter((entry) => entry.id !== item.id);
      await setSavedMessages(next);
      await renderSavedMessages();
    });
    actions.append(open, remove);
    row.append(meta, text, actions);
    savedMessagesList.append(row);
  }
}

function openSavedMessages() {
  closeAppMenu();
  void renderSavedMessages();
  savedMessagesDialog?.showModal();
}

applyTheme();

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
  settingsTheme.value = localStorage.getItem(THEME_KEY) || "larptrix";
  loadThemeEditor();
  customThemeEditor.hidden = settingsTheme.value !== "custom";
  settingsPresence.value = localStorage.getItem(PRESENCE_KEY) || "online";
  settingsCallSounds.checked = readStoredBool(CALL_SOUND_KEY, true);
  settingsMessageSounds.checked = readStoredBool(MESSAGE_SOUND_KEY, true);
  settingsNoiseSuppression.checked = readStoredBool(NOISE_SUPPRESSION_KEY, true);
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
  window.larptixMusicLibrary?.toggle?.();
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
settingsTheme.addEventListener("change", () => {
  customThemeEditor.hidden = settingsTheme.value !== "custom";
  if (settingsTheme.value === "custom") loadThemeEditor();
  applyTheme(settingsTheme.value);
});
themeSaveCustom.addEventListener("click", saveCustomTheme);
settingsPresence.addEventListener("change", () => setPresence(settingsPresence.value));
settingsCallSounds.addEventListener("change", () => localStorage.setItem(CALL_SOUND_KEY, settingsCallSounds.checked ? "1" : "0"));
settingsMessageSounds.addEventListener("change", () => localStorage.setItem(MESSAGE_SOUND_KEY, settingsMessageSounds.checked ? "1" : "0"));
settingsNoiseSuppression.addEventListener("change", () => localStorage.setItem(NOISE_SUPPRESSION_KEY, settingsNoiseSuppression.checked ? "1" : "0"));
menuSaved?.addEventListener("click", openSavedMessages);
savedMessagesClose?.addEventListener("click", () => savedMessagesDialog.close());
mobileChats?.addEventListener("click", () => {
  document.getElementById("chat-view-open").click();
  document.body.classList.toggle("mobile-people-visible");
});
mobileSaved?.addEventListener("click", () => {
  document.body.classList.remove("mobile-people-visible");
  openSavedMessages();
});
mobileMusic?.addEventListener("click", () => {
  document.body.classList.remove("mobile-people-visible");
  window.larptixMusicLibrary?.toggle?.();
});
mobileSettings?.addEventListener("click", () => {
  document.body.classList.remove("mobile-people-visible");
  openSettings();
});

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
      signedIn(user, { registrationE2eRequired: true });
    } else {
      signedIn(user, { registrationE2eRequired: true });
    }
  } catch (err) {
    keyError.textContent = err.message;
    keyError.hidden = false;
    keyContinue.disabled = false;
  }
});
gifOpenButton.addEventListener("click", () => {
  gifEmpty.textContent = "No saved GIFs yet. Save a GIF from any chat message.";
  void renderGifFavorites();
  gifDialog.showModal();
});
gifCloseButton.addEventListener("click", () => gifDialog.close());
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
enableE2eButton.addEventListener("click", async () => {
  try {
    await cryptoWasmReady;
  } catch (err) {
    profileError.textContent = err.message || "Could not initialize E2E encryption.";
    profileError.hidden = false;
    return;
  }
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
  if (cryptoDialogRequired) {
    cryptoError.textContent = "E2E setup is mandatory when creating a new account.";
    cryptoError.hidden = false;
    return;
  }
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
          username: usernameInput.value.trim(),
          email: emailInput.value.trim(),
          password: passwordInput.value,
        });
        localStorage.setItem(registrationE2eKey(user.user_id), "1");
        signedIn(user, { registrationE2eRequired: true });
        return;
      }
      const result = await api("POST", "/api/register", {
        display_name: nameInput.value.trim(),
        username: usernameInput.value.trim(),
      });
      localStorage.setItem(registrationE2eKey(result.user.user_id), "1");
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
  profileOpen.hidden = true;
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
        await matrixCryptoReady;
        let matrixReady = Boolean(matrixCrypto);

        if (matrixReady) {
          try {
            await waitForMatrixDevices(
              peer.is_group ? peer.group_member_ids : [peerId],
            );
          } catch (err) {
            console.warn(
              "[E2E] Matrix device readiness check failed; using legacy E2E fallback",
              err,
            );
            matrixReady = false;
          }
        }

        if (peer.is_group) {
          if (!matrixReady) {
            throw new Error("Group E2E requires Matrix crypto. Keep E2E unlocked and try again.");
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
          if (matrixReady) {
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
fileInput.addEventListener("change", () => queueAttachment(fileInput.files[0]));
audioFileInput.addEventListener("change", () => queueAttachment(audioFileInput.files[0]));
recordAudioButton.addEventListener("click", toggleRecording);
document.getElementById("start-audio-call").addEventListener("click", () => startCall("audio"));
document.getElementById("start-video-call").addEventListener("click", () => startCall("video"));
groupMembersOpen?.addEventListener("click", openGroupMembers);
groupMembersClose?.addEventListener("click", () => groupMembersDialog.close());
groupCallInvite?.addEventListener("click", openGroupMembers);
groupCallStart?.addEventListener("click", () => {
  const state = activeGroupCalls.get(peerId);
  if (state?.active) void joinActiveGroupCall();
  else void startGroupCall("audio");
});
groupCallJoin?.addEventListener("click", () => void joinActiveGroupCall());
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
    signedIn(user, {
      registrationE2eRequired:
        localStorage.getItem(registrationE2eKey(user.user_id)) === "1",
    });
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

function openCryptoDialog(mode, { required = false } = {}) {
  cryptoDialogMode = mode;
  cryptoDialogRequired = required;
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
    cryptoContinue.disabled = !cryptoConfirm.checked;
  } else {
    cryptoDialogTitle.textContent = "Unlock encrypted chats";
    cryptoDialogDescription.textContent = "Enter the separate recovery key you saved when enabling E2E. It is not your sign-in key.";
    cryptoContinue.textContent = "Unlock chats";
    cryptoContinue.disabled = true;
  }
  document.getElementById("crypto-cancel").hidden = cryptoDialogRequired;
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
      cryptoDialogRequired = false;
      registrationE2eRequired = false;
      if (me) localStorage.removeItem(registrationE2eKey(me.user_id));
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

async function loadCryptoStatus({ forceSetup = false } = {}) {
  await cryptoWasmReady;
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
      // A new E2E identity is mandatory: there is no "skip" path.
      openCryptoDialog("enable", { required: true });
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

async function waitForMatrixDevices(userIds, { attempts = 8, delayMs = 350 } = {}) {
  const ids = [...new Set(userIds.filter((id) => id && id !== me?.user_id))];
  if (!ids.length) return;

  let lastMissing = [];
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    lastMissing = [];
    await Promise.all(ids.map(async (userId) => {
      try {
        const result = await api(
          "GET",
          `/api/users/${encodeURIComponent(userId)}/matrix-devices`,
        );
        const devices = Array.isArray(result?.devices) ? result.devices : [];
        if (!devices.length) lastMissing.push(userId);
      } catch {
        lastMissing.push(userId);
      }
    }));

    if (!lastMissing.length) return;
    if (attempt + 1 < attempts) {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }

  throw new Error(
    lastMissing.length === 1
      ? "The recipient's Matrix E2E device is still registering. Ask them to keep Larptrix open for a moment and try again."
      : "One or more group members' Matrix E2E devices are still registering. Ask them to keep Larptrix open for a moment and try again.",
  );
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

function registrationE2eKey(userId) {
  return `larptrix_registration_e2e_required_${userId}`;
}

function signedIn(user, options = {}) {
  me = user;
  registrationE2eRequired = Boolean(
    options.registrationE2eRequired
      || localStorage.getItem(registrationE2eKey(user.user_id)) === "1",
  );

  // Establish the authenticated realtime connection before any UI or E2E
  // initialization can interfere with startup. Once the server sends
  // "welcome", connect() will mark the status online.
  connect();

  forgetE2eDeviceButton.hidden = true;
  gate.close();
  logoutBtn.hidden = false;
  renderMe();
  cryptoReady = loadCryptoStatus({ forceSetup: registrationE2eRequired }).catch((err) => {
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
  usernameInput.hidden = next !== "register";
  usernameInput.required = next === "register";
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
  const nextSocket = new WebSocket(`${proto}://${location.host}/ws`);
  socket = nextSocket;
  setStatus("connecting");
  reconnect = true;

  nextSocket.addEventListener("open", () => {
    if (socket !== nextSocket) return;
    setStatus("online");
    const savedPresence = localStorage.getItem(PRESENCE_KEY) || "online";
    if (me) nextSocket.send(JSON.stringify({ type: "set_presence", status: savedPresence }));
    if (socketHeartbeatTimer) clearInterval(socketHeartbeatTimer);
    socketHeartbeatTimer = setInterval(() => {
      if (socket !== nextSocket || nextSocket.readyState !== WebSocket.OPEN) return;
      try {
        nextSocket.send(JSON.stringify({ type: "ping" }));
      } catch {}
    }, 20_000);
  });

  nextSocket.addEventListener("message", (event) => {
    if (socket !== nextSocket) return;
    const msg = JSON.parse(event.data);
    switch (msg.type) {
      case "pong":
        // Receiving a server response confirms that the WebSocket path is
        // still alive even when there are no chat events.
        if (socket === nextSocket && nextSocket.readyState === WebSocket.OPEN) setStatus("online");
        break;
      case "welcome":
        // The server sends welcome only after the authenticated WebSocket
        // connection has been fully established. Use it as a definitive
        // online signal in addition to the WebSocket open event.
        if (socket === nextSocket && nextSocket.readyState === WebSocket.OPEN) {
          setStatus("online");
        }
        me = msg.user;
        presenceByUserId.set(me.user_id, localStorage.getItem(PRESENCE_KEY) || "online");
        renderPresenceStatus(presenceByUserId.get(me.user_id));
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
        renderGroupCallBanner();
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
        renderGroupCallBanner();
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
        document.getElementById("start-audio-call").hidden = false;
        document.getElementById("start-video-call").hidden = false;
        document.getElementById("start-audio-call").textContent = msg.peer.is_group ? "Group audio" : "Call";
        document.getElementById("start-video-call").textContent = msg.peer.is_group ? "Group video" : "Video";
        groupCallStart.hidden = !msg.peer.is_group;
        groupCallStart.textContent = msg.peer.is_group
          ? (activeGroupCalls.get(msg.peer.user_id)?.active ? "Join group call" : "Group call")
          : "Group call";
        renderGroupCallBanner();
        logEl.replaceChildren();
        messageBodyElementsById.clear();
        msg.history.forEach(appendMessage);
        break;
      case "message":
        if (isForOpenChat(msg.message)) appendMessage(msg.message);
        if (msg.message?.sender_id !== me?.user_id) playIncomingMessageSound();
        break;
      case "presence":
        if (msg.user_id) {
          presenceByUserId.set(msg.user_id, msg.status);
          if (msg.user_id === me?.user_id) renderPresenceStatus(msg.status);
          renderUsers();
        }
        break;
      case "message_deleted":
        deletedMessageIds.add(msg.message_id);
        messagesById.delete(msg.message_id);
        messageBodyElementsById.delete(msg.message_id);
        cryptoRecoveryPending.delete(msg.message_id);
        recoveredBodiesByMessageId.delete(msg.message_id);
        cryptoRecoveryResponsesByMessageId.delete(msg.message_id);
        logEl.querySelector(`[data-message-id="${CSS.escape(msg.message_id)}"]`)?.remove();
        break;
      case "group_call_state":
        handleGroupCallState(msg);
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

  nextSocket.addEventListener("close", () => {
    if (socket !== nextSocket) return;
    if (socketHeartbeatTimer) {
      clearInterval(socketHeartbeatTimer);
      socketHeartbeatTimer = null;
    }
    setStatus("offline");
    endCall(false);
    if (reconnect) setTimeout(connect, 1500);
  });
}

function renderGroupMembersDialog(group) {
  if (!groupMembersList || !group) return;
  const memberIds = Array.isArray(group.group_member_ids) ? group.group_member_ids : [];
  groupMembersTitle.textContent = group.display_name;
  groupMembersHelp.textContent = `${memberIds.length} member(s). Members with an active connection can be invited to the current call.`;
  groupMembersList.replaceChildren();

  for (const memberId of memberIds) {
    const user = users.find((item) => item.user_id === memberId);
    const joined = groupCallId && groupCallGroupId === group.user_id && groupCallJoinedMembers.has(memberId);
    const online = Boolean(user?.online);
    const row = document.createElement("div");
    row.className = "group-member-row";

    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user || { user_id: memberId, display_name: "?" });

    const copy = document.createElement("div");
    copy.className = "group-member-copy";
    const name = document.createElement("strong");
    name.textContent = user?.display_name || "Unknown member";
    const meta = document.createElement("span");
    meta.textContent = user?.username ? `@${user.username} · ${online ? "Online" : "Offline"}` : (online ? "Online" : "Offline");
    copy.append(name, meta);

    const action = document.createElement("button");
    action.type = "button";
    action.className = "ghost group-member-action";
    if (memberId === me?.user_id) {
      action.textContent = "You";
      action.disabled = true;
    } else if (joined) {
      action.textContent = "In call";
      action.disabled = true;
    } else if (groupCallId && groupCallGroupId === group.user_id && online) {
      action.textContent = "Invite";
      action.addEventListener("click", () => {
        sendGroupCallSignal(memberId, "group_invite", {
          group_id: group.user_id,
          call_id: groupCallId,
          media: callMediaKind,
        });
        action.textContent = "Invited";
        action.disabled = true;
      });
    } else {
      action.textContent = online ? "Available" : "Offline";
      action.disabled = true;
    }

    row.append(avatar, copy, action);
    groupMembersList.append(row);
  }
}

function refreshGroupCallParticipants() {
  const group = groups.find((item) => item.user_id === groupCallGroupId && item.is_group);
  const active = Boolean(groupCallId && groupCallGroupId && group);
  groupCallInvite.hidden = !active;
  groupCallCount.hidden = !active;
  if (groupCallStart && peerId) {
    const selectedGroup = groups.find((item) => item.user_id === peerId && item.is_group);
    groupCallStart.hidden = !selectedGroup;
    groupCallStart.textContent = selectedGroup
      ? (activeGroupCalls.get(selectedGroup.user_id)?.active ? "Join group call" : "Group call")
      : "Group call";
  }
  if (active) {
    groupCallCount.textContent = groupCallJoinedMembers.size + "/" + group.group_member_ids.length + " joined";
  }
}
function renderGroupCallBanner() {
  const group = groups.find((item) => item.user_id === peerId && item.is_group);
  const state = group ? activeGroupCalls.get(group.user_id) : null;
  const active = Boolean(group && state?.active);
  if (!groupCallBanner) return;
  groupCallBanner.hidden = !active;
  if (!active) {
    if (groupCallStart) groupCallStart.hidden = !group;
    return;
  }
  const joined = state.participant_ids.includes(me?.user_id);
  groupCallBannerTitle.textContent = state.media === "video" ? "Group video call is active" : "Group call is active";
  groupCallBannerMeta.textContent =
    " · " + state.participant_ids.length + "/" + group.group_member_ids.length + " joined";
  groupCallJoin.textContent = joined ? "Open call" : "Join";
  groupCallJoin.disabled = joined && groupCallGroupId !== group.user_id;
  groupCallJoin.hidden = groupCallId === state.call_id && groupCallGroupId === group.user_id;
  if (groupCallStart) {
    groupCallStart.hidden = false;
    groupCallStart.textContent = joined ? "Group call" : "Join group call";
  }
}

function handleGroupCallState(message) {
  if (!message?.group_id || !message?.call_id) return;
  if (message.active) {
    activeGroupCalls.set(message.group_id, message);
  } else {
    const current = activeGroupCalls.get(message.group_id);
    if (!current || current.call_id === message.call_id) activeGroupCalls.delete(message.group_id);
    if (groupCallId === message.call_id && groupCallGroupId === message.group_id) {
      endCall(false);
    }
  }
  refreshGroupCallParticipants();
  renderGroupCallBanner();
  if (peerId === message.group_id) {
    const group = groups.find((item) => item.user_id === peerId);
    if (group) renderGroupMembersDialog(group);
  }
}

async function joinActiveGroupCall() {
  const group = groups.find((item) => item.user_id === peerId && item.is_group);
  const state = group ? activeGroupCalls.get(group.user_id) : null;
  if (!state?.active) return;
  if (groupCallId === state.call_id && groupCallGroupId === group.user_id) {
    callStage.hidden = false;
    return;
  }
  if (peerConnection || groupPeerConnections.size || groupCallId) endCall(true);
  pendingIncomingCall = {
    sender_id: state.initiator_id,
    peer_id: state.group_id,
    payload: {
      group_id: state.group_id,
      call_id: state.call_id,
      media: state.media,
    },
  };
  await acceptGroupInvite(pendingIncomingCall);
  pendingIncomingCall = null;
}

function sendGroupCallControl(kind) {
  if (!groupCallGroupId || !groupCallId || !socket || socket.readyState !== WebSocket.OPEN) return;
  socket.send(JSON.stringify({
    type: "call_signal",