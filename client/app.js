import initCrypto, { CryptoDevice } from "/crypto-pkg/larptrix_crypto_wasm.js";
import {
  LarptrixMatrixCrypto,
  getOrCreateMatrixDeviceId,
} from "/matrix-crypto.js";

const cryptoWasmReady = initCrypto();

const statusEl = document.getElementById("status");
const usersEl = document.getElementById("users");
const logEl = document.getElementById("log");
const chatEl = document.querySelector("main.chat");
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
const imageViewerVideo = document.getElementById("image-viewer-video");
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
const profileTags = document.getElementById("profile-tags");
const profileTagSelect = document.getElementById("profile-tag-select");
const profileTagNew = document.getElementById("profile-tag-new");
const profileTagAdd = document.getElementById("profile-tag-add");
const profileMicrophone = document.getElementById("profile-microphone");
const profileSpeakers = document.getElementById("profile-speakers");
const profileCamera = document.getElementById("profile-camera");
const profileDevicesStatus = document.getElementById("profile-devices-status");
const profileEmail = document.getElementById("profile-email");
const profileEmailLabel = document.getElementById("profile-email-label");
const profileCustomActivities = document.getElementById("profile-custom-activities");
const profileNewActivity = document.getElementById("profile-new-activity");
const profileAddActivity = document.getElementById("profile-add-activity");
const showMusicActivity = document.getElementById("show-music-activity");
const forgetE2eDeviceButton = document.getElementById("forget-e2e-device");
const resetE2eKeysButton = document.getElementById("reset-e2e-keys");
const resetE2eHelp = document.getElementById("reset-e2e-help");
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
const recordVideoButton = document.getElementById("record-video");
const videoRecording = document.getElementById("video-recording");
const videoRecordingPreview = document.getElementById("video-recording-preview");
const videoRecordingTime = document.getElementById("video-recording-time");
const videoRecordingShape = document.getElementById("video-recording-shape");
const videoRecordingCancel = document.getElementById("video-recording-cancel");
const videoRecordingStatus = document.getElementById("video-recording-status");
const emptyEl = document.getElementById("empty");
const peerName = document.getElementById("peer-name");
const peerMeta = document.getElementById("peer-meta");
const peerVerified = document.getElementById("peer-verified");
const chatTitlebar = document.getElementById("chat-titlebar");
const callStage = document.getElementById("call-stage");
const callStatus = document.getElementById("call-status");
const localVideo = document.getElementById("local-video");
const localScreenVideo = document.getElementById("local-screen-video");
const directCallParticipants = document.getElementById("direct-call-participants");
const directCallJoin = document.getElementById("direct-call-join");
const directCallTopbarAvatar = document.getElementById("direct-call-topbar-avatar");
const directCallTopbarLabel = document.getElementById("direct-call-topbar-label");
const remoteVideo = document.getElementById("remote-video");
const remoteAudio = document.getElementById("remote-audio");
const callAudioPlaceholder = document.getElementById("call-audio-placeholder");
const callPlaceholderLocalAvatar = document.getElementById("call-placeholder-local-avatar");
const callPlaceholderRemoteAvatar = document.getElementById("call-placeholder-remote-avatar");
const callPlaceholderLocalName = document.getElementById("call-placeholder-local-name");
const callPlaceholderRemoteName = document.getElementById("call-placeholder-remote-name");
const enableCallAudio = document.getElementById("enable-call-audio");
const incomingCallDialog = document.getElementById("incoming-call-dialog");
const incomingCallTitle = document.getElementById("incoming-call-title");
const incomingCallKind = document.getElementById("incoming-call-kind");
const screenResolution = document.getElementById("screen-resolution");
const screenFrameRate = document.getElementById("screen-framerate");
const chatBackgroundInput = document.getElementById("chat-background");
const peerProfileDialog = document.getElementById("peer-profile-dialog");
const createGroupDialog = document.getElementById("create-group-dialog");
const createChannelDialog = document.getElementById("create-channel-dialog");
const groupMemberList = document.getElementById("group-member-list");
const channelMemberList = document.getElementById("channel-member-list");
const userSearchInput = document.getElementById("user-search");
const chatViewOpen = document.getElementById("chat-view-open");
const friendRequestsOpen = document.getElementById("friend-requests-open");
const friendRequestsBadge = document.getElementById("friend-requests-badge");
const peopleSearchHint = document.querySelector(".friends-search-hint");
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
const settingsWallpaperTint = document.getElementById("settings-wallpaper-tint");
const settingsAvatarShape = document.getElementById("settings-avatar-shape");
const customThemeEditor = document.getElementById("custom-theme-editor");
const themeBg = document.getElementById("theme-bg");
const themePanel = document.getElementById("theme-panel");
const themeText = document.getElementById("theme-text");
const themeMuted = document.getElementById("theme-muted");
const themeAccent = document.getElementById("theme-accent");
const themeMe = document.getElementById("theme-me");
const themeSaveCustom = document.getElementById("theme-save-custom");
const themeSaveName = document.getElementById("theme-save-name");
const savedThemeSelect = document.getElementById("saved-theme-select");
const themeLoadSaved = document.getElementById("theme-load-saved");
const themeDeleteSaved = document.getElementById("theme-delete-saved");
const settingsPresence = document.getElementById("settings-presence");
const settingsCallSounds = document.getElementById("settings-call-sounds");
const settingsOutgoingCallSounds = document.getElementById("settings-outgoing-call-sounds");
const settingsIncomingCallVolume = document.getElementById("settings-incoming-call-volume");
const settingsOutgoingCallVolume = document.getElementById("settings-outgoing-call-volume");
const settingsMessageSounds = document.getElementById("settings-message-sounds");
const settingsMessagePolicy = document.getElementById("settings-message-policy");
const settingsCallRingtone = document.getElementById("settings-call-ringtone");
const settingsCallRingtoneReset = document.getElementById("settings-call-ringtone-reset");
const settingsCallRingtoneStatus = document.getElementById("settings-call-ringtone-status");
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
const groupCallBannerAvatars = document.getElementById("group-call-banner-avatars");
const directCallAvatarStack = document.getElementById("direct-call-avatar-stack");
const groupCallJoin = document.getElementById("group-call-join");
const endCallButton = document.getElementById("end-call");
const endGroupCallButton = document.getElementById("end-group-call");
const callWindowTitle = document.getElementById("call-window-title");
const callRingtone = document.getElementById("call-ringtone");
const voiceRecording = document.getElementById("voice-recording");
const voiceRecordingTime = document.getElementById("voice-recording-time");
const voiceRecordingCancel = document.getElementById("voice-recording-cancel");
const voiceRecordingStatus = document.getElementById("voice-recording-status");
const CALL_RING_VOLUME_KEY = "larptrix_call_ring_volume";
const OUTGOING_RING_VOLUME_KEY = "larptrix_outgoing_ring_volume";
const SAVED_THEMES_KEY = "larptrix_saved_themes_v1";
const groupMembersOpen = document.getElementById("group-members-open");
const groupMembersDialog = document.getElementById("group-members-dialog");
const groupMembersClose = document.getElementById("group-members-close");
const groupMembersTitle = document.getElementById("group-members-title");
const groupMembersHelp = document.getElementById("group-members-help");
const groupMembersList = document.getElementById("group-members-list");
const groupProfileForm = document.getElementById("group-profile-form");
const groupProfileName = document.getElementById("group-profile-name");
const groupProfileDescription = document.getElementById("group-profile-description");
const groupProfileAvatar = document.getElementById("group-profile-avatar");
const groupProfileBanner = document.getElementById("group-profile-banner");
const groupProfileBannerLabel = document.getElementById("group-profile-banner-label");
const groupProfileError = document.getElementById("group-profile-error");
const groupCallInvite = document.getElementById("group-call-invite");
const groupCallCount = document.getElementById("group-call-count");
const callDeafenButton = document.getElementById("toggle-call-deafen");
const callSettingsOpen = document.getElementById("call-settings-open");
const callSettingsPanel = document.getElementById("call-settings-panel");
const callNoiseSuppression = document.getElementById("call-noise-suppression");
const callSettingsMicrophone = document.getElementById("call-settings-microphone");
const callSettingsSpeakers = document.getElementById("call-settings-speakers");
const callSettingsCamera = document.getElementById("call-settings-camera");
const callParticipantSettings = document.getElementById("call-participant-settings");
const callWindowPin = document.getElementById("call-window-pin");
const menuNewChannel = document.getElementById("menu-new-channel");
const menuAbout = document.getElementById("menu-about");
const aboutDialog = document.getElementById("about-dialog");
const profileBannerFile = document.getElementById("profile-banner-file");
const settingsBrowserNotifications = document.getElementById("settings-browser-notifications");
const settingsEnableNotifications = document.getElementById("settings-enable-notifications");
const settingsNotificationsStatus = document.getElementById("settings-notifications-status");
const settingsUpdateStatus = document.getElementById("settings-update-status");
const settingsCheckUpdates = document.getElementById("settings-check-updates");
const settingsInstallUpdate = document.getElementById("settings-install-update");
const settingsOpenReleases = document.getElementById("settings-open-releases");
const peerProfileBanner = document.getElementById("peer-profile-banner");
const profileBanner = document.querySelector("#profile-dialog .profile-banner");


let socket = null;
let socketHeartbeatTimer = null;
let me = null;
let peerId = null;
let users = [];
let searchResults = [];
let groups = [];
let searchRequestId = 0;
let friendRequests = { friends: [], incoming: [], outgoing: [] };
let peopleView = "chats";
let friendRequestsRequestId = 0;
let friendSearchLoading = false;
let reconnect = false;
let mode = "login";
let legacyLogin = false;
let registerWithPassword = false;
let pendingKeyUser = null;
let pendingAttachments = [];
let pendingGif = null;
let previewUrls = [];
let recorder = null;
let recordingStream = null;
let recordedChunks = [];
let videoRecorder = null;
let videoRecordingStream = null;
let videoRecordedChunks = [];
let videoRecordingStartedAt = 0;
let videoRecordingTimer = null;
let videoRecordingCancelled = false;
let videoMessageShape = localStorage.getItem("larptrix_video_message_shape") === "square" ? "square" : "circle";
let videoMessageFile = null;
const videoMessageShapeByFile = new WeakMap();
let cryptoDevice = null;
let cryptoDeviceBundle = null;
let cryptoStoredState = null;
let cryptoRecoveryKey = null;
let cryptoDialogMode = null;
let cryptoEnabled = false;
let cryptoReady = Promise.resolve();
let cryptoLoadResolve = null;
let customActivities = [];
let localMusicActivity = null;
let customCallRingtoneUrl = null;
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
const SAVED_MESSAGES_ID = "__larptrix_saved_messages__";
const AVATAR_SHAPE_KEY = "larptrix_avatar_shape";
let replyingToMessage = null;
const decryptedPayloadByMessageId = new Map();
const mutedRemoteUserIds = new Set();
const viewedChannelMessages = new Set();
const matrixDeviceCheckCache = new Map();
const channelViewObserver = typeof IntersectionObserver === "function"
  ? new IntersectionObserver((entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting || entry.intersectionRatio < 0.45) continue;
        const messageId = entry.target?.dataset?.messageId;
        if (!messageId || viewedChannelMessages.has(messageId) || !peerId) continue;
        const group = groups.find((item) => item.user_id === peerId && item.is_channel);
        if (!group || !socket || socket.readyState !== WebSocket.OPEN) continue;
        viewedChannelMessages.add(messageId);
        socket.send(JSON.stringify({
          type: "view_message",
          peer_id: peerId,
          message_id: messageId,
        }));
      }
    }, { threshold: [0.45] })
  : null;
let callDeafened = false;

function e2eResetStorageKey(peerUserId) {
  return me && peerUserId ? `larptrix_e2e_reset_${me.user_id}_${peerUserId}` : null;
}
function getE2eResetAt(peerUserId) {
  const key = e2eResetStorageKey(peerUserId);
  if (!key) return 0;
  const value = Number(localStorage.getItem(key));
  return Number.isFinite(value) && value > 0 ? value : 0;
}
function setE2eResetAt(peerUserId, timestamp) {
  const key = e2eResetStorageKey(peerUserId);
  if (!key) return;
  localStorage.setItem(key, String(timestamp));
}
function isMessageVisibleAfterE2eReset(message) {
  if (!message || !peerId || !me) return true;
  const resetAt = getE2eResetAt(peerId);
  return !resetAt || Number(message.created_at) > resetAt;
}
function renderE2eResetNotice(peerUserId) {
  if (!getE2eResetAt(peerUserId)) return;
  const li = document.createElement("li");
  li.className = "e2e-reset-notice";
  const title = document.createElement("strong");
  title.textContent = "You reset your encryption key";
  const detail = document.createElement("span");
  detail.textContent = "Older messages remain stored for the other participant, but this device starts a fresh encrypted history.";
  li.append(title, detail);
  logEl.append(li);
}
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

async function cacheDecryptedMessagePayload(messageId, payload) {
  try {
    if (!messageId || !payload) return;
    const key = await sentPlaintextCacheKey();
    if (!key) return;
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: "AES-GCM", iv },
      key,
      new TextEncoder().encode(JSON.stringify(payload)),
    );
    await writeLocalCryptoRecord({
      id: `decrypted-message:${messageId}`,
      iv: Array.from(iv),
      ciphertext: Array.from(new Uint8Array(encrypted)),
    });
  } catch {}
}

async function loadCachedDecryptedMessagePayload(messageId) {
  try {
    if (!messageId) return null;
    const key = await sentPlaintextCacheKey();
    if (!key) return null;
    const record = await readLocalCryptoRecord(`decrypted-message:${messageId}`);
    if (!record?.iv || !record?.ciphertext) return null;
    const plaintext = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: new Uint8Array(record.iv) },
      key,
      new Uint8Array(record.ciphertext),
    );
    const payload = JSON.parse(new TextDecoder().decode(plaintext));
    return payload && typeof payload === "object" ? payload : null;
  } catch {
    return null;
  }
}

function renderCachedDecryptedPayload(message, bodyElement, payload) {
  if (!message || !bodyElement || !payload) return;
  if (!messageBodyElementsById.has(message.id)) return;
  decryptedPayloadByMessageId.set(message.id, payload);
  message._decryptedPayload = payload;
  renderMessageDecorations(bodyElement.parentElement, payload);
  bodyElement.textContent =
    typeof payload.text === "string" ? payload.text : JSON.stringify(payload);
  void renderEncryptedAttachments(message, payload, bodyElement.parentElement).catch((err) => {
    console.error("[E2E] cached attachment render failed", err);
  });
  bodyElement.classList.add("decrypted-cached");
}

let peerConnection = null;
let localMediaStream = null;
let screenMediaStream = null;
let pendingIncomingCall = null;
let callPeerId = null;
let callMediaKind = null;
let directCallSessionId = null;
let directCallOutgoing = false;
let directCallStartedAt = 0;
let directCallAnsweredAt = 0;
let directCallAnswered = false;
let directCallOfferSent = false;
const loggedDirectCallSessions = new Set();
let callMediaNotice = "";
let pendingIceCandidates = [];
let outgoingCallTimeout = null;
let incomingCallTimeout = null;
let noAnswerCleanupTimeout = null;
let lastDirectCallAvatarTimeout = null;
let lastDirectCallPeerId = null;
let lastDirectCallAvatarPeerId = null;
let lastDirectCallJoinPeerId = null;
let lastDirectCallJoinKind = "audio";
let directCallNoticeTimeout = null;
let callNoAnswer = false;
let voiceRecordingTimer = null;
let voiceRecordingStartedAt = 0;
const speakingMonitors = new Map();
const iceCandidatesBeforeOffer = new Map();
const staleDirectCallSessions = new Map();

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
const OUTGOING_CALL_SOUND_KEY = "larptrix_outgoing_call_sounds";
const MESSAGE_SOUND_KEY = "larptrix_message_sounds";
const NOISE_SUPPRESSION_KEY = "larptrix_noise_suppression";
const CALL_MIC_KEY = "larptrix_call_microphone";
const CALL_CAMERA_KEY = "larptrix_call_camera";
const CALL_SPEAKERS_KEY = "larptrix_call_speakers";
const SAVED_MESSAGES_KEY = "larptrix_saved_messages_v1";
const SAVED_MESSAGES_LOCAL_KEY_ID = "saved-messages-local-key";
function savedMessagesKey() {
  return me ? SAVED_MESSAGES_KEY + "_" + me.user_id : SAVED_MESSAGES_KEY;
}

const TAG_LIBRARY_KEY = "larptrix_tag_library_v1";

function tagLibraryKey() {
  return me?.user_id ? TAG_LIBRARY_KEY + "_" + me.user_id : TAG_LIBRARY_KEY;
}

function normalizeProfileTag(raw) {
  return String(raw || "").trim().replace(/\s+/g, " ");
}

function readProfileTagLibrary(seed = []) {
  const merged = [];
  let stored = [];
  try {
    const parsed = JSON.parse(localStorage.getItem(tagLibraryKey()) || "[]");
    stored = Array.isArray(parsed) ? parsed : [];
  } catch {}

  for (const raw of [...stored, ...(Array.isArray(seed) ? seed : [])]) {
    const tag = normalizeProfileTag(raw);
    if (!tag || tag.length > 24) continue;
    if (!merged.some((item) => item.localeCompare(tag, undefined, { sensitivity: "accent" }) === 0)) {
      merged.push(tag);
    }
  }

  const trimmed = merged.slice(0, 20);
  try {
    localStorage.setItem(tagLibraryKey(), JSON.stringify(trimmed));
  } catch {}
  return trimmed;
}

function writeProfileTagLibrary(tags) {
  const normalized = [];
  for (const raw of Array.isArray(tags) ? tags : []) {
    const tag = normalizeProfileTag(raw);
    if (!tag || tag.length > 24) continue;
    if (!normalized.some((item) => item.localeCompare(tag, undefined, { sensitivity: "accent" }) === 0)) {
      normalized.push(tag);
    }
  }
  try {
    localStorage.setItem(tagLibraryKey(), JSON.stringify(normalized.slice(0, 20)));
  } catch {}
}

function renderProfileTagLibrary(seed = []) {
  if (!profileTagSelect) return;
  const library = readProfileTagLibrary(seed);
  const active = normalizeProfileTag(profileTags?.value || "");
  profileTagSelect.replaceChildren();

  const none = document.createElement("option");
  none.value = "";
  none.textContent = "No active tag";
  profileTagSelect.append(none);

  for (const tag of library) {
    const option = document.createElement("option");
    option.value = tag;
    option.textContent = "🏷️ " + tag;
    profileTagSelect.append(option);
  }

  profileTagSelect.value = library.includes(active) ? active : "";
  if (profileTags) profileTags.value = profileTagSelect.value;
}

function addProfileTag() {
  const tag = normalizeProfileTag(profileTagNew?.value || "");
  if (!tag) return;
  if (tag.length > 24) {
    profileError.textContent = "Tags must be 24 characters or fewer.";
    profileError.hidden = false;
    return;
  }
  const library = readProfileTagLibrary();
  if (!library.some((item) => item.localeCompare(tag, undefined, { sensitivity: "accent" }) === 0)) {
    library.unshift(tag);
    writeProfileTagLibrary(library);
  }
  renderProfileTagLibrary();
  profileTagSelect.value = library.find((item) => item.localeCompare(tag, undefined, { sensitivity: "accent" }) === 0) || tag;
  if (profileTags) profileTags.value = profileTagSelect.value;
  if (profileTagNew) profileTagNew.value = "";
  profileError.hidden = true;
}

async function savedMessagesLocalKey() {
  let key = (await readLocalCryptoRecord(SAVED_MESSAGES_LOCAL_KEY_ID))?.value;
  if (!key) {
    key = await crypto.subtle.generateKey(
      { name: "AES-GCM", length: 256 },
      false,
      ["encrypt", "decrypt"],
    );
    await writeLocalCryptoRecord({ id: SAVED_MESSAGES_LOCAL_KEY_ID, value: key });
  }
  return key;
}

async function readLegacySavedMessages() {
  if (!cryptoRecoveryKey || !me) return null;
  try {
    const raw = localStorage.getItem(savedMessagesKey());
    if (!raw) return null;
    const record = JSON.parse(raw);
    if (!record?.iv || !record?.ciphertext) return null;
    const key = await sentPlaintextCacheKey();
    if (!key) return null;
    const plaintext = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: new Uint8Array(record.iv) },
      key,
      new Uint8Array(record.ciphertext),
    );
    const parsed = JSON.parse(new TextDecoder().decode(plaintext));
    return Array.isArray(parsed) ? parsed : null;
  } catch {
    return null;
  }
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
  if (peerId) applyChatWallpaper(peerId);
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

function currentCustomTheme() {
  return {
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
}

function readSavedThemes() {
  try {
    const parsed = JSON.parse(localStorage.getItem(SAVED_THEMES_KEY) || "[]");
    return Array.isArray(parsed) ? parsed.filter((item) => item?.id && item?.name && item?.theme) : [];
  } catch {
    return [];
  }
}

function renderSavedThemeOptions() {
  if (!savedThemeSelect) return;
  savedThemeSelect.replaceChildren();
  const first = document.createElement("option");
  first.value = "";
  first.textContent = "Saved themes…";
  savedThemeSelect.append(first);
  for (const theme of readSavedThemes()) {
    const option = document.createElement("option");
    option.value = theme.id;
    option.textContent = theme.name;
    savedThemeSelect.append(option);
  }
  const disabled = savedThemeSelect.options.length <= 1;
  savedThemeSelect.disabled = disabled;
  if (themeLoadSaved) themeLoadSaved.disabled = disabled;
  if (themeDeleteSaved) themeDeleteSaved.disabled = disabled;
}

function loadSavedTheme(id) {
  const theme = readSavedThemes().find((item) => item.id === id);
  if (!theme) return;
  localStorage.setItem(CUSTOM_THEME_KEY, JSON.stringify(theme.theme));
  applyTheme("custom");
  settingsTheme.value = "custom";
  loadThemeEditor();
}

function saveCustomTheme() {
  const custom = currentCustomTheme();
  localStorage.setItem(CUSTOM_THEME_KEY, JSON.stringify(custom));
  localStorage.setItem(THEME_KEY, "custom");
  applyTheme("custom");
  settingsTheme.value = "custom";

  const name = (themeSaveName?.value || "My custom theme").trim().slice(0, 40);
  const themes = readSavedThemes();
  const existing = themes.find((item) => item.name.toLocaleLowerCase() === name.toLocaleLowerCase());
  if (existing) {
    existing.theme = custom;
    existing.updatedAt = Date.now();
  } else {
    themes.unshift({ id: crypto.randomUUID(), name, theme: custom, updatedAt: Date.now() });
  }
  localStorage.setItem(SAVED_THEMES_KEY, JSON.stringify(themes.slice(0, 20)));
  if (themeSaveName) themeSaveName.value = "";
  renderSavedThemeOptions();
  if (savedThemeSelect) savedThemeSelect.value = themes[0]?.id || "";
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

function getTauriNotificationApi() {
  return globalThis.__TAURI__?.notification || null;
}

function canUseBrowserNotifications() {
  const native = getTauriNotificationApi();
  return Boolean(native?.sendNotification || typeof Notification === "function");
}

function browserNotificationsEnabled() {
  return localStorage.getItem("larptrix_browser_notifications") === "1";
}

async function requestBrowserNotifications() {
  const native = getTauriNotificationApi();
  if (native?.isPermissionGranted && native?.requestPermission) {
    let granted = await native.isPermissionGranted();
    if (!granted) {
      granted = (await native.requestPermission()) === "granted";
    }
    localStorage.setItem("larptrix_browser_notifications", granted ? "1" : "0");
    renderNotificationSettings();
    return granted;
  }
  if (typeof Notification !== "function") {
    throw new Error("This app does not support notifications on this platform.");
  }
  const permission = await Notification.requestPermission();
  const enabled = permission === "granted";
  localStorage.setItem("larptrix_browser_notifications", enabled ? "1" : "0");
  renderNotificationSettings();
  return enabled;
}

function renderNotificationSettings() {
  if (!settingsBrowserNotifications || !settingsNotificationsStatus) return;
  const native = getTauriNotificationApi();
  const supported = Boolean(native?.sendNotification || typeof Notification === "function");
  settingsBrowserNotifications.disabled = !supported;
  settingsBrowserNotifications.checked = supported && browserNotificationsEnabled();
  if (!supported) {
    settingsNotificationsStatus.textContent = "Notifications are not supported by this app build.";
    return;
  }
  if (native?.sendNotification) {
    settingsNotificationsStatus.textContent = browserNotificationsEnabled()
      ? "Native app notifications are enabled."
      : "Native app notifications are available. Allow them to receive messages and calls.";
    return;
  }
  settingsNotificationsStatus.textContent =
    Notification.permission === "granted" ? "Notifications are allowed." :
    Notification.permission === "denied" ? "Notifications are blocked by the browser or app." :
    "Permission has not been requested yet.";
}

async function showBrowserNotification(title, body, tag, iconUrl = null) {
  if (!browserNotificationsEnabled()) return;
  if (document.visibilityState === "visible" && document.hasFocus()) return;
  const native = getTauriNotificationApi();
  if (native?.sendNotification) {
    try {
      native.sendNotification({
        title,
        body,
        ...(iconUrl ? { icon: iconUrl } : {}),
      });
    } catch {}
    return;
  }
  if (typeof Notification !== "function" || Notification.permission !== "granted") return;
  try {
    const notification = new Notification(title, {
      body,
      tag,
      ...(iconUrl ? { icon: iconUrl } : {}),
    });
    notification.onclick = () => {
      try {
        window.focus();
        const peerFromTag = typeof tag === "string" && tag.startsWith("message-")
          ? tag.slice("message-".length)
          : null;
        if (peerFromTag) openChat(peerFromTag);
      } catch {}
      notification.close?.();
    };
  } catch {}
}

async function loadCustomCallRingtone() {
  if (!callRingtone) return;
  try {
    const record = await readLocalCryptoRecord("call-ringtone");
    if (!record?.blob) {
      if (settingsCallRingtoneStatus) settingsCallRingtoneStatus.textContent = "Default ringtone";
      return;
    }
    if (customCallRingtoneUrl) URL.revokeObjectURL(customCallRingtoneUrl);
    customCallRingtoneUrl = URL.createObjectURL(record.blob);
    callRingtone.src = customCallRingtoneUrl;
    callRingtone.load();
    if (settingsCallRingtoneStatus) {
      settingsCallRingtoneStatus.textContent = record.name
        ? "Custom: " + record.name
        : "Custom ringtone";
    }
  } catch {
    if (settingsCallRingtoneStatus) settingsCallRingtoneStatus.textContent = "Default ringtone";
  }
}

async function setCustomCallRingtone(file) {
  if (!file || !file.type.startsWith("audio/")) throw new Error("Choose an audio file.");
  if (file.size > 15 * 1024 * 1024) throw new Error("Ringtone is too large (maximum 15 MB).");
  await writeLocalCryptoRecord({ id: "call-ringtone", blob: file, name: file.name, type: file.type });
  await loadCustomCallRingtone();
}

async function resetCustomCallRingtone() {
  const db = await openLocalCryptoDb();
  try {
    await new Promise((resolve, reject) => {
      const tx = db.transaction("items", "readwrite");
      tx.objectStore("items").delete("call-ringtone");
      tx.addEventListener("complete", resolve, { once: true });
      tx.addEventListener("error", () => reject(tx.error), { once: true });
      tx.addEventListener("abort", () => reject(tx.error), { once: true });
    });
  } finally {
    db.close();
  }
  if (customCallRingtoneUrl) URL.revokeObjectURL(customCallRingtoneUrl);
  customCallRingtoneUrl = null;
  const source = callRingtone?.querySelector("source");
  if (callRingtone && source) {
    callRingtone.removeAttribute("src");
    source.setAttribute("src", "/assets/02439.mp3");
    callRingtone.load();
  }
  if (settingsCallRingtoneStatus) settingsCallRingtoneStatus.textContent = "Default ringtone";
}
function startCallRingtone() {
  if (!readStoredBool(CALL_SOUND_KEY, true) || localStorage.getItem(PRESENCE_KEY) === "dnd") return;
  if (!callRingtone) return;
  callRingtone.volume = Math.min(1, Math.max(0, Number(localStorage.getItem(CALL_RING_VOLUME_KEY) ?? "0.8")));
  callRingtone.currentTime = 0;
  callRingtone.loop = true;
  callRingtone.play().catch(() => {});
}

function startOutgoingCallRingtone() {
  if (!readStoredBool(OUTGOING_CALL_SOUND_KEY, true)) return;
  if (!callRingtone) return;
  callRingtone.volume = Math.min(1, Math.max(0, Number(localStorage.getItem(OUTGOING_RING_VOLUME_KEY) ?? "0.8")));
  callRingtone.currentTime = 0;
  callRingtone.loop = true;
  callRingtone.play().catch(() => {});
}

function stopCallRingtone() {
  if (!callRingtone) return;
  callRingtone.pause();
  callRingtone.currentTime = 0;
}

function clampWindowPosition(element, x, y) {
  const rect = element.getBoundingClientRect();
  return {
    x: Math.min(Math.max(8, x), Math.max(8, window.innerWidth - rect.width - 8)),
    y: Math.min(Math.max(8, y), Math.max(8, window.innerHeight - rect.height - 8)),
  };
}

function applySavedCallPosition() {
  if (!callStage) return;
  let saved = null;
  try { saved = JSON.parse(localStorage.getItem("larptrix_call_window_position") || "null"); } catch {}
  if (!saved || !Number.isFinite(saved.x) || !Number.isFinite(saved.y)) return;
  const p = clampWindowPosition(callStage, saved.x, saved.y);
  callStage.style.left = p.x + "px";
  callStage.style.top = p.y + "px";
  callStage.style.right = "auto";
  callStage.style.bottom = "auto";
}

function updateCallPlaceholder({ force = false } = {}) {
  if (!callAudioPlaceholder) return;
  const remoteId = callPeerId || groupCallGroupId;
  const remoteUser = groups.find((item) => item.user_id === remoteId)
    || users.find((item) => item.user_id === remoteId)
    || (pendingIncomingCall?.sender_id
      ? users.find((item) => item.user_id === pendingIncomingCall.sender_id)
      : null)
    || { user_id: remoteId || "remote", display_name: "Larptrix user" };

  callPlaceholderLocalName.textContent = me?.display_name || "You";
  callPlaceholderRemoteName.textContent = remoteUser.display_name || "Larptrix user";
  paintAvatar(callPlaceholderLocalAvatar, me || { user_id: "local", display_name: "You" });
  paintAvatar(callPlaceholderRemoteAvatar, remoteUser);
  callPlaceholderRemoteName.classList.remove("call-no-answer");
  renderDirectCallParticipants();

  const hasRemoteVideo = Boolean(
    remoteVideo?.srcObject instanceof MediaStream
      && remoteVideo.srcObject.getVideoTracks().some((track) => track.readyState !== "ended"),
  );
  const hasLocalVideo = Boolean(
    localMediaStream?.getVideoTracks().some((track) => track.readyState !== "ended" && track.enabled),
  );
  callAudioPlaceholder.hidden = !force && (hasRemoteVideo || hasLocalVideo || Boolean(screenMediaStream));
}

function showCallStage() {
  callStage.hidden = false;
  callStage.classList.remove("call-ending-notice");
  applySavedCallPosition();
  void populateCallDeviceSelects();
  updateCallPlaceholder({ force: !localMediaStream?.getVideoTracks().length });
  renderDirectCallParticipants();
  renderCallParticipantSettings();
}

function wireCallWindowDragging() {
  const handle = document.querySelector(".call-dock-heading");
  if (!handle || !callStage) return;
  let dragging = false;
  let offsetX = 0;
  let offsetY = 0;
  handle.addEventListener("pointerdown", (event) => {
    if (event.button !== 0 || event.target.closest("button,select,input,a")) return;
    const rect = callStage.getBoundingClientRect();
    dragging = true;
    offsetX = event.clientX - rect.left;
    offsetY = event.clientY - rect.top;
    handle.setPointerCapture?.(event.pointerId);
    event.preventDefault();
  });
  handle.addEventListener("pointermove", (event) => {
    if (!dragging) return;
    const p = clampWindowPosition(callStage, event.clientX - offsetX, event.clientY - offsetY);
    callStage.style.left = p.x + "px";
    callStage.style.top = p.y + "px";
    callStage.style.right = "auto";
    callStage.style.bottom = "auto";
  });
  const stop = (event) => {
    if (!dragging) return;
    dragging = false;
    handle.releasePointerCapture?.(event.pointerId);
    const rect = callStage.getBoundingClientRect();
    localStorage.setItem("larptrix_call_window_position", JSON.stringify({ x: rect.left, y: rect.top }));
  };
  handle.addEventListener("pointerup", stop);
  handle.addEventListener("pointercancel", stop);
}

function wireCallResponsiveSizing() {
  if (!callStage || typeof ResizeObserver !== "function") return;
  const update = () => {
    const width = callStage.getBoundingClientRect().width;
    callStage.classList.toggle("call-compact", width <= 620);
    callStage.classList.toggle("call-ultra-compact", width <= 460);
  };
  const observer = new ResizeObserver(update);
  observer.observe(callStage);
  update();
}

function setCallPinned(pinned) {
  callStage.classList.toggle("window-pinned", pinned);
  callWindowPin?.setAttribute("aria-pressed", String(pinned));
  if (callWindowPin) callWindowPin.textContent = pinned ? "📍" : "📌";
  localStorage.setItem("larptrix_call_window_pinned", pinned ? "1" : "0");
}

function renderCallParticipantSettings() {
  if (!callParticipantSettings) return;
  callParticipantSettings.replaceChildren();
  const ids = groupCallId
    ? [...groupCallJoinedMembers].filter((id) => id !== me?.user_id)
    : callPeerId && callPeerId !== me?.user_id ? [callPeerId] : [];
  if (!ids.length) {
    const p = document.createElement("p");
    p.className = "settings-help";
    p.textContent = "No remote participants.";
    callParticipantSettings.append(p);
    return;
  }
  for (const id of ids) {
    const row = document.createElement("div");
    row.className = "call-participant-setting";
    const label = document.createElement("span");
    label.textContent = users.find((item) => item.user_id === id)?.display_name || "Participant";
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "ghost";
    const muted = mutedRemoteUserIds.has(id);
    btn.textContent = muted ? "Unmute" : "Mute";
    btn.setAttribute("aria-pressed", String(muted));
    btn.addEventListener("click", () => toggleRemoteUserMuted(id));
    row.append(label, btn);
    callParticipantSettings.append(row);
  }
}

function applyRemoteMuteStates() {
  if (remoteAudio) remoteAudio.muted = callDeafened || mutedRemoteUserIds.has(callPeerId);
  document.querySelectorAll("#group-remotes audio[data-group-remote-id]").forEach((audio) => {
    audio.muted = callDeafened || mutedRemoteUserIds.has(audio.dataset.groupRemoteId);
  });
}

function toggleRemoteUserMuted(userId) {
  if (!userId) return;
  if (mutedRemoteUserIds.has(userId)) mutedRemoteUserIds.delete(userId);
  else mutedRemoteUserIds.add(userId);
  applyRemoteMuteStates();
  renderCallParticipantSettings();
}

function toggleCallDeafen() {
  callDeafened = !callDeafened;
  if (callDeafenButton) {
    callDeafenButton.textContent = callDeafened ? "🔇 Sound off" : "🔊 Deafen";
    callDeafenButton.setAttribute("aria-pressed", String(callDeafened));
  }
  applyRemoteMuteStates();
}

async function replaceCallMicrophoneTrack() {
  const oldTrack = localMediaStream?.getAudioTracks()[0];
  if (!oldTrack || !navigator.mediaDevices?.getUserMedia) return;
  const baseAudio = callAudioConstraints();
  const micId = selectedCallDeviceId("audioinput");
  const audio = micId ? { ...baseAudio, deviceId: { exact: micId } } : baseAudio;
  const stream = await navigator.mediaDevices.getUserMedia({
    audio,
    video: false,
  });
  const newTrack = stream.getAudioTracks()[0];
  if (!newTrack) {
    stream.getTracks().forEach((track) => track.stop());
    throw new Error("Microphone unavailable.");
  }
  newTrack.enabled = oldTrack.enabled;
  const connections = groupCallId ? [...groupPeerConnections.values()] : peerConnection ? [peerConnection] : [];
  for (const connection of connections) {
    const sender = connection.getSenders().find((item) => item.track?.kind === "audio");
    if (sender) await sender.replaceTrack(newTrack);
  }
  oldTrack.stop();
  localMediaStream.removeTrack(oldTrack);
  localMediaStream.addTrack(newTrack);
}

async function toggleCallNoiseSuppression() {
  const enabled = Boolean(callNoiseSuppression?.checked);
  localStorage.setItem(NOISE_SUPPRESSION_KEY, enabled ? "1" : "0");
  try {
    await replaceCallMicrophoneTrack();
    callStatus.textContent = enabled ? "Noise suppression enabled" : "Noise suppression disabled";
  } catch (err) {
    appendSystem("Could not change noise suppression: " + (err.message || err));
    if (callNoiseSuppression) callNoiseSuppression.checked = !enabled;
    localStorage.setItem(NOISE_SUPPRESSION_KEY, enabled ? "0" : "1");
  }
}

async function getSavedMessages() {
  if (!me?.user_id) return [];
  try {
    const localRecord = await readLocalCryptoRecord(`saved-messages:${me.user_id}`);
    if (localRecord?.iv && localRecord?.ciphertext) {
      const key = await savedMessagesLocalKey();
      const plaintext = await crypto.subtle.decrypt(
        { name: "AES-GCM", iv: new Uint8Array(localRecord.iv) },
        key,
        new Uint8Array(localRecord.ciphertext),
      );
      const parsed = JSON.parse(new TextDecoder().decode(plaintext));
      return Array.isArray(parsed) ? parsed : [];
    }

    const legacy = await readLegacySavedMessages();
    if (legacy) {
      await setSavedMessages(legacy);
      return legacy;
    }
  } catch {}
  return [];
}

async function setSavedMessages(items) {
  if (!me?.user_id) return;
  const key = await savedMessagesLocalKey();
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    new TextEncoder().encode(JSON.stringify(items)),
  );
  await writeLocalCryptoRecord({
    id: `saved-messages:${me.user_id}`,
    iv: Array.from(iv),
    ciphertext: Array.from(new Uint8Array(encrypted)),
  });

  // Keep the old encrypted localStorage copy for one release so existing
  // installations can recover it during migration.
  if (cryptoRecoveryKey) {
    const legacyKey = await sentPlaintextCacheKey();
    if (legacyKey) {
      const legacyIv = crypto.getRandomValues(new Uint8Array(12));
      const legacyEncrypted = await crypto.subtle.encrypt(
        { name: "AES-GCM", iv: legacyIv },
        legacyKey,
        new TextEncoder().encode(JSON.stringify(items)),
      );
      localStorage.setItem(savedMessagesKey(), JSON.stringify({
        iv: Array.from(legacyIv),
        ciphertext: Array.from(new Uint8Array(legacyEncrypted)),
      }));
    }
  }
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

function savedChatEntry() {
  return {
    user_id: SAVED_MESSAGES_ID,
    display_name: "Saved Messages",
    username: "saved",
    online: true,
    is_saved_chat: true,
    e2e_enabled: true,
  };
}

function getChatEntries() {
  const entries = [savedChatEntry(), ...users, ...groups, ...searchResults];
  const unique = new Map();
  for (const entry of entries) {
    if (entry?.user_id && !unique.has(entry.user_id)) unique.set(entry.user_id, entry);
  }
  return [...unique.values()];
}

function clearReplyComposer() {
  replyingToMessage = null;
  const preview = document.getElementById("reply-preview");
  if (preview) {
    preview.hidden = true;
    preview.replaceChildren();
  }
}

function setReplyComposer(message) {
  replyingToMessage = {
    id: message.id,
    sender_id: message.sender_id,
    sender_name: message.sender_name || "Unknown",
    text: message._decryptedPayload?.text || message.text || "Encrypted message",
  };
  const preview = document.getElementById("reply-preview");
  const text = document.getElementById("reply-preview-text");
  if (!preview || !text) return;
  text.textContent = replyingToMessage.sender_name + ": " + replyingToMessage.text;
  preview.hidden = false;
  document.getElementById("reply-preview-close")?.addEventListener("click", clearReplyComposer, { once: true });
  bodyInput.focus();
}

function renderMessageDecorations(parent, payload) {
  parent.querySelectorAll(".message-context-preview").forEach((item) => item.remove());
  if (payload?.forwarded_from?.sender_name) {
    const block = document.createElement("div");
    block.className = "message-context-preview forwarded-preview";
    block.textContent = "↪ Forwarded from " + payload.forwarded_from.sender_name;
    parent.insertBefore(block, parent.firstChild);
  }
  if (payload?.reply_to?.sender_name) {
    const block = document.createElement("div");
    block.className = "message-context-preview reply-preview";
    const strong = document.createElement("strong");
    strong.textContent = "↩ " + payload.reply_to.sender_name;
    const quote = document.createElement("span");
    quote.textContent = payload.reply_to.text || "Message";
    block.append(strong, quote);
    parent.insertBefore(block, parent.firstChild);
  }
}

function renderSavedChatHistory() {
  if (peerId !== SAVED_MESSAGES_ID) return;
  logEl.replaceChildren();
  void getSavedMessages().then((items) => {
    if (peerId !== SAVED_MESSAGES_ID) return;
    items.forEach(appendSavedMessage);
    logEl.scrollTop = logEl.scrollHeight;
  });
}

function appendSavedMessage(item) {
  const li = document.createElement("li");
  li.dataset.messageId = item.id;
  li.classList.add("me");
  const savedAvatar = document.createElement("span");
  savedAvatar.className = "avatar message-avatar";
  paintAvatar(savedAvatar, me || { user_id: "saved", display_name: item.sender_name || "Saved" });

  const meta = document.createElement("div");
  meta.className = "meta";
  meta.textContent = (item.sender_name || "Saved") + " · " + new Date(item.created_at).toLocaleTimeString();
  const body = document.createElement("div");
  body.textContent = item.text || "";
  renderMessageDecorations(li, item);
  li.append(savedAvatar, meta, body);
  const actions = document.createElement("div");
  actions.className = "message-actions";

  const reply = document.createElement("button");
  reply.type = "button";
  reply.className = "ghost";
  reply.textContent = "Reply";
  reply.addEventListener("click", () => setReplyComposer({ ...item, _decryptedPayload: item }));
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "ghost";
  remove.textContent = "Remove";
  remove.addEventListener("click", async () => {
    const next = (await getSavedMessages()).filter((entry) => entry.id !== item.id);
    await setSavedMessages(next);
    renderSavedChatHistory();
  });
  actions.append(reply, remove);
  li.append(actions);
  logEl.append(li);
}

async function saveManualSavedMessage(text, extras = {}) {
  const items = await getSavedMessages();
  items.unshift({
    id: crypto.randomUUID(),
    peer_id: SAVED_MESSAGES_ID,
    sender_id: me.user_id,
    sender_name: me.display_name,
    text,
    created_at: Date.now(),
    ...extras,
  });
  await setSavedMessages(items.slice(0, 500));
  clearReplyComposer();
  bodyInput.value = "";
  renderSavedChatHistory();
}

function openSavedMessagesChat() {
  channelViewObserver?.disconnect();
  viewedChannelMessages.clear();
  peerId = SAVED_MESSAGES_ID;
  peerName.textContent = "Saved Messages";
  peerMeta.textContent = "";
  chatTitlebar.hidden = false;
  peerName.hidden = false;
  composer.hidden = false;
  emptyEl.hidden = true;
  document.getElementById("start-audio-call").hidden = true;
  document.getElementById("start-video-call").hidden = true;
  groupMembersOpen.hidden = true;
  groupCallStart.hidden = true;
  groupCallInvite.hidden = true;
  renderUsers();
  renderSavedChatHistory();
}

async function getMessagePlaintextPayload(message) {
  if (decryptedPayloadByMessageId.has(message.id)) return decryptedPayloadByMessageId.get(message.id);
  if (message._decryptedPayload) return message._decryptedPayload;
  const cached = await loadCachedSentPlaintext(message.body);
  const parsed = parseEncryptedPayload(cached);
  return parsed && typeof parsed === "object" ? parsed : null;
}

async function openForwardDialog(message) {
  const payload = await getMessagePlaintextPayload(message);
  const textValue = payload?.text || message.text || "";
  const hasForwardableContent = Boolean(
    textValue
      || payload?.file
      || (Array.isArray(payload?.files) && payload.files.length)
      || payload?.gif
      || payload?.voice
  );
  if (!hasForwardableContent) {
    appendSystem("Wait for this message to decrypt before forwarding.");
    return;
  }

  const dialog = document.getElementById("forward-message-dialog");
  const list = document.getElementById("forward-message-list");
  const search = document.getElementById("forward-message-search");
  const closeButton = document.getElementById("forward-message-close");
  if (!dialog || !list) return;

  const targets = getChatEntries().filter((item) => item.user_id !== me?.user_id);
  const forwardPayload = {
    ...payload,
    ...(textValue ? { text: textValue } : {}),
    forwarded_from: {
      sender_id: message.sender_id,
      sender_name: message.sender_name || "Larptrix user",
      message_id: message.id,
    },
  };
  delete forwardPayload.reply_to;
  if (forwardPayload.forwarded_from?.message_id === message.id) {
    forwardPayload.forwarded_from = {
      sender_id: message.sender_id,
      sender_name: message.sender_name || "Larptrix user",
      message_id: message.id,
    };
  }

  const renderTargets = () => {
    const query = normalizePeopleSearch(search?.value || "");
    list.replaceChildren();

    const matches = targets.filter((target) => {
      if (!query) return true;
      const name = String(target.display_name || "").toLocaleLowerCase();
      const username = String(target.username || "").toLocaleLowerCase();
      return name.includes(query) || username.includes(query);
    });

    if (!matches.length) {
      const empty = document.createElement("p");
      empty.className = "search-empty";
      empty.textContent = "No chats match your search.";
      list.append(empty);
      return;
    }

    for (const target of matches) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "forward-target";

      const avatar = document.createElement("span");
      avatar.className = "avatar";
      paintAvatar(
        avatar,
        target.is_saved_chat
          ? { user_id: "saved", display_name: "Saved Messages" }
          : target,
      );

      const copy = document.createElement("span");
      copy.className = "forward-target-copy";
      const title = document.createElement("strong");
      title.textContent = target.is_saved_chat ? "Saved Messages" : target.display_name;
      copy.append(title);

      if (!target.is_saved_chat && target.username) {
        const handle = document.createElement("small");
        handle.textContent = "@" + target.username;
        copy.append(handle);
      } else if (target.is_group) {
        const kind = document.createElement("small");
        kind.textContent = target.is_channel
          ? (target.subscriber_count || target.group_member_ids?.length || 0) + " subscribers"
          : (target.group_member_ids?.length || 0) + " members";
        copy.append(kind);
      }

      button.append(avatar, copy);
      button.addEventListener("click", async () => {
        button.disabled = true;
        try {
          if (target.is_saved_chat) {
            await saveManualSavedMessage(textValue, {
              ...forwardPayload,
              forwarded_from: { ...forwardPayload.forwarded_from },
            });
          } else {
            await sendEncryptedPayloadToPeer(target.user_id, forwardPayload);
          }
          dialog.close();
        } catch (err) {
          button.disabled = false;
          appendSystem("Forward failed: " + (err.message || err));
        }
      });
      list.append(button);
    }
  };

  search?.addEventListener("input", renderTargets);
  search?.removeAttribute("value");
  if (search) search.value = "";
  closeButton?.addEventListener("click", () => dialog.close(), { once: true });
  renderTargets();
  dialog.showModal();
}

async function sendEncryptedPayloadToPeer(targetId, payloadObject, files = []) {
  if (!cryptoEnabled || !cryptoDevice) throw new Error("Unlock E2E before sending messages.");
  const peer = getChatEntries().find((item) => item.user_id === targetId);
  if (!peer || peer.is_saved_chat) throw new Error("Invalid message target.");

  const fileList = Array.isArray(files) ? files.filter(Boolean) : (files ? [files] : []);
  const uploadedAttachments = [];
  for (const file of fileList) {
    const encrypted = await encryptAttachment(file);
    const uploaded = await uploadFile("/api/upload", encrypted.file);
    uploadedAttachments.push({
      ...encrypted.metadata,
      id: uploaded.id,
      url: uploaded.url || `/api/attachments/${encodeURIComponent(uploaded.id)}`,
    });
  }

  const rawPayload = JSON.stringify({
    ...payloadObject,
    ...(uploadedAttachments.length
      ? {
          files: uploadedAttachments,
          ...(uploadedAttachments.length === 1 ? { file: uploadedAttachments[0] } : {}),
        }
      : {}),
  });

  let encryptedBody;
  await withCryptoStateLock(async () => {
    await matrixCryptoReady;
    let matrixReady = Boolean(matrixCrypto);
    if (matrixReady) {
      try {
        await waitForMatrixDevices(peer.is_group ? peer.group_member_ids : [targetId]);
      } catch {
        matrixReady = false;
      }
    }
    if (peer.is_group) {
      if (!matrixReady) throw new Error("Group E2E requires Matrix crypto.");
      const roomId = matrixCrypto.groupRoomId(peer.user_id);
      await matrixCrypto.prepareRoom(roomId, peer.group_member_ids);
      encryptedBody = JSON.stringify({
        version: 3, message_type: "matrix", sender_device_id: matrixCrypto.deviceId,
        room_id: roomId, ciphertext: await matrixCrypto.encrypt(roomId, rawPayload),
      });
    } else if (matrixReady) {
      const roomId = await matrixCrypto.roomIdForDm(targetId);
      await matrixCrypto.prepareRoom(roomId, [targetId]);
      encryptedBody = JSON.stringify({
        version: 3, message_type: "matrix", sender_device_id: matrixCrypto.deviceId,
        room_id: roomId, ciphertext: await matrixCrypto.encrypt(roomId, rawPayload),
      });
    } else {
      const result = await api("GET", "/api/users/" + encodeURIComponent(targetId) + "/crypto-devices");
      const devices = Array.isArray(result?.devices) ? result.devices : [];
      if (!devices.length) throw new Error("Peer has no E2E devices.");
      const ciphertexts = {};
      for (const bundle of devices) {
        if (!(await ensurePeerFingerprint(peer, bundle))) throw new Error("Device could not be verified.");
        const deviceId = bundle.device_id;
        if (!cryptoDevice.has_session(deviceId)) {
          const claimed = await claimPeerOneTimeKey(targetId, deviceId);
          if (!(await ensurePeerFingerprint(peer, claimed))) throw new Error("Device could not be verified.");
          cryptoDevice.establish_session(deviceId, JSON.stringify(claimed), claimed.fingerprint);
        }
        ciphertexts[deviceId] = cryptoDevice.encrypt(deviceId, rawPayload);
      }
      encryptedBody = JSON.stringify({ version: 2, message_type: "message", sender_device_id: cryptoDevice.device_id(), ciphertexts });
    }
    if (!matrixReady) {
      await persistCryptoState();
    }
  });

  sentPlaintextByCiphertext.set(encryptedBody, rawPayload);
  void cacheSentPlaintext(encryptedBody, rawPayload);
  if (!socket || socket.readyState !== WebSocket.OPEN) throw new Error("Not connected to server.");
  socket.send(JSON.stringify({
    type: "send",
    peer_id: targetId,
    body: encryptedBody,
    ...(uploadedAttachments.length ? {
      attachment_id: uploadedAttachments[0].id,
      attachment_ids: uploadedAttachments.map((attachment) => attachment.id),
    } : {}),
  }));
}
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

const LARPTRIX_RELEASE_API = "https://api.github.com/repos/q933598-ai/larptix/releases/latest";
const LARPTRIX_RELEASES_URL = "https://github.com/q933598-ai/larptix/releases/latest";

let pendingClientUpdate = null;

function normalizedVersion(value) {
  const match = String(value || "")
    .trim()
    .replace(/^v/i, "")
    .match(/^(\d+)(?:\.(\d+))?(?:\.(\d+))?/);
  return match
    ? [Number(match[1]), Number(match[2] || 0), Number(match[3] || 0)]
    : null;
}

function compareClientVersions(left, right) {
  const a = normalizedVersion(left);
  const b = normalizedVersion(right);
  if (!a || !b) return 0;
  for (let i = 0; i < 3; i += 1) {
    if (a[i] > b[i]) return 1;
    if (a[i] < b[i]) return -1;
  }
  return 0;
}

function isAndroidClientRuntime() {
  return /Android/i.test(navigator.userAgent || "") && !globalThis.larptrixDesktop;
}

async function getNativeClientVersion() {
  try {
    if (globalThis.larptrixDesktop?.appVersion) {
      return await globalThis.larptrixDesktop.appVersion();
    }
  } catch (err) {
    console.warn("Desktop app version lookup failed:", err?.message || err);
  }

  try {
    const tauriApp = globalThis.__TAURI__?.app;
    if (typeof tauriApp?.getVersion === "function") {
      return await tauriApp.getVersion();
    }
  } catch (err) {
    console.warn("Tauri app version lookup failed:", err?.message || err);
  }

  return null;
}

async function fetchLatestLarptrixRelease() {
  const response = await fetch(LARPTRIX_RELEASE_API, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "Larptrix-client",
    },
    cache: "no-store",
  });
  if (!response.ok) {
    throw new Error(`GitHub Releases returned HTTP ${response.status}`);
  }
  return response.json();
}

function findAndroidReleaseAsset(release) {
  const version = String(release?.tag_name || "").replace(/^v/i, "");
  return Array.isArray(release?.assets)
    ? release.assets.find((asset) => asset.name === `Larptrix-${version}-android.apk`) || null
    : null;
}

function renderClientUpdateStatus(text) {
  if (settingsUpdateStatus) {
    settingsUpdateStatus.textContent = text;
  }
}

async function checkForClientUpdate({ silent = false } = {}) {
  if (!settingsUpdateStatus && !settingsCheckUpdates) return null;

  if (!silent) {
    renderClientUpdateStatus("Checking for updates…");
  }

  try {
    if (globalThis.larptrixDesktop?.checkForUpdate) {
      const update = await globalThis.larptrixDesktop.checkForUpdate();
      pendingClientUpdate = update || null;
      if (update) {
        settingsInstallUpdate.hidden = false;
        settingsInstallUpdate.textContent = "Install update";
        renderClientUpdateStatus(
          `Larptrix ${update.version} is available (current ${update.currentVersion}).`,
        );
      } else {
        settingsInstallUpdate.hidden = true;
        renderClientUpdateStatus(
          `Larptrix ${await getNativeClientVersion() || "current"} is up to date.`,
        );
      }
      return update || null;
    }

    const currentVersion = await getNativeClientVersion();
    if (!currentVersion) {
      settingsInstallUpdate.hidden = true;
      renderClientUpdateStatus(
        isAndroidClientRuntime()
          ? "Android app version could not be detected."
          : "This web client is updated with the connected server.",
      );
      return null;
    }

    const release = await fetchLatestLarptrixRelease();
    const latestVersion = String(release?.tag_name || "").replace(/^v/i, "");
    if (!latestVersion || compareClientVersions(latestVersion, currentVersion) <= 0) {
      pendingClientUpdate = null;
      settingsInstallUpdate.hidden = true;
      renderClientUpdateStatus(`Larptrix ${currentVersion} is up to date.`);
      return null;
    }

    const android = isAndroidClientRuntime();
    const asset = android ? findAndroidReleaseAsset(release) : null;
    pendingClientUpdate = {
      version: latestVersion,
      currentVersion,
      releaseUrl: release?.html_url || LARPTRIX_RELEASES_URL,
      assetUrl: asset?.browser_download_url || null,
      assetName: asset?.name || null,
    };

    settingsInstallUpdate.hidden = false;
    settingsInstallUpdate.textContent = android ? "Download APK" : "Open update";
    renderClientUpdateStatus(
      android
        ? `Larptrix ${latestVersion} is available. Download the new APK to update.`
        : `Larptrix ${latestVersion} is available.`,
    );
    return pendingClientUpdate;
  } catch (err) {
    if (!silent) {
      renderClientUpdateStatus(err?.message || "Could not check for updates.");
    }
    return null;
  }
}

function openExternalReleaseUrl(url) {
  const target = url || LARPTRIX_RELEASES_URL;
  const link = document.createElement("a");
  link.href = target;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  link.click();
}

function applyAvatarShape(shape = localStorage.getItem(AVATAR_SHAPE_KEY) || "circle") {
  const value = shape === "square" ? "square" : "circle";
  document.body.classList.toggle("avatar-shape-square", value === "square");
  localStorage.setItem(AVATAR_SHAPE_KEY, value);
  if (settingsAvatarShape) settingsAvatarShape.value = value;
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
  settingsOutgoingCallSounds.checked = readStoredBool(OUTGOING_CALL_SOUND_KEY, true);
  if (settingsIncomingCallVolume) settingsIncomingCallVolume.value = String(Number(localStorage.getItem(CALL_RING_VOLUME_KEY) ?? "0.8"));
  if (settingsOutgoingCallVolume) settingsOutgoingCallVolume.value = String(Number(localStorage.getItem(OUTGOING_RING_VOLUME_KEY) ?? "0.8"));
  renderSavedThemeOptions();
  settingsMessageSounds.checked = readStoredBool(MESSAGE_SOUND_KEY, true);
  if (settingsMessagePolicy) settingsMessagePolicy.value = me.message_policy === "friends" ? "friends" : "everyone";
  renderNotificationSettings();
  settingsInstallUpdate.hidden = true;
  void checkForClientUpdate({ silent: true });

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
  setPeopleView("chats");
  closeAppMenu();
});
menuMusic.addEventListener("click", () => {
  window.larptixMusicLibrary?.toggle?.();
  closeAppMenu();
});
menuNewGroup.addEventListener("click", () => {
  renderGroupMemberChoices();
  document.getElementById("group-create-error").hidden = true;
  document.getElementById("group-name").value = "";
  createGroupDialog.showModal();
  closeAppMenu();
});
menuNewChannel?.addEventListener("click", () => {
  renderChannelMemberChoices();
  document.getElementById("channel-create-error").hidden = true;
  document.getElementById("channel-name").value = "";
  document.getElementById("channel-post-policy").value = "admins";
  createChannelDialog?.showModal();
  closeAppMenu();
});
menuAbout?.addEventListener("click", () => {
  aboutDialog?.showModal();
  closeAppMenu();
});
document.getElementById("about-close")?.addEventListener("click", () => aboutDialog?.close());

profileOpen.addEventListener("click", async () => {
  closeAppMenu();
  try {
    const profile = await api("GET", `/api/users/${encodeURIComponent(me.user_id)}/profile`);
    profileName.value = profile.display_name;
    profileUsername.value = profile.username;
    profileAbout.value = profile.about;
    if (profileTags) profileTags.value = Array.isArray(profile.tags) ? (profile.tags[0] || "") : "";
    renderProfileTagLibrary(profile.tags);
    customActivities = getUserActivities(profile)
      .filter((item) => item.kind === "custom")
      .slice(0, 3);
    renderCustomActivityEditor();
    updateOwnProfileCard(profile);
    showMusicActivity.checked = musicActivityEnabled;
    await populateCallDeviceSelects();
    if (profileMicrophone) profileMicrophone.value = selectedCallDeviceId("audioinput");
    if (profileSpeakers) profileSpeakers.value = selectedCallDeviceId("audiooutput");
    if (profileCamera) profileCamera.value = selectedCallDeviceId("videoinput");
    profileBanner.style.backgroundImage = profile.banner_url ? 'url("' + profile.banner_url + '?v=' + Date.now() + '")' : "";
    renderNotificationSettings();
    resetE2eKeysButton.hidden = !cryptoEnabled;
    resetE2eHelp.hidden = true;
    profileDialog.showModal();
  } catch (err) {
    profileError.textContent = err.message;
    profileError.hidden = false;
  }
});
menuSettings.addEventListener("click", openSettings);
document.getElementById("settings-close").addEventListener("click", () => settingsDialog.close());
settingsAvatarShape?.addEventListener("change", () => applyAvatarShape(settingsAvatarShape.value));
settingsCheckUpdates?.addEventListener("click", () => {
  void checkForClientUpdate({ silent: false });
});
settingsInstallUpdate?.addEventListener("click", async () => {
  if (globalThis.larptrixDesktop?.installUpdate) {
    try {
      renderClientUpdateStatus("Installing update…");
      await globalThis.larptrixDesktop.installUpdate();
      return;
    } catch (err) {
      renderClientUpdateStatus(err?.message || "Could not install update.");
      return;
    }
  }

  const target = pendingClientUpdate?.assetUrl || pendingClientUpdate?.releaseUrl || LARPTRIX_RELEASES_URL;
  openExternalReleaseUrl(target);
});
settingsOpenReleases?.addEventListener("click", () => {
  if (globalThis.larptrixDesktop?.openReleases) {
    void globalThis.larptrixDesktop.openReleases();
    return;
  }
  openExternalReleaseUrl(LARPTRIX_RELEASES_URL);
});
settingsEnableNotifications?.addEventListener("click", async () => {
  try {
    await requestBrowserNotifications();
  } catch (err) {
    settingsNotificationsStatus.textContent = err.message || "Could not enable notifications.";
  }
});
settingsBrowserNotifications?.addEventListener("change", async () => {
  if (settingsBrowserNotifications.checked) {
    try {
      const enabled = await requestBrowserNotifications();
      settingsBrowserNotifications.checked = enabled;
    } catch {
      settingsBrowserNotifications.checked = false;
    }
  } else {
    localStorage.setItem("larptrix_browser_notifications", "0");
    renderNotificationSettings();
  }
});
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
settingsWallpaperTint?.addEventListener("change", () => {
  localStorage.setItem("larptrix_wallpaper_tint", settingsWallpaperTint.value);
  if (peerId) applyChatWallpaper(peerId);
});
const screenAudioMode = document.getElementById("screen-audio-mode");
if (screenAudioMode && /Firefox\//.test(navigator.userAgent) && !/Seamonkey\//.test(navigator.userAgent)) {
  for (const option of screenAudioMode.options) {
    if (option.value !== "none") option.disabled = true;
  }
  screenAudioMode.title = "Firefox does not currently expose screen/system audio through getDisplayMedia(). Use Chromium for screen audio.";
}
screenAudioMode?.addEventListener("change", () => {
  if (screenMediaStream) appendSystem("Screen sharing audio settings apply to the next share. Stop and start sharing again to change them.");
});
document.getElementById("call-audio-volume")?.addEventListener("input", (event) => {
  const volume = Number(event.target.value);
  if (remoteAudio) remoteAudio.volume = volume;
  document.querySelectorAll("#group-remotes audio[data-kind=\"audio\"]").forEach((audio) => { audio.volume = volume; });
  localStorage.setItem("larptrix_call_audio_volume", String(volume));
});
settingsTheme.addEventListener("change", () => {
  customThemeEditor.hidden = settingsTheme.value !== "custom";
  if (settingsTheme.value === "custom") loadThemeEditor();
  applyTheme(settingsTheme.value);
});
showMusicActivity?.addEventListener("change", () => {
  musicActivityEnabled = showMusicActivity.checked;
  localStorage.setItem("larptrix_show_music_activity", musicActivityEnabled ? "1" : "0");
  window.larptixMusicStatus?.refresh?.();
});
profileAddActivity?.addEventListener("click", () => {
  const value = profileNewActivity?.value?.trim() || "";
  if (!value) return;
  if (customActivities.length >= 3) return;
  customActivities.push({ kind: "custom", name: value, details: "", url: null, image_url: null });
  profileNewActivity.value = "";
  renderCustomActivityEditor();
});
profileNewActivity?.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    profileAddActivity?.click();
  }
});
themeSaveCustom.addEventListener("click", saveCustomTheme);
themeLoadSaved?.addEventListener("click", () => loadSavedTheme(savedThemeSelect?.value));
themeDeleteSaved?.addEventListener("click", () => {
  const id = savedThemeSelect?.value;
  if (!id) return;
  const next = readSavedThemes().filter((item) => item.id !== id);
  localStorage.setItem(SAVED_THEMES_KEY, JSON.stringify(next));
  renderSavedThemeOptions();
});
profileTagSelect?.addEventListener("change", () => {
  if (profileTags) profileTags.value = normalizeProfileTag(profileTagSelect.value);
});
profileTagAdd?.addEventListener("click", addProfileTag);
profileTagNew?.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    addProfileTag();
  }
});

for (const [select, key, kind, replacer] of [
  [profileMicrophone, CALL_MIC_KEY, "audioinput", null],
  [profileSpeakers, CALL_SPEAKERS_KEY, "audiooutput", null],
  [profileCamera, CALL_CAMERA_KEY, "videoinput", null],
]) {
  select?.addEventListener("change", async () => {
    const value = select.value || "";
    if (value) localStorage.setItem(key, value);
    else localStorage.removeItem(key);
    try {
      if (kind === "audiooutput") {
        await applyCallSpeakerDevice(value);
      } else if (kind === "audioinput" && localMediaStream?.getAudioTracks().length) {
        await replaceCallMicrophoneTrack();
      } else if (kind === "videoinput" && localMediaStream?.getVideoTracks().length) {
        await replaceCallVideoTrack();
      }
      await populateCallDeviceSelects();
    } catch (err) {
      if (profileDevicesStatus) profileDevicesStatus.textContent = err.message || "Could not change call device.";
      await populateCallDeviceSelects();
    }
  });
}
settingsPresence.addEventListener("change", () => setPresence(settingsPresence.value));
settingsCallSounds.addEventListener("change", () => localStorage.setItem(CALL_SOUND_KEY, settingsCallSounds.checked ? "1" : "0"));
settingsOutgoingCallSounds.addEventListener("change", () => localStorage.setItem(OUTGOING_CALL_SOUND_KEY, settingsOutgoingCallSounds.checked ? "1" : "0"));
settingsIncomingCallVolume?.addEventListener("input", (event) => {
  const value = Math.min(1, Math.max(0, Number(event.target.value)));
  localStorage.setItem(CALL_RING_VOLUME_KEY, String(value));
  if (callRingtone && !callRingtone.paused) callRingtone.volume = value;
});
settingsOutgoingCallVolume?.addEventListener("input", (event) => {
  const value = Math.min(1, Math.max(0, Number(event.target.value)));
  localStorage.setItem(OUTGOING_RING_VOLUME_KEY, String(value));
  if (callRingtone && !callRingtone.paused) callRingtone.volume = value;
});
for (const [select, key, kind] of [
  [callSettingsMicrophone, CALL_MIC_KEY, "audioinput"],
  [callSettingsSpeakers, CALL_SPEAKERS_KEY, "audiooutput"],
  [callSettingsCamera, CALL_CAMERA_KEY, "videoinput"],
]) {
  select?.addEventListener("change", async () => {
    const value = select.value || "";
    if (value) localStorage.setItem(key, value);
    else localStorage.removeItem(key);
    try {
      if (kind === "audiooutput") {
        await applyCallSpeakerDevice(value);
      } else if (kind === "audioinput" && localMediaStream?.getAudioTracks().length) {
        await replaceCallMicrophoneTrack();
      } else if (kind === "videoinput" && localMediaStream?.getVideoTracks().length) {
        await replaceCallVideoTrack();
      }
      await populateCallDeviceSelects();
    } catch (err) {
      callStatus.textContent = err.message || "Could not change call device.";
      await populateCallDeviceSelects();
    }
  });
}

settingsMessageSounds.addEventListener("change", () => localStorage.setItem(MESSAGE_SOUND_KEY, settingsMessageSounds.checked ? "1" : "0"));
settingsMessagePolicy?.addEventListener("change", async () => {
  try {
    const updated = await api("PATCH", "/api/me/settings", { message_policy: settingsMessagePolicy.value });
    me = { ...me, message_policy: updated.message_policy || settingsMessagePolicy.value };
  } catch (err) {
    settingsMessagePolicy.value = me?.message_policy === "friends" ? "friends" : "everyone";
    appendSystem(err.message || "Could not update message privacy.");
  }
});
settingsCallRingtone?.addEventListener("change", async () => {
  const file = settingsCallRingtone.files?.[0];
  settingsCallRingtone.value = "";
  if (!file) return;
  try { await setCustomCallRingtone(file); }
  catch (err) { settingsCallRingtoneStatus.textContent = err.message || "Could not set ringtone."; }
});
settingsCallRingtoneReset?.addEventListener("click", async () => {
  try { await resetCustomCallRingtone(); }
  catch (err) { settingsCallRingtoneStatus.textContent = err.message || "Could not reset ringtone."; }
});

menuSaved?.addEventListener("click", openSavedMessagesChat);
savedMessagesClose?.addEventListener("click", () => savedMessagesDialog.close());
mobileChats?.addEventListener("click", () => {
  document.getElementById("chat-view-open").click();
  document.body.classList.toggle("mobile-people-visible");
});
mobileSaved?.addEventListener("click", () => {
  document.body.classList.remove("mobile-people-visible");
  openSavedMessagesChat();
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
function closeMediaViewer() {
  imageViewerVideo?.pause();
  imageViewerVideo?.removeAttribute("src");
  imageViewerVideo?.load();
  imageViewerVideo?.setAttribute("hidden", "");
  imageViewerImage.hidden = false;
  imageViewer.close();
}
document.getElementById("image-viewer-close").addEventListener("click", closeMediaViewer);
imageViewer.addEventListener("click", (event) => {
  if (event.target === imageViewer) closeMediaViewer();
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && imageViewer.open) {
    event.preventDefault();
    closeMediaViewer();
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
resetE2eKeysButton?.addEventListener("click", () => void resetE2eKeys());

document.getElementById("create-group-cancel").addEventListener("click", () => createGroupDialog.close());
document.getElementById("create-group-form").addEventListener("submit", createGroup);
document.getElementById("create-channel-cancel")?.addEventListener("click", () => createChannelDialog.close());
document.getElementById("create-channel-form")?.addEventListener("submit", createChannel);
let friendSearchTimer = null;

function normalizePeopleSearch(raw) {
  return raw.trim().replace(/^@+/, "").replace(/\s+/g, " ").toLocaleLowerCase();
}
function searchScore(user, query) {
  const name = (user?.display_name || "").toLocaleLowerCase();
  const username = (user?.username || "").toLocaleLowerCase();
  if (!query) return 99;
  if (username === query) return 0;
  if (username.startsWith(query)) return 1;
  if (name === query) return 2;
  if (name.startsWith(query)) return 3;
  if (username.includes(query)) return 4;
  if (name.includes(query)) return 5;
  return 9;
}
function updateFriendRequestsBadge() {
  if (!friendRequestsBadge) return;
  const count = friendRequests.incoming.length;
  friendRequestsBadge.hidden = count === 0;
  friendRequestsBadge.textContent = count > 99 ? "99+" : String(count);
}
async function loadFriendRequests() {
  if (!me) return;
  const requestId = ++friendRequestsRequestId;
  try {
    const result = await api("GET", "/api/friends");
    if (requestId !== friendRequestsRequestId) return;
    friendRequests = {
      friends: Array.isArray(result?.friends) ? result.friends : [],
      incoming: Array.isArray(result?.incoming) ? result.incoming : [],
      outgoing: Array.isArray(result?.outgoing) ? result.outgoing : [],
    };
    updateFriendRequestsBadge();
    renderUsers();
  } catch (err) {
    if (requestId === friendRequestsRequestId) {
      appendSystem("Friend requests failed: " + (err.message || err));
    }
  }
}
function requestUserButton(user) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "request-user-button";
  const avatar = document.createElement("span");
  avatar.className = "avatar";
  paintAvatar(avatar, user);
  const name = document.createElement("span");
  name.className = "person-name";
  const label = document.createElement("span");
  label.textContent = user.display_name || "Larptrix user";
  name.append(label);
  if (user.username) {
    const handle = document.createElement("small");
    handle.textContent = "@" + user.username;
    name.append(handle);
  }
  button.append(avatar, name);
  button.addEventListener("click", () => openChat(user.user_id));
  return button;
}
function renderFriendRequests() {
  usersEl.replaceChildren();
  const addSection = (title, entries, kind) => {
    if (!entries.length) return;
    const heading = document.createElement("li");
    heading.className = "friend-requests-heading";
    heading.textContent = title;
    usersEl.append(heading);
    for (const user of entries) {
      const li = document.createElement("li");
      li.className = "friend-request-row";
      li.append(requestUserButton(user));
      const actions = document.createElement("div");
      actions.className = "friend-request-actions";
      if (kind === "incoming") {
        const accept = document.createElement("button");
        accept.type = "button";
        accept.className = "ghost request-accept";
        accept.textContent = "Accept";
        accept.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          accept.disabled = true;
          try {
            await api("POST", "/api/friends/" + encodeURIComponent(user.user_id) + "/accept", {});
            await loadFriendRequests();
          } catch (err) {
            accept.disabled = false;
            appendSystem(err.message || "Could not accept friend request.");
          }
        });
        const decline = document.createElement("button");
        decline.type = "button";
        decline.className = "ghost";
        decline.textContent = "Decline";
        decline.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          decline.disabled = true;
          try {
            await api("DELETE", "/api/friends/" + encodeURIComponent(user.user_id));
            await loadFriendRequests();
          } catch (err) {
            decline.disabled = false;
            appendSystem(err.message || "Could not decline friend request.");
          }
        });
        actions.append(accept, decline);
      } else {
        const cancel = document.createElement("button");
        cancel.type = "button";
        cancel.className = "ghost";
        cancel.textContent = "Cancel";
        cancel.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          cancel.disabled = true;
          try {
            await api("DELETE", "/api/friends/" + encodeURIComponent(user.user_id));
            await loadFriendRequests();
          } catch (err) {
            cancel.disabled = false;
            appendSystem(err.message || "Could not cancel friend request.");
          }
        });
        actions.append(cancel);
      }
      li.append(actions);
      usersEl.append(li);
    }
  };
  addSection("Incoming", friendRequests.incoming, "incoming");
  addSection("Sent", friendRequests.outgoing, "outgoing");
  if (!friendRequests.incoming.length && !friendRequests.outgoing.length) {
    const empty = document.createElement("li");
    empty.className = "search-empty";
    empty.textContent = "No pending friend requests.";
    usersEl.append(empty);
  }
}
function setPeopleView(view) {
  peopleView = view === "requests" ? "requests" : "chats";
  const showingRequests = peopleView === "requests";
  friendRequestsOpen?.classList.toggle("active", showingRequests);
  chatViewOpen?.classList.toggle("active", !showingRequests);
  if (userSearchInput) {
    userSearchInput.hidden = showingRequests;
    if (showingRequests) userSearchInput.value = "";
  }
  if (peopleSearchHint) peopleSearchHint.hidden = showingRequests;
  if (showingRequests) {
    searchResults = [];
    void loadFriendRequests();
  }
  renderUsers();
}
async function refreshFriendSearch() {
  const query = normalizePeopleSearch(userSearchInput.value);
  const requestId = ++searchRequestId;
  if (!query) {
    friendSearchLoading = false;
    searchResults = [];
    renderUsers();
    return;
  }
  friendSearchLoading = true;
  renderUsers();
  try {
    const result = await api("GET", "/api/users/search?q=" + encodeURIComponent(query));
    if (requestId !== searchRequestId) return;
    searchResults = Array.isArray(result?.users) ? result.users : [];
  } catch (err) {
    if (requestId !== searchRequestId) return;
    searchResults = [];
    appendSystem("User search failed: " + (err.message || err));
  } finally {
    if (requestId === searchRequestId) {
      friendSearchLoading = false;
      renderUsers();
    }
  }
}
function scheduleFriendSearch() {
  clearTimeout(friendSearchTimer);
  friendSearchTimer = setTimeout(() => void refreshFriendSearch(), 160);
}
userSearchInput.addEventListener("input", scheduleFriendSearch);
friendRequestsOpen?.addEventListener("click", () => setPeopleView("requests"));
chatViewOpen?.addEventListener("click", () => setPeopleView("chats"));

document.getElementById("emoji-picker-toggle").addEventListener("click", () => {
  emojiPicker.hidden = !emojiPicker.hidden;
});
bodyInput.addEventListener("paste", (event) => {
  const items = [...(event.clipboardData?.items || [])];
  const imageItem = items.find((item) => item.kind === "file" && item.type.startsWith("image/"));
  if (!imageItem) return;
  const file = imageItem.getAsFile();
  if (!file) return;
  event.preventDefault();
  queueAttachment(new File([file], "clipboard-image." + (file.type.split("/")[1] || "png"), { type: file.type }));
  bodyInput.focus();
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
profileBannerFile?.addEventListener("change", async () => {
  const file = profileBannerFile.files?.[0];
  profileBannerFile.value = "";
  if (!file) return;
  try {
    await uploadProfileBanner(file);
  } catch (err) {
    profileError.textContent = err.message || "Could not update profile banner.";
    profileError.hidden = false;
  }
});
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
      tags: profileTags?.value ? [normalizeProfileTag(profileTags.value)] : [],
    });
    customActivities = customActivities
      .map((item) => ({ ...item, kind: "custom" }))
      .filter((item) => item.name.trim())
      .slice(0, 3);
    musicActivityEnabled = showMusicActivity.checked;
    localStorage.setItem("larptrix_show_music_activity", musicActivityEnabled ? "1" : "0");
    await syncActivities();
    window.larptixMusicStatus?.refresh?.();
    renderMe();
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
  if (signedOutUserId) await api("POST", "/api/me/activities", { activities: [] }).catch(() => {});
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
  if (!peerId) return;
  const body = bodyInput.value.trim();
  const files = [...pendingAttachments];
  if (peerId === SAVED_MESSAGES_ID) {
    if (!body && !files.length) return;
    if (files.length) {
      appendSystem("Attachments in Saved Messages are not supported yet.");
      return;
    }
    await saveManualSavedMessage(body, replyingToMessage ? { reply_to: replyingToMessage } : {});
    return;
  }
  if (!cryptoEnabled) {
    appendSystem("Set up E2E before sending messages.");
    return;
  }
  try {
    await waitForSocketOpen();
    await cryptoReady;
    if (!cryptoDevice) throw new Error("Unlock E2E before sending messages.");
    if (!body && !files.length) return;
    await sendEncryptedPayloadToPeer(peerId, {
      text: body,
      ...(replyingToMessage ? { reply_to: replyingToMessage } : {}),
      ...(videoMessageFile && files.includes(videoMessageFile)
        ? {
            video_message: {
              shape: videoMessageShapeByFile.get(videoMessageFile) || videoMessageShape,
              name: videoMessageFile.name,
            },
          }
        : {}),
    }, files);
    bodyInput.value = "";
    clearAttachment();
    clearReplyComposer();
  } catch (err) {
    appendSystem(err.message || "Could not encrypt the message.");
  }
});

photoInput.addEventListener("change", () => {
  queueAttachments(photoInput.files);
  photoInput.value = "";
});
fileInput.addEventListener("change", () => {
  queueAttachments(fileInput.files);
  fileInput.value = "";
});
audioFileInput.addEventListener("change", () => {
  queueAttachments(audioFileInput.files);
  audioFileInput.value = "";
});

recordVideoButton?.addEventListener("click", () => void toggleVideoRecording());

let fileDropDepth = 0;

function hasDroppedFiles(event) {
  return Array.from(event.dataTransfer?.types || []).includes("Files");
}

function setFileDropActive(active) {
  chatEl?.classList.toggle("file-drop-active", active);
}

function handleChatDragEnter(event) {
  if (!hasDroppedFiles(event) || event.target.closest("#music-library")) return;
  event.preventDefault();
  fileDropDepth += 1;
  setFileDropActive(true);
}

function handleChatDragOver(event) {
  if (!hasDroppedFiles(event) || event.target.closest("#music-library")) return;
  event.preventDefault();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
  setFileDropActive(true);
}

function handleChatDragLeave(event) {
  if (!hasDroppedFiles(event) || event.target.closest("#music-library")) return;
  event.preventDefault();
  fileDropDepth = Math.max(0, fileDropDepth - 1);
  if (fileDropDepth === 0) setFileDropActive(false);
}

function handleChatDrop(event) {
  if (!hasDroppedFiles(event) || event.target.closest("#music-library")) return;
  event.preventDefault();
  fileDropDepth = 0;
  setFileDropActive(false);
  queueAttachments(event.dataTransfer.files);
}

chatEl?.addEventListener("dragenter", handleChatDragEnter);
chatEl?.addEventListener("dragover", handleChatDragOver);
chatEl?.addEventListener("dragleave", handleChatDragLeave);
chatEl?.addEventListener("drop", handleChatDrop);

recordAudioButton.addEventListener("click", toggleRecording);
document.getElementById("start-audio-call").addEventListener("click", () => openOrJoinDirectCall("audio"));
document.getElementById("start-video-call").addEventListener("click", () => openOrJoinDirectCall("video"));
groupMembersOpen?.addEventListener("click", openGroupMembers);
groupProfileForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  const group = groups.find((item) => item.user_id === peerId && item.is_group);
  if (!group) return;
  groupProfileError.hidden = true;
  try {
    const endpoint = group.is_channel
      ? "/api/channels/" + encodeURIComponent(group.user_id) + "/settings"
      : "/api/groups/" + encodeURIComponent(group.user_id) + "/settings";
    const updated = await api("PATCH", endpoint, {
      name: groupProfileName.value.trim(),
      description: groupProfileDescription.value.trim(),
      ...(group.is_channel ? { post_policy: group.post_policy || "admins" } : {}),
    });

    if (groupProfileAvatar.files[0]) {
      await uploadFile(
        "/api/groups/" + encodeURIComponent(group.user_id) + "/avatar",
        groupProfileAvatar.files[0],
      );
    }
    if (group.is_channel && groupProfileBanner.files[0]) {
      await uploadFile(
        "/api/groups/" + encodeURIComponent(group.user_id) + "/banner",
        groupProfileBanner.files[0],
      );
    }

    await loadGroups();
    const refreshed = groups.find((item) => item.user_id === group.user_id);
    if (refreshed) {
      peerName.textContent = refreshed.display_name;
      peerMeta.textContent = refreshed.is_channel
        ? `${refreshed.subscriber_count || refreshed.group_member_ids.length} subscriber(s)`
        : `${refreshed.group_member_ids.length} member(s)`;
      renderGroupMembersDialog(refreshed);
    }
  } catch (err) {
    groupProfileError.textContent = err.message || "Could not update group profile.";
    groupProfileError.hidden = false;
  }
});

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
endCallButton?.addEventListener("click", () => endCall(true));
endGroupCallButton?.addEventListener("click", endGroupCallForEveryone);
const savedCallVolume = Number(localStorage.getItem("larptrix_call_audio_volume"));
if (Number.isFinite(savedCallVolume)) {
  remoteAudio.volume = Math.min(1, Math.max(0, savedCallVolume));
  const volumeControl = document.getElementById("call-audio-volume");
  if (volumeControl) volumeControl.value = String(remoteAudio.volume);
}
enableCallAudio.addEventListener("click", () => {
  remoteAudio.play().then(() => {
    enableCallAudio.hidden = true;
  }).catch((err) => appendSystem(err.message || "Could not play call audio."));
});
document.getElementById("toggle-microphone").addEventListener("click", toggleMicrophone);
callDeafenButton?.addEventListener("click", toggleCallDeafen);
callSettingsOpen?.addEventListener("click", () => {
  if (!callSettingsPanel) return;
  callSettingsPanel.hidden = !callSettingsPanel.hidden;
  void populateCallDeviceSelects();
  if (callNoiseSuppression) callNoiseSuppression.checked = readStoredBool(NOISE_SUPPRESSION_KEY, true);
  renderCallParticipantSettings();
});
callNoiseSuppression?.addEventListener("change", () => void toggleCallNoiseSuppression());
callWindowPin?.addEventListener("click", () => {
  setCallPinned(localStorage.getItem("larptrix_call_window_pinned") !== "1");
});

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
    if (document.fullscreenElement) {
      await document.exitFullscreen();
      return;
    }

    // During screen sharing, fullscreen the actual shared video rather than
    // the floating call window. This removes the call controls from over the
    // demo and matches Discord-style screen-share fullscreen behavior.
    const screenTarget = screenMediaStream
      ? localScreenVideo
      : (callStage.classList.contains("screen-sharing") ? remoteVideo : null);
    if (screenTarget && !screenTarget.hidden) {
      await screenTarget.requestFullscreen();
      return;
    }

    if (globalThis.larptrixDesktop?.toggleFullscreen) {
      const fullscreen = await globalThis.larptrixDesktop.toggleFullscreen();
      callStage.classList.toggle("native-window-fullscreen", Boolean(fullscreen));
      return;
    }
    const tauriInvoke = globalThis.__TAURI__?.core?.invoke;
    if (typeof tauriInvoke === "function") {
      const fullscreen = await tauriInvoke("toggle_fullscreen");
      callStage.classList.toggle("native-window-fullscreen", Boolean(fullscreen));
      return;
    }
    await document.getElementById("call-videos").requestFullscreen();
  } catch (err) {
    appendSystem(err.message || "Fullscreen is not available in this app.");
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

wireCallWindowDragging();
wireCallResponsiveSizing();
setTimeout(() => void checkForClientUpdate({ silent: true }), 12000);
setInterval(() => void checkForClientUpdate({ silent: true }), 6 * 60 * 60 * 1000);
setCallPinned(localStorage.getItem("larptrix_call_window_pinned") === "1");
void loadCustomCallRingtone();
applyAvatarShape();
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
      resetE2eKeysButton.hidden = false;
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
      resetE2eKeysButton.hidden = false;
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

async function loadMatrixCryptoStatus({ freshStart = false } = {}) {
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
    freshStart,
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
  const cacheKey = ids.slice().sort().join(",");
  const cachedAt = matrixDeviceCheckCache.get(cacheKey) || 0;
  if (Date.now() - cachedAt < 15000) return;

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

    if (!lastMissing.length) {
      matrixDeviceCheckCache.set(cacheKey, Date.now());
      return;
    }
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

async function clearLocalE2eCaches() {
  if (typeof indexedDB === "undefined") return;
  let db;
  try {
    db = await openLocalCryptoDb();
    await new Promise((resolve, reject) => {
      const transaction = db.transaction("items", "readwrite");
      const store = transaction.objectStore("items");
      const cursorRequest = store.openCursor();
      cursorRequest.onsuccess = () => {
        const cursor = cursorRequest.result;
        if (!cursor) return;
        const id = String(cursor.key || "");
        if (id.startsWith("decrypted-message:") || id.startsWith("sent-plaintext:")) {
          cursor.delete();
        }
        cursor.continue();
      };
      transaction.addEventListener("complete", resolve, { once: true });
      transaction.addEventListener("error", () => reject(transaction.error), { once: true });
      transaction.addEventListener("abort", () => reject(transaction.error), { once: true });
    });
  } catch {
    // Best-effort cleanup; the new recovery key cannot decrypt old cache records anyway.
  } finally {
    db?.close();
  }
}

async function resetE2eKeys() {
  if (!me || !cryptoDevice || !cryptoRecoveryKey) {
    appendSystem("Unlock E2E before resetting your keys.");
    return;
  }
  const currentPeerId = peerId && peerId !== SAVED_MESSAGES_ID
    && !groups.some((group) => group.user_id === peerId)
    ? peerId
    : null;

  const confirmed = window.confirm(
    "Reset your E2E keys? This device will start a new encryption identity. The server will keep the encrypted history, and the other participant will keep their readable copy. This device will show a fresh chat from the reset point."
  );
  if (!confirmed) return;

  const oldCryptoDeviceId = cryptoDevice.device_id();
  const oldMatrixDeviceId = matrixCrypto?.deviceId || null;
  const resetAt = Date.now();

  resetE2eKeysButton.disabled = true;
  resetE2eKeysButton.textContent = "Resetting…";

  try {
    await withCryptoStateLock(async () => {
      if (matrixCrypto) {
        await matrixCrypto.close().catch(() => {});
        matrixCrypto = null;
      }

      if (oldMatrixDeviceId) {
        await api("DELETE", "/api/me/matrix-devices/" + encodeURIComponent(oldMatrixDeviceId));
      }
      await api("DELETE", "/api/me/crypto-devices/" + encodeURIComponent(oldCryptoDeviceId));

      await clearLocalE2eCaches();
      sentPlaintextByCiphertext.clear();
      decryptedPayloadByMessageId.clear();
      recoveredBodiesByMessageId.clear();
      cryptoRecoveryPending.clear();
      cryptoRecoveryResponsesByMessageId.clear();
      messageBodyElementsById.clear();

      if (currentPeerId) {
        setE2eResetAt(currentPeerId, resetAt);
      }

      localStorage.removeItem(savedMessagesKey());

      cryptoDevice?.free();
      cryptoRecoveryKey = createRecoveryKey();
      cryptoDevice = new CryptoDevice(cryptoRecoveryKey);
      cryptoDeviceBundle = JSON.parse(cryptoDevice.public_bundle_json());
      cryptoStoredState = null;
      cryptoEnabled = true;
      await persistCryptoState();
      await rememberRecoveryKey(me.user_id, cryptoRecoveryKey);
      forgetE2eDeviceButton.hidden = false;
      resetE2eKeysButton.hidden = false;
    });

    matrixCryptoReady = (async () => {
      try {
        return Boolean(await loadMatrixCryptoStatus({ freshStart: true }));
      } catch (err) {
        console.error("[E2E] Matrix crypto reset initialization failed", err);
        return false;
      }
    })();
    await matrixCryptoReady;

    if (currentPeerId === peerId) {
      logEl.replaceChildren();
      messageBodyElementsById.clear();
      renderE2eResetNotice(currentPeerId);
    }
    resetE2eHelp.hidden = false;
    cryptoProfileStatus.textContent = "New E2E identity created. Save the new recovery key below.";
    openCryptoDialog("enable", { required: true });
  } catch (err) {
    cryptoProfileStatus.textContent = "E2E reset failed: " + (err.message || String(err));
    appendSystem("Could not reset E2E keys: " + (err.message || String(err)));
  } finally {
    resetE2eKeysButton.disabled = false;
    resetE2eKeysButton.textContent = "Reset E2E keys";
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
  }).then((ready) => {
    if (ready) void retryVisibleMatrixMessages({ attempts: 12, delayMs: 300 });
    return ready;
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

async function waitForSocketOpen(timeoutMs = 8000) {
  if (socket?.readyState === WebSocket.OPEN) return;
  if (!me) throw new Error("Not connected to a Larptrix account.");

  if (!socket || socket.readyState === WebSocket.CLOSED || socket.readyState === WebSocket.CLOSING) {
    connect();
  }

  const candidate = socket;
  if (!candidate) throw new Error("Could not reconnect to the server.");
  if (candidate.readyState === WebSocket.OPEN) return;

  await new Promise((resolve, reject) => {
    let settled = false;
    const finish = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      candidate.removeEventListener("open", onOpen);
      candidate.removeEventListener("error", onFailure);
      candidate.removeEventListener("close", onFailure);
      if (error) reject(error);
      else resolve();
    };
    const onOpen = () => finish();
    const onFailure = () => finish(new Error("Connection to the server was lost. Please try again."));
    const timer = setTimeout(
      () => finish(new Error("Still reconnecting to Larptrix. Please try sending again in a moment.")),
      timeoutMs,
    );

    candidate.addEventListener("open", onOpen, { once: true });
    candidate.addEventListener("error", onFailure, { once: true });
    candidate.addEventListener("close", onFailure, { once: true });
  });

  if (socket !== candidate || candidate.readyState !== WebSocket.OPEN) {
    throw new Error("Connection changed while reconnecting. Please try again.");
  }
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
        users = Array.isArray(msg.users) ? msg.users : [];
        me.message_policy = msg.user?.message_policy || me.message_policy || "everyone";
        renderMe();
        renderUsers();
        void loadGroups();
        void loadFriendRequests();
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
        users = Array.isArray(msg.users) ? msg.users : [];
        if (!users.some((user) => user.user_id === peerId) && peerId && !groups.some((group) => group.user_id === peerId)) {
          peerId = null;
          chatTitlebar.hidden = true;
          composer.hidden = true;
          emptyEl.hidden = false;
        }
        renderUsers();
        renderGroupCallBanner();
        if (userSearchInput.value.trim()) void refreshFriendSearch();
        break;
      case "groups":
        groups = msg.groups.map((group) => ({
          user_id: group.group_id,
          display_name: group.name,
          online: true,
          is_group: true,
          is_channel: Boolean(group.is_channel),
          admin_ids: Array.isArray(group.admin_ids) ? group.admin_ids : [],
          post_policy: group.post_policy || (group.is_channel ? "admins" : "members"),
          group_member_ids: Array.isArray(group.member_ids) ? group.member_ids : [],
          group_description: group.description || "",
          group_avatar_url: group.avatar_url || null,
          group_banner_url: group.banner_url || null,
          avatar_url: group.avatar_url || null,
          banner_url: group.banner_url || null,
          subscriber_count: Number(group.subscriber_count || group.member_ids?.length || 0),
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
        peerName.classList.toggle("has-activities", getUserActivities(msg.peer).length > 0);
        peerMeta.textContent = msg.peer.is_channel
          ? `${msg.peer.subscriber_count || msg.peer.group_member_ids?.length || 0} subscriber(s)`
          : msg.peer.is_group
            ? `${msg.peer.group_member_ids?.length || 0} member(s)`
            : "";
        if (!msg.peer.is_group) void refreshPeerVerification(msg.peer.user_id);
        if (msg.peer.is_group && !groups.some((group) => group.user_id === msg.peer.user_id)) {
          groups.push({ ...msg.peer });
          renderUsers();
        } else if (msg.peer.is_group) {
          groups = groups.map((group) => group.user_id === msg.peer.user_id ? { ...group, ...msg.peer } : group);
        }
        document.getElementById("start-audio-call").hidden = msg.peer.is_group;
        document.getElementById("start-video-call").hidden = msg.peer.is_group;
        groupCallStart.hidden = !msg.peer.is_group || msg.peer.is_channel;
        groupCallStart.textContent = msg.peer.is_group && !msg.peer.is_channel
          ? (activeGroupCalls.get(msg.peer.user_id)?.active ? "Join group call" : "Group call")
          : "Group call";
        groupMembersOpen.hidden =
          !msg.peer.is_group
          || (msg.peer.is_channel && !msg.peer.admin_ids?.includes(me?.user_id));
        renderGroupCallBanner();
        channelViewObserver?.disconnect();
        viewedChannelMessages.clear();
        logEl.replaceChildren();
        messageBodyElementsById.clear();
        renderE2eResetNotice(msg.peer.user_id);
        msg.history.forEach(appendMessage);
        void retryVisibleMatrixMessages({ attempts: 10, delayMs: 300 });
        break;
      case "message":
        if (isForOpenChat(msg.message)) appendMessage(msg.message);
        if (msg.message?.sender_id !== me?.user_id) {
          playIncomingMessageSound();
          const sender = users.find((user) => user.user_id === msg.message.sender_id);
          showBrowserNotification(
            msg.message.sender_name || "Larptrix",
            "New message",
            "message-" + msg.message.sender_id,
            sender?.avatar_url || null,
          );
        }
        break;
      case "presence":
        if (msg.user_id) {
          presenceByUserId.set(msg.user_id, msg.status);
          if (msg.user_id === me?.user_id) renderPresenceStatus(msg.status);
          renderUsers();
        }
        break;
      case "message_reaction": {
        const message = messagesById.get(msg.message_id);
        if (message) {
          message.reactions = Array.isArray(msg.reactions) ? msg.reactions : [];
          const row = logEl.querySelector(`[data-message-id="${CSS.escape(msg.message_id)}"]`);
          if (row) renderMessageReactions(message, row);
        }
        break;
      }
      case "message_view_update": {
        const message = messagesById.get(msg.message_id);
        if (message) message.view_count = Number(msg.view_count || 0);
        const row = logEl.querySelector(`[data-message-id="${CSS.escape(msg.message_id)}"]`);
        if (row) renderMessageReactions(message, row);
        break;
      }
      case "message_deleted":
        deletedMessageIds.add(msg.message_id);
        messagesById.delete(msg.message_id);
        messageBodyElementsById.delete(msg.message_id);
        cryptoRecoveryPending.delete(msg.message_id);
        recoveredBodiesByMessageId.delete(msg.message_id);
        cryptoRecoveryResponsesByMessageId.delete(msg.message_id);
        void writeLocalCryptoRecord({ id: `decrypted-message:${msg.message_id}`, deleted: true }).catch(() => {});
        const deletedRow = logEl.querySelector(`[data-message-id="${CSS.escape(msg.message_id)}"]`);
        if (deletedRow) channelViewObserver?.unobserve(deletedRow);
        deletedRow?.remove();
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
          await retryVisibleMatrixMessages({ attempts: 10, delayMs: 300 });
        }).catch((err) => {
          console.error("[E2E] Matrix to-device processing failed", err);
        });
        break;
      case "call_signal":
        if (
          msg.kind === "offer"
          && msg.sender_id
          && msg.sender_id !== me?.user_id
          && localStorage.getItem(PRESENCE_KEY) !== "dnd"
        ) {
          const caller = users.find((item) => item.user_id === msg.sender_id)?.display_name || "Larptrix user";
          showBrowserNotification(caller, "Incoming call", "call-" + msg.sender_id);
        }
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
  const isAdmin = (group.admin_ids || []).includes(me?.user_id);
  const canViewMembers = !group.is_channel || isAdmin;

  groupMembersTitle.textContent = group.is_channel ? "Channel settings" : "Group settings";
  groupMembersHelp.textContent = group.is_channel
    ? `${group.subscriber_count || memberIds.length} subscriber(s) · ${group.post_policy === "admins" ? "admins can post" : "all members can post"}`
    : `${memberIds.length} member(s)`;

  groupMembersList.replaceChildren();

  if (isAdmin) {
    groupProfileForm.hidden = false;
    groupProfileName.value = group.display_name || "";
    groupProfileDescription.value = group.group_description || "";
    groupProfileAvatar.value = "";
    groupProfileBanner.value = "";
    groupProfileBannerLabel.hidden = !group.is_channel;
    groupProfileError.hidden = true;

    const settingsBox = document.createElement("div");
    settingsBox.className = "channel-admin-settings";

    if (group.is_channel) {
      const policyLabel = document.createElement("label");
      policyLabel.textContent = "Who can post";
      const policy = document.createElement("select");
      policy.innerHTML = '<option value="admins">Admins only</option><option value="members">All members</option>';
      policy.value = group.post_policy || "admins";
      policy.addEventListener("change", async () => {
        try {
          await api("PATCH", "/api/channels/" + encodeURIComponent(group.user_id) + "/settings", {
            post_policy: policy.value,
            name: group.display_name,
            description: group.group_description || "",
          });
          group.post_policy = policy.value;
        } catch (err) {
          appendSystem(err.message || "Could not update channel posting settings.");
          policy.value = group.post_policy || "admins";
        }
      });
      settingsBox.append(policyLabel, policy);
    }

    groupMembersList.append(settingsBox);
  } else {
    groupProfileForm.hidden = true;
  }

  if (group.is_channel && !canViewMembers) {
    return;
  }

  const friendIds = new Set(
    friendRequests.friends.map((friend) => friend.user_id)
  );

  if (isAdmin) {
    const addBox = document.createElement("div");
    addBox.className = "group-add-member";
    const addSelect = document.createElement("select");
    addSelect.className = "group-add-member-select";
    addSelect.innerHTML = '<option value="">Add friend…</option>';
    users
      .filter(
        (item) =>
          friendIds.has(item.user_id)
          && item.user_id !== me?.user_id
          && !item.is_group
          && !memberIds.includes(item.user_id)
          && item.e2e_enabled
      )
      .forEach((item) => {
        const option = document.createElement("option");
        option.value = item.user_id;
        option.textContent = item.display_name + (item.username ? " · @" + item.username : "");
        addSelect.append(option);
      });

    const addButton = document.createElement("button");
    addButton.type = "button";
    addButton.className = "ghost";
    addButton.textContent = "Add";
    addButton.addEventListener("click", async () => {
      if (!addSelect.value) return;
      addButton.disabled = true;
      try {
        await api("POST", "/api/groups/" + encodeURIComponent(group.user_id) + "/members", {
          user_id: addSelect.value,
        });
        await loadGroups();
        const updated = groups.find((item) => item.user_id === group.user_id);
        if (updated) renderGroupMembersDialog(updated);
        addSelect.value = "";
      } catch (err) {
        appendSystem(err.message || "Could not add group member.");
      } finally {
        addButton.disabled = false;
      }
    });
    addBox.append(addSelect, addButton);
    groupMembersList.append(addBox);
  }

  const membersHeading = document.createElement("strong");
  membersHeading.textContent = group.is_channel ? "Subscribers" : "Members";
  groupMembersList.append(membersHeading);

  for (const memberId of memberIds) {
    const user = users.find((item) => item.user_id === memberId);
    const online = Boolean(user?.online);
    const row = document.createElement("div");
    row.className = "group-member-row";

    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user || {
      user_id: memberId,
      display_name: "?",
      avatar_url: null,
      is_group: false,
    });

    const copy = document.createElement("div");
    copy.className = "group-member-copy";
    const name = document.createElement("strong");
    name.textContent = user?.display_name || "Unknown member";
    const meta = document.createElement("span");
    meta.textContent = user?.username
      ? `@${user.username} · ${online ? "Online" : "Offline"}`
      : (online ? "Online" : "Offline");
    copy.append(name, meta);

    if (isAdmin && group.is_channel && group.admin_ids?.includes(memberId)) {
      const admin = document.createElement("span");
      admin.className = "channel-admin-label";
      admin.textContent = "Admin";
      copy.append(admin);
    }

    row.append(avatar, copy);
    groupMembersList.append(row);
  }
}
function openGroupMembers() {
  const group = groups.find((item) => item.user_id === peerId && item.is_group)
    || groups.find((item) => item.user_id === groupCallGroupId && item.is_group);
  if (!group) return;
  if (group.is_channel && !group.admin_ids?.includes(me?.user_id)) {
    appendSystem("Only channel admins can view subscribers.");
    return;
  }
  renderGroupMembersDialog(group);
  groupMembersDialog.showModal();
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
  groupCallBannerAvatars?.replaceChildren();
  for (const id of state.participant_ids.slice(0, 6)) {
    const avatar = document.createElement("span");
    avatar.className = "avatar call-mini-avatar";
    paintAvatar(avatar, users.find((item) => item.user_id === id) || (id === me?.user_id ? me : null) || { user_id: id, display_name: "?" });
    groupCallBannerAvatars?.append(avatar);
  }
  groupCallBannerMeta.textContent =
    " · " + state.participant_ids.length + "/" + group.group_member_ids.length + " joined";
  groupCallJoin.textContent = joined ? "Open call" : "Join";
  groupCallJoin.disabled = joined && groupCallGroupId !== group.user_id;
  groupCallJoin.hidden = groupCallId === state.call_id && groupCallGroupId === group.user_id;
  updateGroupCallControls();
  if (groupCallStart) {
    groupCallStart.hidden = false;
    groupCallStart.textContent = joined ? "Group call" : "Join group call";
  }
}

function handleGroupCallState(message) {
  if (!message?.group_id || !message?.call_id) return;
  if (message.active) {
    activeGroupCalls.set(message.group_id, message);
    if (groupCallId === message.call_id && groupCallGroupId === message.group_id) {
      groupCallInitiatorId = message.initiator_id || groupCallInitiatorId;
      groupCallJoinedMembers.clear();
      for (const id of message.participant_ids || []) groupCallJoinedMembers.add(id);
      for (const remoteId of [...groupPeerConnections.keys()]) {
        if (!(message.participant_ids || []).includes(remoteId)) removeGroupPeer(remoteId);
      }
    }
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
    showCallStage();
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

function updateGroupCallControls() {
  const inGroupCall = Boolean(groupCallId && groupCallGroupId);
  if (endCallButton) endCallButton.textContent = inGroupCall ? "Leave call" : "End call";
  if (endGroupCallButton) {
    endGroupCallButton.hidden = !(inGroupCall && groupCallInitiatorId === me?.user_id);
  }
}

function sendGroupCallControl(kind) {
  if (!groupCallGroupId || !groupCallId || !socket || socket.readyState !== WebSocket.OPEN) return;
  socket.send(JSON.stringify({
    type: "call_signal",
    peer_id: groupCallGroupId,
    kind,
    payload: {
      group_id: groupCallGroupId,
      call_id: groupCallId,
      media: callMediaKind || "audio",
    },
  }));
}

function endGroupCallForEveryone() {
  if (!groupCallGroupId || !groupCallId || groupCallInitiatorId !== me?.user_id) return;
  sendGroupCallControl("group_end");
  endCall(false);
}

function isDirectCallActive(id) {
  return Boolean(
    peerConnection
      && callPeerId === id
      && ["connecting", "connected", "new"].includes(peerConnection.connectionState),
  );
}

function formatCallDuration(totalSeconds) {
  const seconds = Math.max(0, Math.floor(Number(totalSeconds) || 0));
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  return minutes + ":" + String(rest).padStart(2, "0");
}

function getDirectCallUser(id) {
  if (!id) return null;
  return id === me?.user_id
    ? me
    : users.find((item) => item.user_id === id)
      || { user_id: id, display_name: "Larptrix user" };
}

function renderDirectCallTopbar(id = peerId) {
  if (!directCallJoin) return;
  const callButton = document.getElementById("start-audio-call");
  const videoButton = document.getElementById("start-video-call");
  const selected = getChatEntries().find((item) => item.user_id === id);
  const remoteId = callPeerId
    || pendingIncomingCall?.sender_id
    || lastDirectCallJoinPeerId
    || (id && id !== me?.user_id ? id : null);
  const active = Boolean(remoteId && isDirectCallActive(remoteId));
  const incoming = Boolean(remoteId && pendingIncomingCall?.sender_id === remoteId && !peerConnection);
  const joinable = Boolean(remoteId && lastDirectCallJoinPeerId === remoteId && !peerConnection && !directCallOutgoing);
  const isDirect = Boolean(remoteId && !selected?.is_group && !groupCallId);

  if (!isDirect) {
    directCallJoin.hidden = true;
    if (callButton) callButton.hidden = Boolean(selected?.is_group);
    if (videoButton) videoButton.hidden = Boolean(selected?.is_group);
    return;
  }

  if (active && directCallOutgoing) {
    paintAvatar(directCallTopbarAvatar, getDirectCallUser(remoteId));
    directCallTopbarLabel.textContent = "Calling…";
    directCallJoin.disabled = true;
    directCallJoin.hidden = false;
  } else if (incoming || joinable) {
    paintAvatar(directCallTopbarAvatar, getDirectCallUser(remoteId));
    directCallTopbarLabel.textContent =
      (callMediaKind || lastDirectCallJoinKind) === "video" ? "Join video" : "Join call";
    directCallJoin.disabled = false;
    directCallJoin.hidden = false;
  } else if (active) {
    paintAvatar(directCallTopbarAvatar, getDirectCallUser(remoteId));
    directCallTopbarLabel.textContent = "Connected";
    directCallJoin.disabled = true;
    directCallJoin.hidden = false;
  } else {
    directCallJoin.hidden = true;
  }

  if (callButton) callButton.hidden = true;
  if (videoButton) videoButton.hidden = true;
}

async function sendDirectCallLog(peerTargetId, kind, answered, startedAt, answeredAt = 0, endedAt = Date.now(), sessionId = null) {
  if (!peerTargetId || !startedAt || !directCallOfferSent) return;
  if (sessionId && loggedDirectCallSessions.has(sessionId)) return;
  if (sessionId) loggedDirectCallSessions.add(sessionId);
  const durationStart = answered && answeredAt ? answeredAt : startedAt;
  const duration = Math.max(0, Math.round((endedAt - durationStart) / 1000));
  try {
    await sendEncryptedPayloadToPeer(peerTargetId, {
      call_log: {
        kind: kind === "video" ? "video" : "audio",
        status: answered ? "completed" : "missed",
        duration_seconds: duration,
      },
    });
  } catch (err) {
    console.warn("[Call] Could not save call history:", err?.message || err);
  }
}

function updateDirectCallButtons(id = peerId) {
  const callButton = document.getElementById("start-audio-call");
  const videoButton = document.getElementById("start-video-call");
  if (!callButton || !videoButton) return;
  const selected = getChatEntries().find((item) => item.user_id === id);
  if (selected?.is_group) {
    callButton.textContent = "Call";
    videoButton.textContent = "Video";
    callButton.hidden = true;
    videoButton.hidden = true;
    renderDirectCallTopbar(id);
    return;
  }
  const joining = isDirectCallActive(id);
  const joinable = Boolean(
    pendingIncomingCall?.sender_id === id
      || lastDirectCallJoinPeerId === id
  );
  callButton.textContent = joining || joinable ? "Join" : "Call";
  videoButton.textContent = joining || joinable ? "Join video" : "Video";
  renderDirectCallTopbar(id);
}
function openOrJoinDirectCall(kind) {
  if (!peerId) return;
  if (pendingIncomingCall?.sender_id === peerId && !peerConnection) {
    void acceptIncomingCall();
    return;
  }
  if (lastDirectCallJoinPeerId === peerId && !pendingIncomingCall && !peerConnection) {
    void startCall(lastDirectCallJoinKind || kind);
    return;
  }
  if (isDirectCallActive(peerId)) {
    callStage.hidden = false;
    callStage.classList.remove("call-collapsed");
    updateCallPlaceholder({ force: !localMediaStream?.getVideoTracks().length });
    return;
  }
  void startCall(kind);
}

function openChat(id) {
  if (id === SAVED_MESSAGES_ID) {
    openSavedMessagesChat();
    return;
  }
  // Switching chats must not terminate an active call.
  // Calls live independently from the currently opened chat.
  peerId = id;
  const selected = getChatEntries().find((user) => user.user_id === id);
  peerVerified.hidden = true;
  if (!selected?.is_group) void refreshPeerVerification(id);
  const isChannel = Boolean(selected?.is_channel);
  const isGroup = Boolean(selected?.is_group);
  document.getElementById("start-audio-call").hidden = isGroup || isChannel;
  document.getElementById("start-video-call").hidden = isGroup || isChannel;
  updateDirectCallButtons(id);
  groupMembersOpen.hidden = !isGroup || (isChannel && !selected?.admin_ids?.includes(me?.user_id));
  groupCallStart.hidden = !isGroup || isChannel;
  groupCallStart.textContent = isGroup && !isChannel
    ? (activeGroupCalls.get(id)?.active ? "Join group call" : "Group call")
    : "Group call";
  peerMeta.textContent = isChannel
    ? `${selected?.subscriber_count || selected?.group_member_ids?.length || 0} subscriber(s)`
    : isGroup
      ? `${selected?.group_member_ids?.length || 0} member(s)`
      : "";
  applyChatWallpaper(id);
  chatTitlebar.hidden = false;
  renderUsers();
  renderDirectCallAvatarStack();
  renderGroupCallBanner();
  updateGroupCallControls();
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "open", peer_id: id }));
  }
}


async function startGroupCall(kind) {
  if (!peerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  const group = groups.find((item) => item.user_id === peerId && item.is_group);
  if (!group) return;
  if (group.is_channel) {
    appendSystem("Channels do not support calls.");
    return;
  }
  if (typeof globalThis.RTCPeerConnection !== "function") {
    appendSystem("Group calls require WebRTC support in this desktop runtime.");
    return;
  }

  const onlineMembers = group.group_member_ids
    .filter((id) => id !== me.user_id)
    .filter((id) => users.some((user) => user.user_id === id && user.online));

  const existing = activeGroupCalls.get(group.user_id);
  if (existing?.active) {
    await joinActiveGroupCall();
    return;
  }

  if (peerConnection || groupPeerConnections.size || groupCallId) endCall(true);

  groupCallId = crypto.randomUUID();
  groupCallGroupId = group.user_id;
  groupCallMemberIds = [...group.group_member_ids];
  groupCallInitiatorId = me.user_id;
  updateGroupCallControls();
  startOutgoingCallRingtone();
  groupCallJoinedMembers.clear();
  groupCallJoinedMembers.add(me.user_id);
  callPeerId = group.user_id;
  callMediaKind = kind;
  activeGroupCalls.set(group.user_id, {
    group_id: group.user_id,
    call_id: groupCallId,
    media: kind,
    initiator_id: me.user_id,
    participant_ids: [me.user_id],
    active: true,
  });
  refreshGroupCallParticipants();
  renderGroupCallBanner();
  callWindowTitle.textContent = "group call.exe";

  try {
    localMediaStream = await acquireCallMedia(kind);
    showCallStage();
    callStage.classList.remove("call-collapsed");
    remoteVideo.hidden = true;
    remoteAudio.hidden = true;
    document.getElementById("group-remotes").hidden = false;
    await attachLocalMediaPreview();
    callStatus.textContent = "Starting group " + (kind === "video" ? "video " : "") + "call…" + callMediaNotice;
    refreshGroupCallParticipants();

    for (const remoteId of onlineMembers) {
      sendGroupCallSignal(remoteId, "group_invite", {
        group_id: groupCallGroupId,
        call_id: groupCallId,
        media: kind,
      });
    }
  } catch (err) {
    appendSystem("Group call setup failed: " + (err.message || "Check camera and microphone permissions."));
    endCall(false);
  }
}

async function establishGroupOffers() {
  const onlineMembers = groupCallMemberIds
    .filter((id) => id !== me.user_id)
    .filter((id) => groupCallJoinedMembers.has(id))
    .filter((id) => users.some((user) => user.user_id === id && user.online));

  for (const remoteId of onlineMembers) {
    if (me.user_id < remoteId) {
      await createGroupOffer(remoteId);
    }
  }
}

async function createGroupPeerConnection(remoteId) {
  if (typeof globalThis.RTCPeerConnection !== "function") {
    throw new Error("WebRTC is unavailable.");
  }
  const config = await api("GET", "/api/rtc-config");
  if (!Array.isArray(config.ice_servers) || config.ice_servers.length === 0) {
    throw new Error("The server returned no ICE servers. Configure STUN/TURN.");
  }

  const connection = new globalThis.RTCPeerConnection({ iceServers: config.ice_servers });
  connection.addEventListener("icecandidate", (event) => {
    if (event.candidate) {
      sendGroupCallSignal(remoteId, "ice_candidate", {
        group_id: groupCallGroupId,
        call_id: groupCallId,
        candidate: event.candidate.toJSON(),
      });
    }
  });
  connection.addEventListener("track", (event) => {
    const stream = event.streams[0] || new MediaStream([event.track]);
    renderGroupRemoteTrack(remoteId, stream, event.track.kind);
  });
  connection.addEventListener("connectionstatechange", () => {
    if (connection !== groupPeerConnections.get(remoteId)) return;
    const connected = [...groupPeerConnections.values()]
      .filter((peer) => peer.connectionState === "connected").length;
    if (connection.connectionState === "connected") {
      callStatus.textContent = "Group call · " + (connected + 1) + " participant(s) connected" + callMediaNotice;
    } else if (connection.connectionState === "failed" || connection.connectionState === "closed") {
      removeGroupPeer(remoteId);
    }
  });
  connection.addEventListener("iceconnectionstatechange", () => {
    if (connection !== groupPeerConnections.get(remoteId)) return;
    if (connection.iceConnectionState === "failed") {
      const name = users.find((user) => user.user_id === remoteId)?.display_name || "participant";
      callStatus.textContent = name + " could not connect" + callMediaNotice;
    }
  });
  groupPeerConnections.set(remoteId, connection);
  return connection;
}

async function createGroupOffer(remoteId) {
  if (groupPeerConnections.has(remoteId)) return groupPeerConnections.get(remoteId);
  const connection = await createGroupPeerConnection(remoteId);
  for (const track of localMediaStream?.getTracks() || []) {
    connection.addTrack(track, localMediaStream);
  }
  if (!localMediaStream?.getAudioTracks().length && connection.addTransceiver) {
    connection.addTransceiver("audio", { direction: "recvonly" });
  }
  applyCallCodecPreferences(connection);
  const offer = await connection.createOffer();
  await connection.setLocalDescription(offer);
  sendGroupCallSignal(remoteId, "offer", {
    group_id: groupCallGroupId,
    call_id: groupCallId,
    target_id: remoteId,
    description: connection.localDescription,
    media: callMediaKind,
  });
  return connection;
}

async function acceptGroupInvite(signal) {
  const payload = signal.payload || {};
  const groupId = payload.group_id;
  const callId = payload.call_id;
  if (!groupId || !callId) throw new Error("Group call invitation is malformed.");

  const group = groups.find((item) => item.user_id === groupId && item.is_group);
  if (!group) throw new Error("This group is no longer available.");

  stopCallRingtone();
  groupCallGroupId = groupId;
  groupCallId = callId;
  groupCallMemberIds = [...group.group_member_ids];
  groupCallInitiatorId = signal.sender_id;
  updateGroupCallControls();
  callPeerId = groupId;
  callMediaKind = payload.media === "video" ? "video" : "audio";
  callWindowTitle.textContent = "group call.exe";
  groupCallJoinedMembers.clear();
  groupCallJoinedMembers.add(me.user_id);
  groupCallJoinedMembers.add(signal.sender_id);
  refreshGroupCallParticipants();


  stopCallRingtone();
  try {
    localMediaStream = await acquireCallMedia(callMediaKind);
    showCallStage();
    callStage.classList.remove("call-collapsed");
    remoteVideo.hidden = true;
    remoteAudio.hidden = true;
    document.getElementById("group-remotes").hidden = false;
    await attachLocalMediaPreview();
    callStatus.textContent = "Joining group call…" + callMediaNotice;
    for (const memberId of groupCallMemberIds) {
      if (memberId !== me.user_id) {
        sendGroupCallSignal(memberId, "group_join", {
          group_id: groupCallGroupId,
          call_id: groupCallId,
        });
      }
    }
    await establishGroupOffers();
  } catch (err) {
    appendSystem("Could not join group call: " + (err.message || "Check camera and microphone permissions."));
    endCall(false);
  }
}

async function handleGroupCallSignal(signal) {
  const payload = signal.payload || {};
  const groupId = payload.group_id || signal.peer_id;
  const callId = payload.call_id;
  if (!groupId || !callId) return;

  const group = groups.find((item) => item.user_id === groupId && item.is_group);
  if (!group || !group.group_member_ids.includes(me.user_id)) return;

  if (signal.kind === "group_invite") {
    if (groupCallId === callId && groupCallGroupId === groupId) return;
    if (
      pendingIncomingCall?.payload?.group_id === groupId
      && pendingIncomingCall?.payload?.call_id === callId
      && incomingCallDialog.open
    ) return;
    pendingIncomingCall = signal;
    callPeerId = groupId;
    callMediaKind = payload.media === "video" ? "video" : "audio";
    const caller = users.find((user) => user.user_id === signal.sender_id);
    incomingCallTitle.textContent = (caller?.display_name || "Larptrix user") + " invited you";
    incomingCallKind.textContent = (callMediaKind === "video" ? "Group video" : "Group") + " call · " + group.display_name;
    document.getElementById("accept-call").textContent = "Join";
    startCallRingtone();
    showBrowserNotification(
      incomingCallTitle.textContent || "Incoming group call",
      incomingCallKind.textContent || "Group call",
      "group-call-" + groupId,
      group.group_avatar_url || group.avatar_url || null,
    );
    // Direct calls are handled from the chat header so the normal chat stays visible.
    return;
  }

  if (groupCallId !== callId || groupCallGroupId !== groupId) return;

  if (signal.kind === "group_join") {
    stopCallRingtone();
    const alreadyKnown = groupCallJoinedMembers.has(signal.sender_id);
    groupCallJoinedMembers.add(signal.sender_id);
    refreshGroupCallParticipants();
    if (!alreadyKnown) {
      sendGroupCallSignal(signal.sender_id, "group_join", {
        group_id: groupId,
        call_id: callId,
      });
      await establishGroupOffers();
    }
    return;
  }

  if (signal.kind === "offer") {
    let connection = groupPeerConnections.get(signal.sender_id);
    if (!connection) connection = await createGroupPeerConnection(signal.sender_id);
    if (!connection.currentRemoteDescription) {
      await connection.setRemoteDescription(signal.payload.description);
      for (const candidate of groupPendingIceCandidates.get(signal.sender_id) || []) {
        await connection.addIceCandidate(candidate);
      }
      groupPendingIceCandidates.delete(signal.sender_id);
    }
    for (const track of localMediaStream?.getTracks() || []) {
      if (!connection.getSenders().some((sender) => sender.track === track)) {
        connection.addTrack(track, localMediaStream);
      }
    }
    applyCallCodecPreferences(connection);
    const answer = await connection.createAnswer();
    await connection.setLocalDescription(answer);
    sendGroupCallSignal(signal.sender_id, "answer", {
      group_id: groupId,
      call_id: callId,
      target_id: signal.sender_id,
      description: connection.localDescription,
      media: callMediaKind,
    });
    return;
  }

  if (signal.kind === "answer") {
    const connection = groupPeerConnections.get(signal.sender_id);
    if (!connection) return;
    await connection.setRemoteDescription(signal.payload.description);
    for (const candidate of groupPendingIceCandidates.get(signal.sender_id) || []) {
      await connection.addIceCandidate(candidate);
    }
    groupPendingIceCandidates.delete(signal.sender_id);
    return;
  }

  if (signal.kind === "ice_candidate") {
    const candidate = signal.payload?.candidate;
    if (!candidate) return;
    const connection = groupPeerConnections.get(signal.sender_id);
    if (connection?.remoteDescription) await connection.addIceCandidate(candidate);
    else groupPendingIceCandidates.set(signal.sender_id, [
      ...(groupPendingIceCandidates.get(signal.sender_id) || []),
      candidate,
    ]);
    return;
  }

  if (signal.kind === "hangup" || signal.kind === "reject") {
    groupCallJoinedMembers.delete(signal.sender_id);
    refreshGroupCallParticipants();
    removeGroupPeer(signal.sender_id);
    if (signal.kind === "reject") {
      const name = users.find((user) => user.user_id === signal.sender_id)?.display_name || "Participant";
      callStatus.textContent = name + " declined the group call" + callMediaNotice;
    }
  }
}

function sendGroupCallSignal(targetId, kind, payload) {
  if (!groupCallGroupId || !groupCallId || !socket || socket.readyState !== WebSocket.OPEN) return;
  if (targetId !== me?.user_id) {
    const target = users.find((user) => user.user_id === targetId);
    if (target && !target.online) return;
  }
  socket.send(JSON.stringify({
    type: "call_signal",
    peer_id: groupCallGroupId,
    kind,
    payload: { ...payload, group_id: groupCallGroupId, call_id: groupCallId, target_id: targetId },
  }));
}

function renderGroupRemoteTrack(remoteId, stream, kind) {
  const container = document.getElementById("group-remotes");
  let tile = container.querySelector('[data-group-tile-id="' + CSS.escape(remoteId) + '"]');
  if (!tile) {
    tile = document.createElement("article");
    tile.className = "group-video-tile";
    tile.dataset.groupTileId = remoteId;

    const media = document.createElement("video");
    media.className = "group-remote-video";
    media.dataset.groupRemoteId = remoteId;
    media.dataset.kind = "video";
    media.autoplay = true;
    media.playsInline = true;
    tile.append(media);

    const audio = document.createElement("audio");
    audio.dataset.groupRemoteId = remoteId;
    audio.dataset.kind = "audio";
    audio.autoplay = true;
    audio.hidden = true;
    tile.append(audio);

    const identity = document.createElement("div");
    identity.className = "group-participant";
    const avatar = document.createElement("span");
    avatar.className = "avatar";
    avatar.dataset.groupAvatarId = remoteId;
    const user = users.find((item) => item.user_id === remoteId);
    paintAvatar(avatar, user || { display_name: "?" });
    const name = document.createElement("span");
    name.textContent = user?.display_name || "Participant";
    identity.append(avatar, name);
    tile.append(identity);
    container.append(tile);
  }

  const media = tile.querySelector('[data-kind="' + kind + '"]');
  if (media) {
    if (kind === "audio") {
      const volume = Number(localStorage.getItem("larptrix_call_audio_volume"));
      if (Number.isFinite(volume)) media.volume = Math.min(1, Math.max(0, volume));
      const audioStream = media.srcObject instanceof MediaStream
        ? media.srcObject
        : new MediaStream();
      for (const track of stream.getAudioTracks()) {
        if (!audioStream.getTracks().includes(track)) audioStream.addTrack(track);
      }
      media.srcObject = audioStream;
      void applyCallSpeakerDevice(selectedCallDeviceId("audiooutput"));
      startSpeakingMonitor(
        audioStream,
        "group-" + remoteId,
        tile.querySelector("[data-group-avatar-id=\"" + CSS.escape(remoteId) + "\"]"),
      );
      for (const track of stream.getAudioTracks()) {
        track.addEventListener("ended", () => {
          if (media.srcObject instanceof MediaStream && media.srcObject.getTracks().includes(track)) {
            media.srcObject.removeTrack(track);
          }
        }, { once: true });
      }
    } else {
      media.srcObject = stream;
    }
    media.play?.().catch(() => {});
    applyRemoteMuteStates();
  }
}


function removeGroupPeer(remoteId) {
  const connection = groupPeerConnections.get(remoteId);
  connection?.close();
  groupPeerConnections.delete(remoteId);
  groupPendingIceCandidates.delete(remoteId);
  stopSpeakingMonitor("group-" + remoteId);
  document.getElementById("group-remotes")
    ?.querySelectorAll('[data-group-remote-id="' + CSS.escape(remoteId) + '"]')
    .forEach((element) => element.remove());
  if (groupCallId && !pendingIncomingCall) {
    callStatus.textContent = groupCallJoinedMembers.size > 1
      ? "Group call · waiting for participants" + callMediaNotice
      : "Group call · waiting for participants to join" + callMediaNotice;
  }
}

async function startCall(kind) {
  if (!peerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  const selectedGroup = groups.find((item) => item.user_id === peerId && item.is_group);
  if (selectedGroup) {
    await startGroupCall(kind);
    return;
  }
  if (groupCallId) endCall(true);
  callWindowTitle.textContent = "call.exe";
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
  clearTimeout(lastDirectCallAvatarTimeout);
  lastDirectCallAvatarTimeout = null;
  clearTimeout(directCallNoticeTimeout);
  directCallNoticeTimeout = null;
  lastDirectCallPeerId = null;
  lastDirectCallAvatarPeerId = null;
  lastDirectCallJoinPeerId = null;
  lastDirectCallJoinKind = kind;
  directCallSessionId = crypto.randomUUID();
  directCallOutgoing = true;
  directCallStartedAt = Date.now();
  directCallAnsweredAt = 0;
  directCallAnswered = false;
  directCallOfferSent = false;
  callPeerId = peerId;
  callMediaKind = kind;
  callNoAnswer = false;
  updateDirectCallButtons(peerId);
  renderDirectCallAvatarStack();
  updateCallPlaceholder({ force: true });
  startOutgoingCallRingtone();
  try {
    localMediaStream = await acquireCallMedia(kind);
    showCallStage();
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
      call_id: directCallSessionId,
      description: peerConnection.localDescription,
      media: kind,
      started_at: directCallStartedAt,
    });
    directCallOfferSent = true;
    renderDirectCallTopbar(peerId);
    clearTimeout(outgoingCallTimeout);
    outgoingCallTimeout = setTimeout(() => {
      if (!peerConnection || callPeerId !== peerId) return;
      stopCallRingtone();
      callNoAnswer = true;
      lastDirectCallAvatarPeerId = me?.user_id || null;
      callStatus.textContent = "No answer";
      callAudioPlaceholder.hidden = true;
      callPlaceholderRemoteAvatar.replaceChildren();
      callPlaceholderRemoteName.textContent = "No answer";
      callPlaceholderRemoteName.classList.add("call-no-answer");
      renderDirectCallAvatarStack();
      updateDirectCallButtons(peerId);
      clearTimeout(noAnswerCleanupTimeout);
      noAnswerCleanupTimeout = setTimeout(() => {
        if (!callNoAnswer || callPeerId !== peerId) return;
        const unansweredPeerId = callPeerId;
        const unansweredKind = callMediaKind || "audio";
        sendCallSignal("hangup", {
          call_id: directCallSessionId,
          reason: "no_answer",
        });
        endCall(false);
        showDirectCallNotice(unansweredPeerId, "No answer", {
          join: false,
          persist: true,
          kind: unansweredKind,
          duration: 3000,
          avatarPeerId: me?.user_id,
        });
      }, 3000);
    }, 15000);
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
      updateCallPlaceholder();
      const volume = Number(localStorage.getItem("larptrix_call_audio_volume"));
      if (Number.isFinite(volume)) remoteAudio.volume = Math.min(1, Math.max(0, volume));
      const stream = remoteAudio.srcObject instanceof MediaStream
        ? remoteAudio.srcObject
        : new MediaStream();
      if (!stream.getTracks().includes(event.track)) stream.addTrack(event.track);
      remoteAudio.srcObject = stream;
      void applyCallSpeakerDevice(selectedCallDeviceId("audiooutput"));
      startSpeakingMonitor(stream, "remote", callPlaceholderRemoteAvatar);
      event.track.addEventListener("ended", () => {
        if (remoteAudio.srcObject instanceof MediaStream && remoteAudio.srcObject.getTracks().includes(event.track)) {
          remoteAudio.srcObject.removeTrack(event.track);
        }
      }, { once: true });
      remoteAudio.play().then(() => {
        enableCallAudio.hidden = true;
      }).catch(() => {
        enableCallAudio.hidden = false;
        callStatus.textContent = "Connected. Use Enable sound to hear the call.";
      });
    } else if (event.track.kind === "video") {
      const existing = remoteVideo.srcObject instanceof MediaStream ? remoteVideo.srcObject : null;
      let stream = event.streams[0] || null;
      if (!stream) {
        stream = new MediaStream();
        for (const track of existing?.getAudioTracks() || []) stream.addTrack(track);
        stream.addTrack(event.track);
      } else if (existing?.getVideoTracks().length && !existing.getVideoTracks().includes(event.track)) {
        stream = new MediaStream();
        stream.addTrack(event.track);
        for (const track of existing.getAudioTracks()) stream.addTrack(track);
      }
      remoteVideo.srcObject = stream;
      remoteVideo.hidden = false;
      callAudioPlaceholder.hidden = true;
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
    updateDirectCallButtons(peerId);
    callStatus.textContent = `${baseStatus}${callMediaNotice}`;
    if (connection.connectionState === "failed" || connection.connectionState === "closed") {
      if (directCallOutgoing && callPeerId && !callNoAnswer) {
        clearTimeout(outgoingCallTimeout);
        outgoingCallTimeout = null;
        clearTimeout(noAnswerCleanupTimeout);
        noAnswerCleanupTimeout = null;
        const unansweredPeerId = callPeerId;
        const unansweredKind = callMediaKind || "audio";
        endCall(false);
        showDirectCallNotice(unansweredPeerId, "No answer", {
          join: false,
          persist: true,
          kind: unansweredKind,
          duration: 3000,
          avatarPeerId: me?.user_id,
        });
        return;
      }
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
  if (signal?.payload?.group_id || groups.some((group) => group.user_id === signal?.peer_id && group.is_group)) {
    await handleGroupCallSignal(signal);
    return;
  }
  if (!me || signal.sender_id === me.user_id) return;
  const incomingCallId = signal.payload?.call_id || null;
  const now = Date.now();
  for (const [sessionId, expiresAt] of staleDirectCallSessions) {
    if (expiresAt <= now) staleDirectCallSessions.delete(sessionId);
  }
  if (incomingCallId && staleDirectCallSessions.has(incomingCallId)) return;
  if (signal.kind === "offer" && !peerConnection) {
    if (
      pendingIncomingCall
      && pendingIncomingCall.sender_id === signal.sender_id
      && pendingIncomingCall.payload?.call_id
      && incomingCallId
      && pendingIncomingCall.payload.call_id !== incomingCallId
    ) return;
    pendingIncomingCall = signal;
    directCallSessionId = incomingCallId;
    directCallOutgoing = false;
    directCallStartedAt = Number(signal.payload?.started_at) || Date.now();
    directCallAnsweredAt = 0;
    directCallAnswered = false;
    directCallOfferSent = true;
    callPeerId = signal.sender_id;
    renderDirectCallAvatarStack();
    pendingIceCandidates = iceCandidatesBeforeOffer.get(signal.sender_id) || [];
    iceCandidatesBeforeOffer.delete(signal.sender_id);
    callMediaKind = signal.payload.media === "video" ? "video" : "audio";
    const caller = users.find((user) => user.user_id === signal.sender_id);
    incomingCallTitle.textContent = `Call from ${caller?.display_name || "Larptrix user"}`;
    const requestedKind = callMediaKind === "video" ? "Video call" : "Voice call";
    callWindowTitle.textContent = "call.exe";
    incomingCallKind.textContent = typeof globalThis.RTCPeerConnection === "function"
      ? requestedKind
      : `${requestedKind} · open the browser client and ask the caller to retry`;
    document.getElementById("accept-call").textContent = typeof globalThis.RTCPeerConnection === "function"
      ? "Accept"
      : "Open browser";
    startCallRingtone();
    renderDirectCallTopbar(signal.sender_id);
    clearTimeout(incomingCallTimeout);
    incomingCallTimeout = setTimeout(() => {
      if (!pendingIncomingCall || pendingIncomingCall.sender_id !== signal.sender_id) return;
      const missedPeerId = signal.sender_id;
      const missedKind = callMediaKind || "audio";
      stopCallRingtone();
      incomingCallDialog.open && incomingCallDialog.close();
      pendingIncomingCall = null;
      callPeerId = null;
      showDirectCallNotice(missedPeerId, "No answer", {
        join: true,
        kind: missedKind,
        duration: 3000,
      });
    }, 20000);
    incomingCallDialog.showModal();
    return;
  }
  if (
    (signal.kind === "hangup" || signal.kind === "reject")
    && pendingIncomingCall?.sender_id === signal.sender_id
    && (!pendingIncomingCall.payload?.call_id || pendingIncomingCall.payload.call_id === incomingCallId)
  ) {
    clearTimeout(incomingCallTimeout);
    incomingCallTimeout = null;
    const missedPeerId = pendingIncomingCall.sender_id;
    const missedKind = callMediaKind || "audio";
    const wasNoAnswer = signal.kind === "hangup" && signal.payload?.reason === "no_answer";
    pendingIncomingCall = null;
    stopCallRingtone();
    if (wasNoAnswer) {
      callPeerId = null;
      showDirectCallNotice(missedPeerId, "No answer", {
        join: true,
        kind: missedKind,
        duration: 3000,
        avatarPeerId: missedPeerId,
      });
    } else {
      renderDirectCallAvatarStack();
      updateDirectCallButtons(peerId);
    }
    return;
  }

  if (signal.kind === "ice_candidate" && !peerConnection) {
    if (
      pendingIncomingCall
      && signal.sender_id === callPeerId
      && (!directCallSessionId || !incomingCallId || incomingCallId === directCallSessionId)
    ) {
      pendingIceCandidates.push(signal.payload);
    }

    // Ignore ICE candidates that arrive without a pending offer.
    // They may belong to a previous/ended ICE generation.
    return;
  }

  if (signal.sender_id !== callPeerId) return;
  if (!peerConnection) return;
  if (directCallSessionId && incomingCallId && incomingCallId !== directCallSessionId) return;
  if (signal.kind === "answer") {
    clearTimeout(outgoingCallTimeout);
    outgoingCallTimeout = null;
    clearTimeout(noAnswerCleanupTimeout);
    noAnswerCleanupTimeout = null;
    callNoAnswer = false;
    directCallAnswered = true;
    directCallAnsweredAt = Date.now();
    renderDirectCallAvatarStack();
    renderDirectCallTopbar(peerId);
    stopCallRingtone();
    await peerConnection.setRemoteDescription(signal.payload.description || signal.payload);
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
    sendCallSignal("answer", {
      call_id: directCallSessionId,
      description: peerConnection.localDescription,
    });
  } else if (signal.kind === "reject" || signal.kind === "hangup") {
    clearTimeout(outgoingCallTimeout);
    outgoingCallTimeout = null;
    endCall(false);
  }
}

async function acceptIncomingCall() {
  if (pendingIncomingCall?.payload?.group_id || groups.some((group) => group.user_id === pendingIncomingCall?.peer_id && group.is_group)) {
    const groupIncoming = pendingIncomingCall;
    pendingIncomingCall = null;
    incomingCallDialog.close();
    await acceptGroupInvite(groupIncoming);
    return;
  }
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
  clearTimeout(incomingCallTimeout);
  incomingCallTimeout = null;
  stopCallRingtone();
  incomingCallDialog.close();
  peerId = incoming.sender_id;
  callNoAnswer = false;
  directCallStartedAt = Number(incoming.payload?.started_at) || directCallStartedAt || Date.now();
  directCallAnsweredAt = Date.now();
  directCallAnswered = true;
  directCallOutgoing = false;
  lastDirectCallJoinPeerId = null;
  lastDirectCallJoinKind = callMediaKind || "audio";
  updateCallPlaceholder({ force: true });
  renderDirectCallAvatarStack();
  renderUsers();
  if (socket?.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "open", peer_id: peerId }));
  }
  try {
    localMediaStream = await acquireCallMedia(callMediaKind);
    showCallStage();
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
    sendCallSignal("answer", {
      call_id: directCallSessionId,
      description: peerConnection.localDescription,
    });
  } catch (err) {
    appendSystem(err.message || "Could not accept the call. Check camera and microphone permissions.");
    sendCallSignal("reject", { call_id: directCallSessionId });
    endCall(false);
  }
}

function rejectIncomingCall() {
  clearTimeout(incomingCallTimeout);
  incomingCallTimeout = null;
  stopCallRingtone();
  if (pendingIncomingCall?.payload?.group_id) {
    const incoming = pendingIncomingCall;
    const groupId = incoming.payload.group_id;
    const callId = incoming.payload.call_id;
    const target = incoming.sender_id;
    pendingIncomingCall = null;
    incomingCallDialog.close();
    groupCallGroupId = groupId;
    groupCallId = callId;
    callPeerId = groupId;
    sendGroupCallSignal(target, "reject", { group_id: groupId, call_id: callId });
    groupCallGroupId = null;
    groupCallId = null;
    callPeerId = null;
    return;
  }
  if (pendingIncomingCall) {
    const rejectedPeerId = pendingIncomingCall.sender_id;
    callPeerId = pendingIncomingCall.sender_id;
    directCallSessionId = pendingIncomingCall.payload?.call_id || directCallSessionId;
    sendCallSignal("reject", { call_id: directCallSessionId });
    iceCandidatesBeforeOffer.delete(rejectedPeerId);
  }
  pendingIncomingCall = null;
  incomingCallDialog.close();
  callPeerId = null;
  renderDirectCallTopbar(peerId);
}

async function flushIceCandidates() {
  const candidates = pendingIceCandidates;
  pendingIceCandidates = [];
  for (const candidate of candidates) await peerConnection.addIceCandidate(candidate);
}

function callAudioConstraints() {
  const supported = navigator.mediaDevices?.getSupportedConstraints?.() || {};
  return {
    ...(supported.echoCancellation ? { echoCancellation: true } : {}),
    ...(supported.noiseSuppression ? { noiseSuppression: readStoredBool(NOISE_SUPPRESSION_KEY, true) } : {}),
    ...(supported.autoGainControl ? { autoGainControl: true } : {}),
  };
}

async function applyCallSpeakerDevice(deviceId) {
  if (!deviceId) return;
  const audioElements = [remoteAudio, ...document.querySelectorAll("#group-remotes audio[data-kind=\"audio\"]")];
  for (const audio of audioElements) {
    if (typeof audio?.setSinkId !== "function") continue;
    try {
      await audio.setSinkId(deviceId);
    } catch {}
  }
}

function selectedCallDeviceId(kind) {
  if (kind === "audioinput") return localStorage.getItem(CALL_MIC_KEY) || "";
  if (kind === "videoinput") return localStorage.getItem(CALL_CAMERA_KEY) || "";
  if (kind === "audiooutput") return localStorage.getItem(CALL_SPEAKERS_KEY) || "";
  return "";
}

async function populateCallDeviceSelects() {
  if (!navigator.mediaDevices?.enumerateDevices) return;
  try {
    const devices = await navigator.mediaDevices.enumerateDevices();
    const fill = (select, kind, defaultLabel) => {
      if (!select) return;
      const previous = select.value || selectedCallDeviceId(kind);
      select.replaceChildren();
      const def = document.createElement("option");
      def.value = "";
      def.textContent = defaultLabel;
      select.append(def);
      for (const device of devices.filter((item) => item.kind === kind)) {
        if (!device.deviceId) continue;
        const option = document.createElement("option");
        option.value = device.deviceId;
        option.textContent = device.label || `${defaultLabel} ${option.index}`;
        select.append(option);
      }
      select.value = [...select.options].some((option) => option.value === previous) ? previous : "";
    };
    fill(profileMicrophone, "audioinput", "Default microphone");
    fill(profileSpeakers, "audiooutput", "Default speakers");
    fill(profileCamera, "videoinput", "Default camera");
    fill(callSettingsMicrophone, "audioinput", "Default microphone");
    fill(callSettingsSpeakers, "audiooutput", "Default speakers");
    fill(callSettingsCamera, "videoinput", "Default camera");
    if (profileDevicesStatus) {
      profileDevicesStatus.textContent = devices.some((device) => device.label)
        ? "Choose the devices Larptrix should use for calls."
        : "Device names appear after the browser grants microphone/camera permission.";
    }
  } catch (err) {
    if (profileDevicesStatus) profileDevicesStatus.textContent = err.message || "Could not enumerate call devices.";
  }
}

async function replaceCallVideoTrack() {
  const oldTrack = localMediaStream?.getVideoTracks()[0];
  if (!oldTrack || !navigator.mediaDevices?.getUserMedia) return;
  const deviceId = selectedCallDeviceId("videoinput");
  const stream = await navigator.mediaDevices.getUserMedia({
    audio: false,
    video: deviceId ? { deviceId: { exact: deviceId }, width: { ideal: 1280 }, height: { ideal: 720 } } : true,
  });
  const newTrack = stream.getVideoTracks()[0];
  if (!newTrack) throw new Error("Camera unavailable.");
  const connections = groupCallId ? [...groupPeerConnections.values()] : peerConnection ? [peerConnection] : [];
  for (const connection of connections) {
    const sender = connection.getSenders().find((item) => item.track?.kind === "video");
    if (sender) await sender.replaceTrack(newTrack);
  }
  oldTrack.stop();
  localMediaStream.removeTrack(oldTrack);
  localMediaStream.addTrack(newTrack);
  localVideo.srcObject = localMediaStream;
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
  const cameraId = selectedCallDeviceId("videoinput");
  const preferredVideo = cameraId
    ? { deviceId: { exact: cameraId }, width: { ideal: 1280 }, height: { ideal: 720 } }
    : { width: { ideal: 1280 }, height: { ideal: 720 } };
  const videoAttempts = kind === "video"
    ? [preferredVideo, true]
    : [false];
  const attempts = [];
  if (hasMicrophone) {
    // Prefer the browser's native audio profile first. Some Linux/PipeWire
    // devices reject processing constraints even though plain microphone
    // capture works perfectly.
    const micId = selectedCallDeviceId("audioinput");
    const audioConstraint = micId ? { deviceId: { exact: micId } } : true;
    for (const video of videoAttempts) attempts.push({ audio: audioConstraint, video });

    const baseAudio = callAudioConstraints();
    const processedAudioConstraint = micId ? { ...baseAudio, deviceId: { exact: micId } } : baseAudio;
    for (const video of videoAttempts) attempts.push({ audio: processedAudioConstraint, video });
    if (kind === "video") {
      attempts.push({ audio: true, video: false });
      attempts.push({ audio: callAudioConstraints(), video: false });
    }
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
      void populateCallDeviceSelects();
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

function sendCallSignal(kind, payload = {}) {
  if (!callPeerId || !socket || socket.readyState !== WebSocket.OPEN) return;
  const nextPayload = { ...payload };
  if (directCallSessionId && !nextPayload.call_id) {
    nextPayload.call_id = directCallSessionId;
  }
  socket.send(JSON.stringify({
    type: "call_signal",
    peer_id: callPeerId,
    kind,
    payload: nextPayload,
  }));
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
  if (!callPeerId) return;
  if (screenMediaStream) {
    await stopScreenShare();
    return;
  }
  try {
    const resolution = {
      "720p": { width: 1280, height: 720 },
      "1080p": { width: 1920, height: 1080 },
      "1440p": { width: 2560, height: 1440 },
      "4k": { width: 3840, height: 2160 },
    }[screenResolution?.value || "1080p"] || { width: 1920, height: 1080 };
    const frameRate = Number(screenFrameRate?.value || 60);
    const webkitGtk = navigator.platform.toLowerCase().includes("linux")
      && navigator.userAgent.includes("AppleWebKit")
      && !/(Chrome|Chromium)/.test(navigator.userAgent);
    if (!navigator.mediaDevices?.getDisplayMedia) {
      throw new Error("Screen capture is not supported by this desktop runtime.");
    }
    const audioMode = document.getElementById("screen-audio-mode")?.value || "none";
    const firefox = /Firefox\//.test(navigator.userAgent) && !/Seamonkey\//.test(navigator.userAgent);
    const captureAudio = audioMode !== "none" && !webkitGtk && !firefox;
    const supported = navigator.mediaDevices.getSupportedConstraints?.() || {};
    const audioConstraints = captureAudio
      ? {
          suppressLocalAudioPlayback: true,
          ...(supported.restrictOwnAudio ? { restrictOwnAudio: true } : {}),
        }
      : false;
    // Always keep a final, completely unconstrained capture fallback.
    // Chromium/WebView builds get the rich options first (for audio/quality);
    // Firefox gets only the minimal form because several capture hints are
    // rejected by some Firefox + PipeWire combinations.
    // Start with the browser's simplest valid screen-capture request.
    // This avoids InvalidStateError/TypeError combinations from optional
    // capture hints and leaves quality selection to the selected track.
    // Use the least restrictive request first. Browser-specific display
    // capture hints are intentionally avoided here because they can trigger
    // "Invalid capture constraints" before the source picker even opens.
    const captureOptions = [{ video: true }];
    let captureError = null;
    for (const options of captureOptions) {
      try {
        screenMediaStream = await navigator.mediaDevices.getDisplayMedia(options);
        break;
      } catch (err) {
        captureError = err;
      }
    }
    if (!screenMediaStream) {
      const detail = captureError?.message || captureError?.name || "unknown capture error";
      throw new Error("Screen capture failed (" + detail + ").");
    }
    const screenTrack = screenMediaStream.getVideoTracks()[0];
    if (!screenTrack) throw new Error("The selected share source has no video track.");
    if (!firefox) {
      try {
        await screenTrack.applyConstraints({
          width: { ideal: resolution.width },
          height: { ideal: resolution.height },
          frameRate: { ideal: frameRate },
        });
      } catch {}
    }
    const actual = screenTrack.getSettings?.() || {};
    const actualWidth = actual.width || resolution.width;
    const actualHeight = actual.height || resolution.height;
    const actualFps = actual.frameRate ? Math.round(actual.frameRate) : frameRate;
    const audioTracks = screenMediaStream.getAudioTracks();
    callMediaNotice = audioTracks.length
      ? ` · sharing ${actualWidth}×${actualHeight} @ ${actualFps} fps + audio`
      : ` · sharing ${actualWidth}×${actualHeight} @ ${actualFps} fps`;
    callStage.classList.add("screen-sharing");
    callAudioPlaceholder.hidden = true;
    localScreenVideo.srcObject = screenMediaStream;
    localScreenVideo.muted = true;
    localScreenVideo.defaultMuted = true;
    localScreenVideo.hidden = false;
    localScreenVideo.play().catch(() => {});

    const connections = groupCallId
      ? [...groupPeerConnections.entries()]
      : peerConnection ? [["direct", peerConnection]] : [];
    let sharedAudio = false;

    for (const [remoteId, connection] of connections) {
      let sender = connection.getSenders().find((item) =>
        item.track?.kind === "video" || item.track === localMediaStream?.getVideoTracks()[0]
      );
      let renegotiate = false;
      if (sender) {
        await sender.replaceTrack(screenTrack);
      } else {
        sender = connection.addTrack(screenTrack, screenMediaStream);
        renegotiate = true;
      }
      const screenAudioTrack = screenMediaStream.getAudioTracks()[0];
      if (screenAudioTrack && !connection.getSenders().some((item) => item.track === screenAudioTrack)) {
        connection.addTrack(screenAudioTrack, screenMediaStream);
        renegotiate = true;
        sharedAudio = true;
      }
      try {
        const params = sender.getParameters();
        params.encodings = params.encodings?.length ? params.encodings : [{}];
        params.encodings[0].maxBitrate = Math.max(3_000_000, Math.min(24_000_000, Math.round(resolution.width * resolution.height * frameRate * 0.012)));
        params.encodings[0].maxFramerate = frameRate;
        await sender.setParameters(params);
      } catch {}
      if (renegotiate) {
        applyCallCodecPreferences(connection);
        const offer = await connection.createOffer();
        await connection.setLocalDescription(offer);
        if (groupCallId) {
          sendGroupCallSignal(remoteId, "offer", {
            group_id: groupCallGroupId,
            call_id: groupCallId,
            target_id: remoteId,
            description: connection.localDescription,
            media: callMediaKind,
          });
        } else {
          sendCallSignal("offer", { description: connection.localDescription, media: callMediaKind });
        }
      }
    }

    const settings = screenTrack.getSettings();
    const audioStatus = sharedAudio
      ? " with shared audio"
      : webkitGtk
        ? " (screen audio unavailable in WebKitGTK)"
        : firefox && audioMode !== "none"
          ? " (Firefox does not provide screen audio)"
          : " (source audio unavailable)";
    callStatus.textContent = `Sharing ${settings.width || "?"}×${settings.height || "?"} at ${Math.round(settings.frameRate || 0)} fps${audioStatus}`;
    screenTrack.addEventListener("ended", stopScreenShare, { once: true });
    document.getElementById("toggle-screen-share").textContent = "Stop sharing";
  } catch (err) {
    if (err.name !== "NotAllowedError") {
      appendSystem(
        err.name === "TypeError" || /Invalid capture constraints/i.test(err.message || "")
          ? "Screen sharing was rejected by the browser. Try selecting Share screen again; Firefox is using a compatibility capture mode."
          : err.message || "Could not start screen sharing.",
      );
    }
    screenMediaStream = null;
  }
}

async function stopScreenShare() {
  if (!screenMediaStream) return;
  callStage.classList.remove("screen-sharing");
  const screenTrack = screenMediaStream.getVideoTracks()[0];
  const cameraTrack = localMediaStream?.getVideoTracks()[0];
  const connections = groupCallId
    ? [...groupPeerConnections.entries()]
    : peerConnection ? [["direct", peerConnection]] : [];

  for (const [remoteId, connection] of connections) {
    const sender = connection.getSenders().find((item) => item.track === screenTrack);
    if (sender) await sender.replaceTrack(cameraTrack || null);
    for (const audioTrack of screenMediaStream.getAudioTracks()) {
      const audioSender = connection.getSenders().find((item) => item.track === audioTrack);
      if (audioSender) await audioSender.replaceTrack(null);
    }
  }

  screenMediaStream.getTracks().forEach((track) => track.stop());
  screenMediaStream = null;
  localScreenVideo.srcObject = null;
  localScreenVideo.muted = true;
  localScreenVideo.hidden = true;
  updateCallPlaceholder({ force: !localMediaStream?.getVideoTracks().length });
  document.getElementById("toggle-screen-share").textContent = "Share screen";
  if (callStatus.textContent.startsWith("Sharing ")) callStatus.textContent = "Connected";
}


function endCall(notifyPeer) {
  const preserveNoAnswerAvatar = Boolean(callNoAnswer && callPeerId);
  const endedDirectPeerId = callPeerId;
  const endedDirectCallKind = callMediaKind || "audio";
  clearTimeout(outgoingCallTimeout);
  outgoingCallTimeout = null;
  clearTimeout(incomingCallTimeout);
  incomingCallTimeout = null;
  clearTimeout(noAnswerCleanupTimeout);
  noAnswerCleanupTimeout = null;
  clearTimeout(directCallNoticeTimeout);
  directCallNoticeTimeout = null;
  stopCallRingtone();
  stopSpeakingMonitor("local");
  stopSpeakingMonitor("remote");
  for (const key of [...speakingMonitors.keys()].filter((item) => item.startsWith("group-"))) {
    stopSpeakingMonitor(key);
  }

  const endedDirectSessionId = directCallSessionId;
  const endedDirectOfferSent = directCallOfferSent;
  const endedDirectStartedAt = directCallStartedAt;
  const endedDirectAnsweredAt = directCallAnsweredAt;
  const endedDirectAnswered = directCallAnswered;
  const endedDirectOutgoing = directCallOutgoing;
  if (
    endedDirectPeerId
    && endedDirectSessionId
    && endedDirectOfferSent
    && endedDirectOutgoing
  ) {
    void sendDirectCallLog(
      endedDirectPeerId,
      endedDirectCallKind,
      endedDirectAnswered,
      endedDirectStartedAt,
      endedDirectAnsweredAt,
      Date.now(),
      endedDirectSessionId,
    );
  }

  if (groupCallGroupId && groupCallId) {
    const groupId = groupCallGroupId;
    const callId = groupCallId;
    if (notifyPeer) {
      sendGroupCallControl("group_leave");
      const state = activeGroupCalls.get(groupId);
      if (state) {
        state.participant_ids = state.participant_ids.filter((id) => id !== me?.user_id);
        activeGroupCalls.set(groupId, state);
      }
    }

    for (const [remoteId, connection] of groupPeerConnections) {
      connection.close();
      document.getElementById("group-remotes")
        ?.querySelectorAll("[data-group-remote-id=\"" + CSS.escape(remoteId) + "\"]")
        .forEach((element) => element.remove());
    }
    groupPeerConnections.clear();
    groupPendingIceCandidates.clear();
    groupCallJoinedMembers.clear();
    refreshGroupCallParticipants();
    groupCallId = null;
    groupCallGroupId = null;
    groupCallMemberIds = [];
    groupCallInitiatorId = null;
    renderGroupCallBanner();
  } else if (notifyPeer && callPeerId) {
    sendCallSignal("hangup", {});
  }

  if (callSettingsPanel) callSettingsPanel.hidden = true;
  callDeafened = false;
  mutedRemoteUserIds.clear();
  applyRemoteMuteStates();
  if (callDeafenButton) {
    callDeafenButton.textContent = "🔊 Deafen";
    callDeafenButton.setAttribute("aria-pressed", "false");
  }
  if (incomingCallDialog.open) incomingCallDialog.close();
  screenMediaStream?.getTracks().forEach((track) => track.stop());
  localMediaStream?.getTracks().forEach((track) => track.stop());
  peerConnection?.close();
  peerConnection = null;
  localMediaStream = null;
  screenMediaStream = null;
  pendingIncomingCall = null;
  pendingIceCandidates = [];
  if (endedDirectPeerId) iceCandidatesBeforeOffer.delete(endedDirectPeerId);
  if (directCallSessionId) {
    staleDirectCallSessions.set(directCallSessionId, Date.now() + 30000);
  }
  callPeerId = null;
  callMediaKind = null;
  directCallSessionId = null;
  directCallOutgoing = false;
  directCallStartedAt = 0;
  directCallAnsweredAt = 0;
  directCallAnswered = false;
  directCallOfferSent = false;
  callNoAnswer = false;

  if (preserveNoAnswerAvatar && endedDirectPeerId) {
    lastDirectCallJoinPeerId = endedDirectPeerId;
    lastDirectCallJoinKind = endedDirectCallKind;
    lastDirectCallPeerId = endedDirectPeerId;
    lastDirectCallAvatarPeerId = me?.user_id || endedDirectPeerId;
    clearTimeout(lastDirectCallAvatarTimeout);
    lastDirectCallAvatarTimeout = setTimeout(() => {
      if (lastDirectCallPeerId === endedDirectPeerId) {
        lastDirectCallPeerId = null;
        lastDirectCallAvatarPeerId = null;
        lastDirectCallAvatarTimeout = null;
        renderDirectCallAvatarStack();
      }
    }, 8000);
  } else {
    clearTimeout(lastDirectCallAvatarTimeout);
    lastDirectCallAvatarTimeout = null;
    lastDirectCallPeerId = null;
    lastDirectCallAvatarPeerId = null;
    lastDirectCallJoinPeerId = null;
    lastDirectCallJoinKind = "audio";
  }

  updateDirectCallButtons(peerId);
  updateGroupCallControls();
  renderDirectCallAvatarStack();
  localVideo.srcObject = null;
  localScreenVideo.srcObject = null;
  localScreenVideo.hidden = true;
  callAudioPlaceholder.hidden = true;
  callStage.classList.remove("screen-sharing");
  remoteVideo.srcObject = null;
  remoteVideo.hidden = false;
  remoteAudio.srcObject = null;
  remoteAudio.hidden = false;
  document.getElementById("group-remotes").replaceChildren();
  document.getElementById("group-remotes").hidden = true;
  document.getElementById("toggle-screen-share").hidden = false;
  enableCallAudio.hidden = true;
  callStage.hidden = true;
  callStage.classList.remove("call-collapsed");
  callStage.classList.remove("call-ending-notice");
  renderDirectCallParticipants();
  const collapseButton = document.getElementById("call-collapse");
  collapseButton.textContent = "−";
  collapseButton.title = "Minimize call";
  document.getElementById("toggle-microphone").textContent = "🎙 Mute mic";
  document.getElementById("toggle-microphone").setAttribute("aria-pressed", "false");
  document.getElementById("toggle-camera").textContent = "📷 Turn camera off";
  document.getElementById("toggle-screen-share").textContent = "Share screen";
}
function isForOpenChat(message) {
  if (peerId === SAVED_MESSAGES_ID) return false;
  if (!peerId || !me) return false;
  const group = groups.find((item) => item.user_id === peerId);
  if (group?.is_group) return message.recipient_id === peerId;
  return (
    (message.sender_id === me.user_id && message.recipient_id === peerId) ||
    (message.sender_id === peerId && message.recipient_id === me.user_id)
  );
}

function setStatus(text) {
  if (text === "online") {
    renderPresenceStatus(localStorage.getItem(PRESENCE_KEY) || "online");
    return;
  }
  renderPresenceStatus(text);
}

function renderMe() {
  if (!me) return;
  profileOpen.hidden = false;
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

function primaryUserTag(user) {
  const tags = Array.isArray(user?.tags) ? user.tags.filter(Boolean) : [];
  return tags[0] || "";
}

function appendUserTag(parent, user, { compact = false } = {}) {
  const tag = primaryUserTag(user);
  if (!tag) return;
  const el = document.createElement("small");
  el.className = compact ? "user-tag user-tag-compact" : "user-tag";
  el.textContent = "🏷️ " + tag;
  el.title = tagsTitle(user);
  parent.append(el);
}

function tagsTitle(user) {
  return (Array.isArray(user?.tags) ? user.tags : []).map((tag) => "🏷️ " + tag).join(" · ");
}

function renderUsers() {
  if (peopleView === "requests") {
    renderFriendRequests();
    return;
  }
  usersEl.replaceChildren();
  const query = normalizePeopleSearch(userSearchInput.value);
  const base = query
    ? [...users.filter((user) => user.display_name.toLocaleLowerCase().includes(query)
        || (user.username || "").toLocaleLowerCase().includes(query)),
      ...searchResults]
    : [savedChatEntry(), ...friendRequests.friends, ...groups];
  const dedupe = new Map();
  for (const user of base) {
    if (!user || user.user_id === me?.user_id || dedupe.has(user.user_id)) continue;
    dedupe.set(user.user_id, user);
  }
  const matches = [...dedupe.values()];
  const pinned = getPinnedChats();
  matches.sort((a, b) =>
    (query ? searchScore(a, query) - searchScore(b, query) : 0)
    || (a.is_saved_chat ? -2 : 0) - (b.is_saved_chat ? -2 : 0)
    || (a.friend_status === "accepted" ? -1 : 0) - (b.friend_status === "accepted" ? -1 : 0)
    || (pinned.has(b.user_id) ? 1 : 0) - (pinned.has(a.user_id) ? 1 : 0)
    || a.display_name.localeCompare(b.display_name)
  );

  if (query && friendSearchLoading) {
    const loading = document.createElement("li");
    loading.className = "search-empty";
    loading.textContent = "Searching…";
    usersEl.append(loading);
    return;
  }

  if (query && !matches.length) {
    const empty = document.createElement("li");
    empty.className = "search-empty";
    empty.textContent = "No users found.";
    usersEl.append(empty);
    return;
  }

  for (const user of matches) {
    const li = document.createElement("li");
    li.className = user.friend_status && user.friend_status !== "accepted" ? "search-person" : "";
    li.classList.toggle("chat-active", user.user_id === peerId);

    const button = document.createElement("button");
    button.type = "button";
    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user);
    const dot = document.createElement("span");
    const presence = user.is_saved_chat ? "saved" : getPresence(user);
    dot.className = user.is_saved_chat ? "dot saved-dot" : presence === "online" ? "dot on" : presence === "dnd" ? "dot dnd" : "dot";
    dot.title = user.is_saved_chat ? "Saved Messages" : presence;

    const name = document.createElement("span");
    name.className = "person-name";
    const label = document.createElement("span");
    label.textContent = user.is_saved_chat ? "Saved Messages" : user.is_group ? "👥 " + user.display_name : user.display_name;
    appendUserTag(label, user, { compact: true });
    name.append(label);
    if (!user.is_saved_chat && user.username && !user.is_group) {
      const handle = document.createElement("small");
      handle.textContent = "@" + user.username;
      name.append(handle);
    }
    if (!user.is_saved_chat && !user.is_group) {
      renderActivityEntries(name, user, { compact: true, limit: 2 });
    }
    button.append(avatar, name, dot);

    if (!query || user.is_saved_chat || user.is_group || user.friend_status === "accepted") {
      button.addEventListener("click", () => openChat(user.user_id));
    } else {
      button.classList.add("search-result-button");
      button.addEventListener("click", () => openChat(user.user_id));
    }

    if (query && !user.is_saved_chat && !user.is_group && user.friend_status !== "accepted") {
      const relation = document.createElement("button");
      relation.type = "button";
      relation.className = "chat-friend-action ghost";
      if (user.friend_status === "pending_incoming") {
        relation.textContent = "Accept";
        relation.title = "Accept friend request";
        relation.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          try {
            await api("POST", "/api/friends/" + encodeURIComponent(user.user_id) + "/accept", {});
            await Promise.all([refreshFriendSearch(), loadFriendRequests()]);
          } catch (err) {
            appendSystem(err.message || "Could not accept friend request.");
          }
        });
      } else if (user.friend_status === "pending_outgoing") {
        relation.textContent = "Requested";
        relation.disabled = true;
        relation.title = "Friend request already sent";
      } else {
        relation.textContent = "Add";
        relation.title = "Add to friends";
        relation.addEventListener("click", async (event) => {
          event.preventDefault();
          event.stopPropagation();
          relation.disabled = true;
          try {
            await api("POST", "/api/friends/" + encodeURIComponent(user.user_id), {});
            await Promise.all([refreshFriendSearch(), loadFriendRequests()]);
          } catch (err) {
            relation.disabled = false;
            appendSystem(err.message || "Could not add friend.");
          }
        });
      }
      li.append(button, relation);
      usersEl.append(li);
      continue;
    }

    const pin = document.createElement("button");
    pin.type = "button";
    pin.className = "chat-pin ghost";
    pin.title = pinned.has(user.user_id) ? "Unpin chat" : "Pin chat";
    pin.textContent = pinned.has(user.user_id) ? "★" : "☆";
    pin.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      togglePinnedChat(user.user_id);
    });
    li.append(button, pin);
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
      is_channel: Boolean(group.is_channel),
      admin_ids: Array.isArray(group.admin_ids) ? group.admin_ids : [],
      post_policy: group.post_policy || (group.is_channel ? "admins" : "members"),
      group_member_ids: Array.isArray(group.member_ids) ? group.member_ids : [],
      group_description: group.description || "",
      group_avatar_url: group.avatar_url
        ? group.avatar_url + (group.avatar_url.includes("?") ? "&" : "?") + "v=" + Date.now()
        : null,
      group_banner_url: group.banner_url
        ? group.banner_url + (group.banner_url.includes("?") ? "&" : "?") + "v=" + Date.now()
        : null,
      avatar_url: group.avatar_url
        ? group.avatar_url + (group.avatar_url.includes("?") ? "&" : "?") + "v=" + Date.now()
        : null,
      banner_url: group.banner_url
        ? group.banner_url + (group.banner_url.includes("?") ? "&" : "?") + "v=" + Date.now()
        : null,
      subscriber_count: Number(group.subscriber_count || group.member_ids?.length || 0),
    }));
    renderUsers();
  } catch (err) {
    appendSystem(`Could not load groups: ${err.message}`);
  }
}

function renderGroupMemberChoices() {
  groupMemberList.replaceChildren();
  const friendIds = new Set(
    friendRequests.friends.map((friend) => friend.user_id)
  );
  for (const user of users.filter(
    (item) => friendIds.has(item.user_id) && item.user_id !== me?.user_id && !item.is_group
  )) {
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
      is_channel: Boolean(created.is_channel),
      admin_ids: Array.isArray(created.admin_ids) ? created.admin_ids : [me.user_id],
      post_policy: created.post_policy || "members",
      group_member_ids: created.member_ids || [],
      group_description: created.description || "",
      group_avatar_url: created.avatar_url || null,
      group_banner_url: created.banner_url || null,
      avatar_url: created.avatar_url || null,
      banner_url: created.banner_url || null,
      subscriber_count: Number(created.subscriber_count || created.member_ids?.length || 0),
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

async function createChannel(event) {
  event.preventDefault();
  const error = document.getElementById("channel-create-error");
  error.hidden = true;
  const memberIds = [...channelMemberList.querySelectorAll("input:checked")].map((input) => input.value);
  try {
    const created = await api("POST", "/api/channels", {
      name: document.getElementById("channel-name").value.trim(),
      member_ids: memberIds,
      post_policy: document.getElementById("channel-post-policy").value,
    });
    const newChannel = {
      user_id: created.group_id,
      display_name: created.name,
      online: true,
      is_group: true,
      is_channel: true,
      admin_ids: Array.isArray(created.admin_ids) ? created.admin_ids : [me.user_id],
      post_policy: created.post_policy || "admins",
      group_member_ids: created.member_ids || [],
      group_description: created.description || "",
      group_avatar_url: created.avatar_url || null,
      group_banner_url: created.banner_url || null,
      avatar_url: created.avatar_url || null,
      banner_url: created.banner_url || null,
      subscriber_count: Number(created.subscriber_count || created.member_ids?.length || 0),
    };
    groups = [...groups.filter((group) => group.user_id !== newChannel.user_id), newChannel];
    createChannelDialog.close();
    renderUsers();
    openChat(created.group_id);
  } catch (err) {
    error.textContent = err.message;
    error.hidden = false;
  }
}

function renderChannelMemberChoices() {
  channelMemberList.replaceChildren();
  const friendIds = new Set(
    friendRequests.friends.map((friend) => friend.user_id)
  );
  for (const user of users.filter(
    (item) => friendIds.has(item.user_id) && item.user_id !== me?.user_id && !item.is_group
  )) {
    const label = document.createElement("label");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.value = user.user_id;
    checkbox.disabled = !user.e2e_enabled;
    const name = document.createElement("span");
    name.textContent = user.e2e_enabled ? user.display_name + " · E2E ready" : user.display_name + " · E2E required";
    label.append(checkbox, name);
    channelMemberList.append(label);
  }
}
function paintAvatar(el, user) {
  el.replaceChildren();
  if (user?.is_saved_chat) {
    el.classList.add("saved-avatar");
    el.classList.remove("emoji-avatar");
    el.textContent = "★";
    return;
  }
  if (user?.avatar_url) {
    el.classList.remove("saved-avatar", "emoji-avatar");
    const img = document.createElement("img");
    const cacheKey = user.avatar_id || user.avatar_version || user.updated_at || Date.now();
    img.src = user.avatar_url + (user.avatar_url.includes("?") ? "&" : "?") + "v=" + encodeURIComponent(cacheKey);
    img.alt = "";
    el.append(img);
    return;
  }
  if (user?.is_channel) {
    el.classList.add("emoji-avatar");
    el.classList.remove("saved-avatar");
    el.textContent = "#";
    return;
  }
  el.classList.toggle("emoji-avatar", !user.avatar_url);
  if (user.avatar_url) {
    const img = document.createElement("img");
    const cacheKey = user.avatar_id || user.updated_at || user.avatar_version || Date.now();
    img.src = user.avatar_url + (user.avatar_url.includes("?") ? "&" : "?") + "v=" + encodeURIComponent(cacheKey);
    img.alt = "";
    el.append(img);
  } else {
    const faces = ["🐸", "🦊", "🐙", "🐟", "🦉", "🐧", "🐢", "🦋"];
    const seed = [...(user.user_id || user.display_name || "")].reduce((v, ch) => v + ch.charCodeAt(0), 0);
    el.classList.add("emoji-avatar");
    el.textContent = faces[seed % faces.length];
  }
}

const commonReactionEmojis = ["👍", "❤️", "😂", "😮", "😢", "🔥"];

function sendMessageReaction(message, emoji, add = true) {
  if (!message?.id || !socket || socket.readyState !== WebSocket.OPEN || !peerId) return;
  socket.send(JSON.stringify({
    type: "react",
    peer_id: peerId,
    message_id: message.id,
    emoji,
    add,
  }));
}

function renderMessageReactions(message, li) {
  if (!li || !message?.id) return;
  let bar = li.querySelector(".message-reactions");
  if (!bar) {
    bar = document.createElement("div");
    bar.className = "message-reactions";
    li.append(bar);
  }
  bar.replaceChildren();

  for (const reaction of (message.reactions || [])) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "reaction-chip";
    button.classList.toggle("reacted", Boolean(reaction.reacted));
    button.textContent = `${reaction.emoji} ${reaction.count}`;
    button.title = reaction.reacted ? "Remove reaction" : "React with " + reaction.emoji;
    button.addEventListener("click", () => {
      sendMessageReaction(message, reaction.emoji, !reaction.reacted);
    });
    bar.append(button);
  }

  if (message.is_channel || groups.some((item) => item.user_id === peerId && item.is_channel)) {
    const viewCount = document.createElement("span");
    viewCount.className = "message-view-count";
    viewCount.textContent = `◉ ${message.view_count || 0}`;
    viewCount.title = "Unique subscribers who viewed this message";
    bar.append(viewCount);
  }
}

function createReactionPicker(message) {
  const wrap = document.createElement("span");
  wrap.className = "reaction-picker-wrap";

  const open = document.createElement("button");
  open.type = "button";
  open.className = "ghost reaction-open";
  open.textContent = "☺ React";
  open.title = "Add a reaction";

  const picker = document.createElement("span");
  picker.className = "reaction-picker";
  picker.hidden = true;
  for (const emoji of commonReactionEmojis) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "reaction-picker-item";
    button.textContent = emoji;
    button.title = "React with " + emoji;
    button.addEventListener("click", () => {
      sendMessageReaction(message, emoji, true);
      picker.hidden = true;
    });
    picker.append(button);
  }

  open.addEventListener("click", (event) => {
    event.stopPropagation();
    picker.hidden = !picker.hidden;
  });
  document.addEventListener("click", (event) => {
    if (!wrap.contains(event.target)) picker.hidden = true;
  }, { once: true });

  wrap.append(open, picker);
  return wrap;
}

function appendMessage(message) {
  messagesById.set(message.id, message);
  if (deletedMessageIds.has(message.id)) return;
  if (!isMessageVisibleAfterE2eReset(message)) return;
  const li = document.createElement("li");
  li.dataset.messageId = message.id;
  if (me && message.sender_id === me.user_id) li.classList.add("me");

  const senderUser = message.sender_id === me?.user_id
    ? me
    : users.find((user) => user.user_id === message.sender_id)
      || groups.find((group) => group.user_id === message.sender_id)
      || { user_id: message.sender_id, display_name: message.sender_name || "Larptrix user" };

  const messageAvatar = document.createElement("span");
  messageAvatar.className = "avatar message-avatar";
  paintAvatar(messageAvatar, senderUser);

  const meta = document.createElement("div");
  meta.className = "meta";
  meta.textContent = (message.sender_name || "Larptrix user") + " · " + new Date(message.created_at).toLocaleTimeString();
  if (senderUser?.tags?.length) {
    const tag = document.createElement("span");
    tag.className = "user-tag message-user-tag";
    tag.textContent = "🏷️ " + senderUser.tags[0];
    tag.title = tagsTitle(senderUser);
    meta.append(tag);
  }

  const previousRow = logEl.lastElementChild;
  const previousMessage = previousRow?.dataset?.messageId
    ? messagesById.get(previousRow.dataset.messageId)
    : null;
  const groupedWithPrevious = Boolean(
    previousMessage
      && previousMessage.sender_id === message.sender_id
      && Math.abs(Number(message.created_at) - Number(previousMessage.created_at)) <= 90000
      && !previousRow.classList.contains("e2e-reset-notice")
  );
  if (groupedWithPrevious) li.classList.add("message-grouped");

  li.append(messageAvatar, meta);

  let encryptedBodyElement = null;
  let messageBodyForSave = null;
  if (message.body) {
    const body = document.createElement("div");
    const envelope = parseCryptoEnvelope(message.body);
    if (envelope) {
      body.textContent = "Encrypted message";
      encryptedBodyElement = body;
    } else {
      body.textContent = cryptoEnabled
        ? "⚠️ Legacy message (not end-to-end encrypted): " + message.body
        : message.body;
    }
    li.append(body);
    messageBodyForSave = body;
  }

  const actions = document.createElement("div");
  actions.className = "message-actions";
  actions.append(createReactionPicker(message));

  const reply = document.createElement("button");
  reply.type = "button";
  reply.className = "ghost";
  reply.textContent = "Reply";
  reply.title = "Reply to this message";
  reply.addEventListener("click", () => setReplyComposer(message));
  actions.append(reply);

  const forward = document.createElement("button");
  forward.type = "button";
  forward.className = "ghost";
  forward.textContent = "Forward";
  forward.title = "Forward this message";
  forward.addEventListener("click", () => void openForwardDialog(message));
  actions.append(forward);

  if (me && message.sender_id === me.user_id) {
    const deleteButton = document.createElement("button");
    deleteButton.type = "button";
    deleteButton.className = "ghost message-delete-button";
    deleteButton.textContent = "Delete";
    deleteButton.title = "Delete this message for everyone";
    deleteButton.addEventListener("click", () => {
      if (!window.confirm("Delete this message for everyone?")) return;
      deleteButton.disabled = true;
      socket?.send(JSON.stringify({
        type: "delete",
        peer_id: message.recipient_id,
        message_id: message.id,
      }));
    });
    actions.append(deleteButton);
  }

  if (messageBodyForSave) {
    const saveButton = document.createElement("button");
    saveButton.type = "button";
    saveButton.className = "ghost";
    saveButton.textContent = "☆ Save";
    saveButton.title = "Save this message";
    saveButton.addEventListener("click", () => {
      void toggleSavedMessage(message, messageBodyForSave.textContent || "");
    });
    actions.append(saveButton);
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
    } else if (message.attachment.mime.startsWith("video/")) {
      const video = document.createElement("video");
      video.className = "chat-video";
      video.controls = true;
      video.playsInline = true;
      video.preload = "metadata";
      video.src = message.attachment.url;
      video.addEventListener("dblclick", () => openVideoViewer(video.src, message.attachment.name));
      li.append(video);
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
      link.textContent = message.attachment.name + " (" + formatSize(message.attachment.size_bytes) + ")";
      li.append(link);
    }
  }

  li.append(actions);
  renderMessageReactions(message, li);
  logEl.append(li);

  const currentChannel = groups.find(
    (item) => item.user_id === peerId && item.is_channel
  );
  if (currentChannel && channelViewObserver) {
    channelViewObserver.observe(li);
  }

  if (encryptedBodyElement) {
    messageBodyElementsById.set(message.id, encryptedBodyElement);

    void loadCachedDecryptedMessagePayload(message.id).then((payload) => {
      if (!payload) return;
      renderCachedDecryptedPayload(message, encryptedBodyElement, payload);
    });

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
      if (effectiveMessage !== message) cryptoRecoveryResponsesByMessageId.delete(message.id);
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

async function stopSpeakingMonitor(key) {
  const monitor = speakingMonitors.get(key);
  if (!monitor) return;
  monitor.stopped = true;
  if (monitor.raf) cancelAnimationFrame(monitor.raf);
  try { monitor.source?.disconnect(); } catch {}
  try { monitor.analyser?.disconnect(); } catch {}
  const closeResult = monitor.audioContext?.close?.();
  closeResult?.catch?.(() => {});
  monitor.element?.classList.remove("speaking");
  speakingMonitors.delete(key);
}

function startSpeakingMonitor(stream, key, element) {
  stopSpeakingMonitor(key);
  if (!stream?.getAudioTracks().length || !element || typeof AudioContext === "undefined") return;
  try {
    const audioContext = new AudioContext();
    const source = audioContext.createMediaStreamSource(stream);
    const analyser = audioContext.createAnalyser();
    analyser.fftSize = 256;
    analyser.smoothingTimeConstant = 0.78;
    source.connect(analyser);
    const data = new Uint8Array(analyser.fftSize);
    const state = { audioContext, source, analyser, element, raf: 0, stopped: false };
    const tick = () => {
      if (state.stopped) return;
      analyser.getByteTimeDomainData(data);
      let sum = 0;
      for (const value of data) {
        const centered = (value - 128) / 128;
        sum += centered * centered;
      }
      const rms = Math.sqrt(sum / data.length);
      element.classList.toggle("speaking", rms > 0.055);
      state.raf = requestAnimationFrame(tick);
    };
    speakingMonitors.set(key, state);
    tick();
    audioContext.resume?.().catch?.(() => {});
  } catch {}
}

function renderDirectCallParticipants() {
  if (!directCallParticipants) return;
  directCallParticipants.replaceChildren();

  if (callStage?.classList.contains("call-ending-notice") && !callNoAnswer) {
    directCallParticipants.hidden = true;
    return;
  }

  const demo = callNoAnswer;
  if (demo) {
    const demoId =
      lastDirectCallAvatarPeerId
      || lastDirectCallPeerId
      || pendingIncomingCall?.sender_id
      || null;
    if (!demoId) {
      directCallParticipants.hidden = true;
      return;
    }
    const user = demoId === me?.user_id
      ? me
      : users.find((item) => item.user_id === demoId)
        || { user_id: demoId, display_name: "Larptrix user" };
    const chip = document.createElement("div");
    chip.className = "call-participant-chip";
    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user);
    const name = document.createElement("span");
    name.textContent = user?.display_name || "Participant";
    chip.append(avatar, name);
    directCallParticipants.append(chip);
    directCallParticipants.hidden = false;
    return;
  }

  const direct = !groupCallId && (
    callPeerId
      || pendingIncomingCall?.sender_id
      || lastDirectCallPeerId
      || lastDirectCallJoinPeerId
  );
  if (!direct) {
    directCallParticipants.hidden = true;
    return;
  }

  const remoteId = callPeerId
    || pendingIncomingCall?.sender_id
    || lastDirectCallPeerId
    || lastDirectCallJoinPeerId;
  const remoteUser = users.find((item) => item.user_id === remoteId)
    || (pendingIncomingCall?.sender_id === remoteId
      ? users.find((item) => item.user_id === pendingIncomingCall.sender_id)
      : null)
    || { user_id: remoteId, display_name: "Larptrix user" };

  const onlyRemote = callNoAnswer || (!peerConnection && lastDirectCallJoinPeerId === remoteId);
  const ids = onlyRemote ? [remoteId] : [me?.user_id, remoteId];

  for (const id of ids.filter(Boolean).filter((item, index, array) => array.indexOf(item) === index)) {
    const user = id === me?.user_id ? me : users.find((item) => item.user_id === id) || remoteUser;
    const chip = document.createElement("div");
    chip.className = "call-participant-chip";

    const avatar = document.createElement("span");
    avatar.className = "avatar";
    paintAvatar(avatar, user || { user_id: id, display_name: "?" });

    const name = document.createElement("span");
    name.textContent = user?.display_name || "Participant";
    chip.append(avatar, name);
    directCallParticipants.append(chip);
  }

  directCallParticipants.hidden = directCallParticipants.childElementCount === 0;
}

function showDirectCallNotice(targetPeerId, message, { join = false, persist = false, kind = "audio", duration = 3000, avatarPeerId = targetPeerId } = {}) {
  if (!targetPeerId) return;
  clearTimeout(directCallNoticeTimeout);
  directCallNoticeTimeout = null;
  lastDirectCallPeerId = targetPeerId;
  lastDirectCallAvatarPeerId = avatarPeerId || targetPeerId;
  lastDirectCallJoinPeerId = join ? targetPeerId : null;
  lastDirectCallJoinKind = kind;
  callNoAnswer = true;

  // Once a call has ended, keep the chat unobstructed. A missed call can
  // still be joined from the chat header without reopening the call window.
  callStage.hidden = true;
  callStage.classList.remove("call-ending-notice");
  directCallJoin && (directCallJoin.hidden = !join);
  renderDirectCallTopbar(peerId || targetPeerId);

  if (!join && !persist) {
    directCallNoticeTimeout = setTimeout(() => {
      if (lastDirectCallPeerId !== targetPeerId) return;
      directCallNoticeTimeout = null;
      lastDirectCallPeerId = null;
      lastDirectCallAvatarPeerId = null;
      lastDirectCallJoinPeerId = null;
      lastDirectCallJoinKind = "audio";
      callNoAnswer = false;
      renderDirectCallTopbar(peerId);
      renderDirectCallAvatarStack();
    }, duration);
  }
}

function renderDirectCallAvatarStack() {
  // The old compact stack duplicated the full call participant UI.
  // Keep the element hidden; the no-answer demo uses the large centered avatar.
  if (directCallAvatarStack) {
    directCallAvatarStack.replaceChildren();
    directCallAvatarStack.hidden = true;
  }
  renderDirectCallParticipants();
}

function attachLocalMediaPreview() {
  localVideo.srcObject = localMediaStream;
  localVideo.hidden = !localMediaStream?.getVideoTracks().length;
  startSpeakingMonitor(
    localMediaStream,
    "local",
    callPlaceholderLocalAvatar,
  );
  document.getElementById("toggle-microphone").disabled = !localMediaStream?.getAudioTracks().length;
  document.getElementById("toggle-camera").disabled = !localMediaStream?.getVideoTracks().length;
  if (!localVideo.hidden) {
    const playback = localVideo.play();
    if (playback && typeof playback.catch === "function") {
      playback.catch(() => {
        callStatus.textContent = "Camera is on. Click the preview to start local playback.";
      });
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

async function retryVisibleMatrixMessages({ attempts = 8, delayMs = 350 } = {}) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    let failed = false;
    const entries = [...messageBodyElementsById.entries()];
    for (const [messageId, bodyElement] of entries) {
      const message = messagesById.get(messageId);
      if (!message || !parseCryptoEnvelope(message.body)) continue;
      if (
        !["Encrypted message", "Decrypting…"].includes(bodyElement.textContent || "")
        && !(bodyElement.textContent || "").startsWith("Could not decrypt Matrix message:")
      ) continue;
      const ok = await displayEncryptedMessage(message, bodyElement, { allowRecovery: false });
      if (!ok) failed = true;
    }
    if (!failed) return;
    if (attempt + 1 < attempts) {
      await new Promise((resolve) => setTimeout(resolve, delayMs * Math.min(attempt + 1, 3)));
    }
  }

  for (const [messageId, bodyElement] of messageBodyElementsById.entries()) {
    const message = messagesById.get(messageId);
    if (!message || !parseCryptoEnvelope(message.body)) continue;
    if ((bodyElement.textContent || "") === "Decrypting…") {
      bodyElement.textContent = "Could not decrypt Matrix message yet. E2E keys are still unavailable.";
    }
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
      decryptedPayloadByMessageId.set(message.id, payload);
      void cacheDecryptedMessagePayload(message.id, payload);
      message._decryptedPayload = payload;
      renderMessageDecorations(bodyElement.parentElement, payload);
      bodyElement.textContent =
        typeof payload?.text === "string" ? payload.text : JSON.stringify(payload);

      if (payload?.files?.length || (payload?.file && message.attachment)) {
        try {
          await renderEncryptedAttachments(message, payload, bodyElement.parentElement);
        } catch (err) {
          bodyElement.textContent =
            `Could not open encrypted attachment: ${err?.message || String(err)}`;
        }
      }
      if (payload?.gif) {
        try {
          renderSelectedGif(payload.gif, bodyElement.parentElement);
        } catch (err) {
          bodyElement.textContent =
            `Could not open GIF: ${err?.message || String(err)}`;
        }
      }
      return true;
    } catch (err) {
      bodyElement.textContent = "Decrypting…";
      return false;
    }
  }

  if (message.sender_id === me?.user_id) {
    await cryptoReady;
    const cached = sentPlaintextByCiphertext.get(message.body)
      || await loadCachedSentPlaintext(message.body);
    if (cached) sentPlaintextByCiphertext.set(message.body, cached);
    const payload = parseEncryptedPayload(cached);
    decryptedPayloadByMessageId.set(message.id, payload);
    message._decryptedPayload = payload;
    renderMessageDecorations(bodyElement.parentElement, payload);
    bodyElement.textContent = payload?.text || "Encrypted message sent from this device";
    if (payload?.files?.length || (payload?.file && message.attachment)) {
      try {
        await renderEncryptedAttachments(message, payload, bodyElement.parentElement);
      } catch (err) {
        bodyElement.textContent = `Could not open encrypted attachment: ${err?.message || String(err)}`;
      }
    }
      if (payload?.gif) {
        try {
          renderSelectedGif(payload.gif, bodyElement.parentElement);
        } catch (err) {
          bodyElement.textContent =
            `Could not open GIF: ${err?.message || String(err)}`;
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
    decryptedPayloadByMessageId.set(message.id, payload);
    message._decryptedPayload = payload;
    renderMessageDecorations(bodyElement.parentElement, payload);
    bodyElement.textContent = payload?.text ?? result;

    if (payload?.files?.length || (payload?.file && message.attachment)) {
      try {
        await renderEncryptedAttachments(message, payload, bodyElement.parentElement);
      } catch (err) {
        bodyElement.textContent =
          `Could not open encrypted attachment: ${err?.message || String(err)}`;
      }
    }
    if (payload?.gif) {
      try {
        renderSelectedGif(payload.gif, bodyElement.parentElement);
      } catch (err) {
        bodyElement.textContent =
          `Could not open GIF: ${err?.message || String(err)}`;
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

async function blobSha256(blob) {
  const digest = await crypto.subtle.digest("SHA-256", await blob.arrayBuffer());
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function optimizeAttachmentImage(file) {
  if (!file?.type || !file.type.startsWith("image/") || file.type === "image/gif") return file;
  // Keep small images untouched. Large photos/screenshots are downscaled and
  // re-encoded as WebP only when the result is meaningfully smaller.
  if (file.size < 1_500_000) return file;
  let image;
  try {
    image = await createImageBitmap(file);
    const maxDimension = 2560;
    const scale = Math.min(1, maxDimension / image.width, maxDimension / image.height);
    if (scale === 1 && file.type === "image/webp") return file;

    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(image.width * scale));
    canvas.height = Math.max(1, Math.round(image.height * scale));
    const context = canvas.getContext("2d", { alpha: true });
    if (!context) return file;
    context.imageSmoothingEnabled = true;
    context.imageSmoothingQuality = "high";
    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise((resolve) => canvas.toBlob(resolve, "image/webp", 0.86));
    if (!blob || blob.size >= file.size * 0.93) return file;

    const name = file.name.replace(/.[^.]+$/, "") + ".webp";
    return new File([blob], name, { type: "image/webp", lastModified: file.lastModified });
  } catch {
    return file;
  } finally {
    image?.close?.();
  }
}

async function maybeCompressAttachmentBytes(bytes) {
  if (typeof CompressionStream !== "function" || !bytes?.length) {
    return { bytes, compression: null };
  }
  try {
    const stream = new Blob([bytes]).stream().pipeThrough(new CompressionStream("gzip"));
    const compressed = new Uint8Array(await new Response(stream).arrayBuffer());
    // Never spend CPU/storage on a compressed representation that barely helps.
    if (compressed.length + 512 >= bytes.length || compressed.length > bytes.length * 0.97) {
      return { bytes, compression: null };
    }
    return { bytes: compressed, compression: "gzip" };
  } catch {
    return { bytes, compression: null };
  }
}

async function encryptAttachment(file) {
  const sourceFile = await optimizeAttachmentImage(file);
  const sourceBytes = new Uint8Array(await sourceFile.arrayBuffer());
  const prepared = await maybeCompressAttachmentBytes(sourceBytes);
  const key = await crypto.subtle.generateKey({ name: "AES-GCM", length: 256 }, true, ["encrypt"]);
  const rawKey = await crypto.subtle.exportKey("raw", key);
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, prepared.bytes);
  const gifId = sourceFile.type === "image/gif"
    ? await blobSha256(sourceFile)
    : null;
  return {
    file: new File([ciphertext], sourceFile.name + ".encrypted", { type: "application/octet-stream" }),
    metadata: {
      key: bytesToBase64(new Uint8Array(rawKey)),
      iv: bytesToBase64(iv),
      name: sourceFile.name,
      mime: sourceFile.type || "application/octet-stream",
      size: sourceFile.size,
      stored_size: ciphertext.byteLength,
      ...(prepared.compression ? { compression: prepared.compression } : {}),
      ...(gifId ? { gif_id: gifId } : {}),
    },
  };
}

const GIF_FAVORITES_DB = "larptrix-local-media";
const GIF_FAVORITES_STORE = "gifs";

function openGifFavoritesDb() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(GIF_FAVORITES_DB, 1);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(GIF_FAVORITES_STORE)) {
        db.createObjectStore(GIF_FAVORITES_STORE, { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error || new Error("Could not open GIF favorites."));
  });
}

async function readGifFavoritesRaw() {
  const db = await openGifFavoritesDb();
  const items = await new Promise((resolve, reject) => {
    const tx = db.transaction(GIF_FAVORITES_STORE, "readonly");
    const request = tx.objectStore(GIF_FAVORITES_STORE).getAll();
    request.onsuccess = () => resolve(request.result || []);
    request.onerror = () => reject(request.error || new Error("Could not load saved GIFs."));
  });
  db.close();
  return items;
}

async function saveGifFavorite(blob, name, gifId = null) {
  const stableId = gifId || await blobSha256(blob);
  const items = await readGifFavoritesRaw();
  for (const existing of items) {
    const existingId = existing.gif_id || await blobSha256(existing.blob);
    if (existingId === stableId) {
      if (!existing.gif_id) {
        const db = await openGifFavoritesDb();
        await new Promise((resolve, reject) => {
          const tx = db.transaction(GIF_FAVORITES_STORE, "readwrite");
          tx.objectStore(GIF_FAVORITES_STORE).put({ ...existing, gif_id: stableId });
          tx.oncomplete = resolve;
          tx.onerror = () => reject(tx.error || new Error("Could not update saved GIF."));
        });
        db.close();
      }
      throw new Error("This GIF is already saved.");
    }
  }

  const db = await openGifFavoritesDb();
  const item = {
    id: crypto.randomUUID(),
    gif_id: stableId,
    name: name || "saved.gif",
    mime: blob.type || "image/gif",
    blob,
    createdAt: Date.now(),
  };
  await new Promise((resolve, reject) => {
    const tx = db.transaction(GIF_FAVORITES_STORE, "readwrite");
    tx.objectStore(GIF_FAVORITES_STORE).put(item);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error || new Error("Could not save GIF."));
  });
  db.close();
  return stableId;
}

async function listGifFavorites() {
  const items = await readGifFavoritesRaw();
  return items.sort((a, b) => b.createdAt - a.createdAt);
}

async function deleteGifFavorite(id) {
  const db = await openGifFavoritesDb();
  await new Promise((resolve, reject) => {
    const tx = db.transaction(GIF_FAVORITES_STORE, "readwrite");
    tx.objectStore(GIF_FAVORITES_STORE).delete(id);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error || new Error("Could not remove GIF."));
  });
  db.close();
}

async function addGifToFavorites(blob, name, container, gifId = null) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "ghost gif-save-button";
  button.textContent = "♡ Save GIF";
  const stableId = gifId || await blobSha256(blob);

  try {
    const favorites = await listGifFavorites();
    const alreadySaved = favorites.some((favorite) => favorite.gif_id === stableId);
    if (alreadySaved) {
      button.textContent = "♥ Saved";
      button.disabled = true;
    }
  } catch {}

  button.addEventListener("click", async () => {
    button.disabled = true;
    try {
      await saveGifFavorite(blob, name, stableId);
      button.textContent = "♥ Saved";
    } catch (err) {
      button.disabled = false;
      button.textContent = err?.message || "Save failed";
      if (err?.message === "This GIF is already saved.") button.disabled = true;
    }
  });
  container.append(button);
}

async function renderGifFavorites() {
  if (!gifResults || !gifEmpty) return;
  gifResults.replaceChildren();
  try {
    const favorites = await listGifFavorites();
    gifEmpty.hidden = favorites.length > 0;
    for (const favorite of favorites) {
      const item = document.createElement("article");
      item.className = "gif-favorite";
      const image = document.createElement("img");
      image.src = URL.createObjectURL(favorite.blob);
      image.alt = favorite.name;
      image.loading = "lazy";
      image.className = "gif-favorite-image";

      const actions = document.createElement("div");
      actions.className = "gif-favorite-actions";
      const send = document.createElement("button");
      send.type = "button";
      send.textContent = "Send";
      send.addEventListener("click", () => {
        const file = new File([favorite.blob], favorite.name || "saved.gif", {
          type: favorite.mime || favorite.blob.type || "image/gif",
        });
        queueAttachment(file);
        gifDialog.close();
      });
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "ghost";
      remove.textContent = "Remove";
      remove.addEventListener("click", async () => {
        await deleteGifFavorite(favorite.id);
        void renderGifFavorites();
      });
      actions.append(send, remove);
      item.append(image, actions);
      gifResults.append(item);
    }
  } catch (err) {
    gifEmpty.hidden = false;
    gifEmpty.textContent = err?.message || "Could not load saved GIFs.";
  }
}

async function renderEncryptedAttachment(attachment, metadata, container, { videoMessageShape: videoShape = null } = {}) {
  const urlForAttachment = attachment?.url
    || metadata?.url
    || (metadata?.id ? `/api/attachments/${encodeURIComponent(metadata.id)}` : null);
  if (!urlForAttachment) throw new Error("Encrypted attachment has no file URL.");
  const response = await fetch(urlForAttachment, { credentials: "same-origin" });
  if (!response.ok) throw new Error("Encrypted attachment could not be loaded.");
  const ciphertext = await response.arrayBuffer();
  const key = await crypto.subtle.importKey("raw", base64ToBytes(metadata.key), "AES-GCM", false, ["decrypt"]);
  let plaintext = new Uint8Array(await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: base64ToBytes(metadata.iv) },
    key,
    ciphertext,
  ));

  if (metadata.compression === "gzip" && typeof DecompressionStream === "function") {
    try {
      const stream = new Blob([plaintext]).stream().pipeThrough(new DecompressionStream("gzip"));
      plaintext = new Uint8Array(await new Response(stream).arrayBuffer());
    } catch {
      throw new Error("Compressed attachment could not be decompressed.");
    }
  }

  const blob = new Blob([plaintext], { type: metadata.mime });
  const url = URL.createObjectURL(blob);

  if (metadata.mime === "image/gif") {
    const image = document.createElement("img");
    image.className = "photo";
    image.src = url;
    image.alt = metadata.name;
    image.tabIndex = 0;
    image.setAttribute("role", "button");
    image.title = "Open image";
    image.addEventListener("click", () => openImageViewer(url, metadata.name));
    container.append(image);
    void addGifToFavorites(blob, metadata.name, container, metadata.gif_id || null);
  } else if (metadata.mime.startsWith("image/")) {
    const image = document.createElement("img");
    image.className = "photo";
    image.src = url;
    image.alt = metadata.name;
    image.tabIndex = 0;
    image.setAttribute("role", "button");
    image.title = "Open image";
    image.addEventListener("click", () => openImageViewer(url, metadata.name));
    container.append(image);
  } else if (metadata.mime.startsWith("video/")) {
    const video = document.createElement("video");
    const resolvedShape =
      videoShape === "square"
        ? "square"
        : videoShape === "circle"
          ? "circle"
          : null;
    video.className = "chat-video" + (resolvedShape
      ? " video-message-media video-message-" + resolvedShape
      : "");
    video.controls = true;
    video.playsInline = true;
    video.preload = "metadata";
    video.src = url;
    video.title = metadata.name;
    video.addEventListener("dblclick", () => openVideoViewer(url, metadata.name));
    container.append(video);
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

async function renderEncryptedAttachments(message, payload, container) {
  const files = Array.isArray(payload?.files) && payload.files.length
    ? payload.files
    : payload?.file && message?.attachment
      ? [{
          ...payload.file,
          id: message.attachment.id,
          url: message.attachment.url,
        }]
      : [];

  if (!files.length) return;
  const existingMedia = container.querySelector(".message-media-stack, .message-media-grid");
  if (existingMedia) return;

  const media = document.createElement("div");
  media.className = files.length > 1 && files.every((file) => String(file?.mime || "").startsWith("image/"))
    ? "message-media-grid"
    : "message-media-stack";
  container.append(media);

  for (const [index, metadata] of files.entries()) {
    const fallbackAttachment = index === 0
      ? message?.attachment
      : Array.isArray(message?.attachments) ? message.attachments[index] : null;
    await renderEncryptedAttachment(
      fallbackAttachment,
      metadata,
      media,
      {
        videoMessageShape:
          payload?.video_message?.name === metadata?.name
            ? payload.video_message.shape
            : null,
      },
    );
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

function queueAttachments(files) {
  const incoming = [...files].filter((file) => file instanceof File && file.size > 0);
  if (!incoming.length) return;
  pendingAttachments.push(...incoming);
  renderAttachmentPreview();
}

function renderAttachmentPreview() {
  for (const url of previewUrls) URL.revokeObjectURL(url);
  previewUrls = [];
  attachmentPreview.replaceChildren();

  pendingAttachments.forEach((file, index) => {
    const item = document.createElement("div");
    item.className = "attachment-preview-item";
    const url = URL.createObjectURL(file);
    previewUrls.push(url);

    if (file.type.startsWith("image/")) {
      const image = document.createElement("img");
      image.src = url;
      image.alt = file.name;
      item.append(image);
    } else if (file.type.startsWith("video/")) {
      const video = document.createElement("video");
      const shape = videoMessageShapeByFile.get(file);
      video.className = shape
        ? "video-message-preview video-message-" + shape
        : "video-message-preview";
      video.src = url;
      video.muted = true;
      video.playsInline = true;
      video.preload = "metadata";
      video.autoplay = true;
      video.loop = true;
      item.append(video);
    } else if (file.type.startsWith("audio/")) {
      const audio = document.createElement("audio");
      audio.controls = true;
      audio.src = url;
      item.append(audio);
    } else {
      const fileIcon = document.createElement("span");
      fileIcon.className = "attachment-file-icon";
      fileIcon.textContent = "↗";
      item.append(fileIcon);
    }

    const info = document.createElement("span");
    info.className = "attachment-preview-name";
    info.textContent = file.name;
    info.title = file.name;
    item.append(info);

    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "ghost";
    remove.textContent = "×";
    remove.title = "Remove attachment";
    remove.setAttribute("aria-label", `Remove ${file.name}`);
    remove.addEventListener("click", () => {
      pendingAttachments.splice(index, 1);
      renderAttachmentPreview();
    });
    item.append(remove);
    attachmentPreview.append(item);
  });

  attachmentPreview.hidden = pendingAttachments.length === 0;
}

function queueAttachment(file) {
  queueAttachments(file ? [file] : []);
}


function browserLocale() {
  const locale = (navigator.language || "en-US").replace("-", "_").trim();
  return locale.length <= 16 ? locale : "en_US";
}

function browserCountry() {
  const parts = browserLocale().split("_");
  const country = parts[1] || "US";
  return /^[A-Za-z]{2}$/.test(country) ? country.toUpperCase() : "US";
}

function renderSelectedGif(gif, container) {
  if (!gif || typeof gif.url !== "string") {
    throw new Error("This GIF has an invalid media URL.");
  }

  const mediaUrl = new URL(gif.url);
  if (
    mediaUrl.protocol !== "https:"
    || mediaUrl.hostname.toLowerCase() !== "static.klipy.com"
  ) {
    throw new Error("This GIF did not come from KLIPY.");
  }

  if (container.querySelector("[data-klipy-gif-embed]")) return;
  const figure = document.createElement("figure");
  figure.dataset.klipyGifEmbed = "1";
  figure.className = "gif-attachment";

  const image = document.createElement("img");
  image.src = gif.url;
  image.alt = typeof gif.title === "string" && gif.title ? gif.title : "KLIPY GIF";
  image.loading = "lazy";
  image.decoding = "async";
  image.referrerPolicy = "no-referrer";
  figure.append(image);

  if (typeof gif.item_url === "string") {
    try {
      const itemUrl = new URL(gif.item_url);
      if (
        itemUrl.protocol === "https:"
        && (itemUrl.hostname.toLowerCase() === "klipy.com"
          || itemUrl.hostname.toLowerCase().endsWith(".klipy.com"))
      ) {
        const link = document.createElement("a");
        link.href = gif.item_url;
        link.target = "_blank";
        link.rel = "noopener noreferrer";
        link.textContent = "View on KLIPY";
        figure.append(link);
      }
    } catch {}
  }

  container.append(figure);
}

function clearAttachment() {
  pendingAttachments = [];
  pendingGif = null;
  videoMessageFile = null;
  for (const url of previewUrls) URL.revokeObjectURL(url);
  previewUrls = [];
  attachmentPreview.replaceChildren();
  attachmentPreview.hidden = true;
  photoInput.value = "";
  fileInput.value = "";
  audioFileInput.value = "";
}

function updateVoiceRecordingUi() {
  if (!voiceRecording || !voiceRecordingTime) return;
  const elapsed = Math.max(0, Date.now() - voiceRecordingStartedAt);
  const seconds = Math.floor(elapsed / 1000);
  voiceRecordingTime.textContent = Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}

function cancelVoiceRecording() {
  if (recorder && recorder.state !== "inactive") recorder.stop();
  recorder = null;
  recordingStream?.getTracks().forEach((track) => track.stop());
  recordingStream = null;
  recordedChunks = [];
  clearInterval(voiceRecordingTimer);
  voiceRecordingTimer = null;
  voiceRecording?.setAttribute("hidden", "");
  if (recordAudioButton) recordAudioButton.textContent = "🎙";
}

function selectedVideoMessageShape() {
  return videoMessageShape === "square" ? "square" : "circle";
}

function updateVideoRecordingUi() {
  if (!videoRecordingTime) return;
  const elapsed = Math.max(0, Date.now() - videoRecordingStartedAt);
  const seconds = Math.floor(elapsed / 1000);
  videoRecordingTime.textContent =
    Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}

function applyVideoMessageShape(shape) {
  videoMessageShape = shape === "square" ? "square" : "circle";
  localStorage.setItem("larptrix_video_message_shape", videoMessageShape);
  const wrap = videoRecording?.querySelector(".video-recording-preview-wrap");
  if (wrap) wrap.dataset.shape = videoMessageShape;
  if (videoRecordingShape) {
    videoRecordingShape.textContent = videoMessageShape === "circle" ? "Circle" : "Square";
    videoRecordingShape.setAttribute("aria-pressed", String(videoMessageShape === "circle"));
  }
}

function finishVideoRecordingCleanup() {
  videoRecording?.setAttribute("hidden", "");
  if (videoRecordingTimer) clearInterval(videoRecordingTimer);
  videoRecordingTimer = null;
  if (videoRecordingPreview) videoRecordingPreview.srcObject = null;
}

function cancelVideoRecording() {
  videoRecordingCancelled = true;
  if (videoRecorder && videoRecorder.state !== "inactive") {
    videoRecorder.stop();
  } else {
    videoRecordingStream?.getTracks().forEach((track) => track.stop());
    videoRecordingStream = null;
    videoRecordedChunks = [];
    videoRecorder = null;
    finishVideoRecordingCleanup();
  }
}

async function toggleVideoRecording() {
  if (videoRecorder && videoRecorder.state === "recording") {
    videoRecorder.stop();
    return;
  }
  if (recorder && recorder.state === "recording") {
    appendSystem("Stop the voice recording before starting a video message.");
    return;
  }
  if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") {
    appendSystem("Video recording is not supported by this browser.");
    return;
  }

  try {
    const cameraId = selectedCallDeviceId("videoinput");
    const audioId = selectedCallDeviceId("audioinput");
    const videoConstraint = cameraId
      ? { deviceId: { exact: cameraId }, width: { ideal: 720 }, height: { ideal: 720 } }
      : { width: { ideal: 720 }, height: { ideal: 720 } };
    const attempts = [
      {
        video: videoConstraint,
        audio: audioId
          ? { deviceId: { exact: audioId }, echoCancellation: true }
          : true,
      },
      { video: videoConstraint, audio: false },
      { video: true, audio: false },
    ];

    let lastError = null;
    for (const constraints of attempts) {
      try {
        videoRecordingStream = await navigator.mediaDevices.getUserMedia(constraints);
        break;
      } catch (err) {
        lastError = err;
      }
    }
    if (!videoRecordingStream) throw lastError || new Error("Could not access the camera.");

    const mimeType = [
      "video/webm;codecs=vp9,opus",
      "video/webm;codecs=vp8,opus",
      "video/webm",
    ].find((type) => MediaRecorder.isTypeSupported(type));

    videoRecorder = new MediaRecorder(
      videoRecordingStream,
      mimeType
        ? { mimeType, videoBitsPerSecond: 2_500_000, audioBitsPerSecond: 96_000 }
        : undefined,
    );
    videoRecordedChunks = [];
    videoRecordingCancelled = false;
    videoRecordingStartedAt = Date.now();

    if (videoRecordingPreview) {
      videoRecordingPreview.srcObject = videoRecordingStream;
      videoRecordingPreview.muted = true;
      videoRecordingPreview.playsInline = true;
      videoRecordingPreview.play().catch(() => {});
    }

    applyVideoMessageShape(selectedVideoMessageShape());
    videoRecording?.removeAttribute("hidden");
    if (videoRecordingStatus) videoRecordingStatus.textContent =
      "Recording " + (selectedVideoMessageShape() === "circle" ? "circle" : "square") + " video message…";
    if (recordVideoButton) {
      recordVideoButton.textContent = "⏹";
      recordVideoButton.title = "Stop video message recording";
      recordVideoButton.classList.add("is-recording");
    }

    videoRecordingTimer = setInterval(updateVideoRecordingUi, 250);
    updateVideoRecordingUi();

    const stopAfterLimit = setTimeout(() => {
      if (videoRecorder?.state === "recording") videoRecorder.stop();
    }, 60_000);

    videoRecorder.addEventListener("dataavailable", (event) => {
      if (event.data.size) videoRecordedChunks.push(event.data);
    });

    videoRecorder.addEventListener("stop", () => {
      clearTimeout(stopAfterLimit);
      const instance = videoRecorder;
      const stream = videoRecordingStream;
      const wasCancelled = videoRecordingCancelled;
      const blob = new Blob(videoRecordedChunks, {
        type: instance?.mimeType || "video/webm",
      });

      stream?.getTracks().forEach((track) => track.stop());
      videoRecordingStream = null;
      videoRecorder = null;
      videoRecordedChunks = [];

      if (recordVideoButton) {
        recordVideoButton.textContent = "📹";
        recordVideoButton.title = "Record a video message";
        recordVideoButton.classList.remove("is-recording");
      }

      finishVideoRecordingCleanup();

      if (!wasCancelled && blob.size) {
        const file = new File([blob], "video-message.webm", {
          type: blob.type || "video/webm",
        });
        videoMessageFile = file;
        videoMessageShapeByFile.set(file, selectedVideoMessageShape());
        queueAttachment(file);
        if (videoRecordingStatus) videoRecordingStatus.textContent = "Video message ready to send.";
      }
      videoRecordingCancelled = false;
    }, { once: true });

    videoRecorder.start(250);
  } catch (err) {
    videoRecordingStream?.getTracks().forEach((track) => track.stop());
    videoRecordingStream = null;
    videoRecorder = null;
    finishVideoRecordingCleanup();
    if (recordVideoButton) {
      recordVideoButton.textContent = "📹";
      recordVideoButton.title = "Record a video message";
      recordVideoButton.classList.remove("is-recording");
    }
    appendSystem(err.message || "Could not access the camera.");
  }
}

async function toggleRecording() {
  if (recorder && recorder.state === "recording") {
    recorder.stop();
    return;
  }
  if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === "undefined") {
    appendSystem("Audio recording is not supported by this browser.");
    return;
  }
  try {
    recordingStream = await navigator.mediaDevices.getUserMedia({ audio: selectedCallDeviceId("audioinput") ? { deviceId: { exact: selectedCallDeviceId("audioinput") } } : true });
    const mimeType = ["audio/webm;codecs=opus", "audio/ogg;codecs=opus"]
      .find((type) => MediaRecorder.isTypeSupported(type));
    recorder = new MediaRecorder(recordingStream, mimeType ? { mimeType } : undefined);
    recordedChunks = [];
    voiceRecordingStartedAt = Date.now();
    voiceRecording?.removeAttribute("hidden");
    if (voiceRecordingStatus) voiceRecordingStatus.textContent = "Recording voice message…";
    voiceRecordingTimer = setInterval(updateVoiceRecordingUi, 250);
    updateVoiceRecordingUi();
    if (recordAudioButton) recordAudioButton.textContent = "⏹";
    recorder.addEventListener("dataavailable", (event) => {
      if (event.data.size) recordedChunks.push(event.data);
    });
    recorder.addEventListener("stop", () => {
      const wasCancelled = voiceRecording?.dataset.cancelled === "1";
      voiceRecording?.dataset && delete voiceRecording.dataset.cancelled;
      const blob = new Blob(recordedChunks, { type: recorder?.mimeType || "audio/webm" });
      const stream = recordingStream;
      recordingStream = null;
      stream?.getTracks().forEach((track) => track.stop());
      clearInterval(voiceRecordingTimer);
      voiceRecordingTimer = null;
      recorder = null;
      voiceRecording?.setAttribute("hidden", "");
      if (!wasCancelled && blob.size) {
        queueAttachment(new File([blob], "voice-message.webm", { type: blob.type || "audio/webm" }));
      }
      if (recordAudioButton) recordAudioButton.textContent = "🎙";
    }, { once: true });
    recorder.start(120);
  } catch (err) {
    appendSystem(err.message || "Could not access the microphone.");
    recordingStream?.getTracks().forEach((track) => track.stop());
    recordingStream = null;
    recorder = null;
    voiceRecording?.setAttribute("hidden", "");
  }
}

videoRecordingShape?.addEventListener("click", () => {
  applyVideoMessageShape(selectedVideoMessageShape() === "circle" ? "square" : "circle");
});
directCallJoin?.addEventListener("click", () => {
  const target = callPeerId || pendingIncomingCall?.sender_id || lastDirectCallJoinPeerId;
  if (!target) return;
  peerId = target;
  clearTimeout(directCallNoticeTimeout);
  directCallNoticeTimeout = null;
  if (pendingIncomingCall?.sender_id === target && !peerConnection) {
    void acceptIncomingCall();
    return;
  }
  const kind = callMediaKind || lastDirectCallJoinKind || "audio";
  void startCall(kind);
});

videoRecordingCancel?.addEventListener("click", cancelVideoRecording);

voiceRecordingCancel?.addEventListener("click", () => {
  if (!recorder || recorder.state === "inactive") return;
  voiceRecording.dataset.cancelled = "1";
  voiceRecordingStatus.textContent = "Cancelling…";
  recorder.stop();
});

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
  if (imageViewerVideo) {
    imageViewerVideo.pause();
    imageViewerVideo.removeAttribute("src");
    imageViewerVideo.hidden = true;
  }
  imageViewerImage.hidden = false;
  imageViewerImage.src = src;
  imageViewerImage.alt = alt;
  imageViewer.showModal();
}

function openVideoViewer(src, name = "Video") {
  imageViewerImage.hidden = true;
  imageViewerImage.removeAttribute("src");
  if (!imageViewerVideo) return;
  imageViewerVideo.hidden = false;
  imageViewerVideo.setAttribute("aria-label", name);
  imageViewerVideo.src = src;
  imageViewer.showModal();
  imageViewerVideo.play().catch(() => {});
}

function getUserActivities(user) {
  const activities = Array.isArray(user?.activities) ? user.activities : [];
  if (activities.length) return activities;
  if (user?.activity) {
    return [{
      kind: "custom",
      name: String(user.activity),
      details: "",
      url: null,
      image_url: null,
    }];
  }
  return [];
}

function activityIcon(kind) {
  if (kind === "music") return "♫";
  return "💬";
}

function renderActivityEntries(parent, user, { compact = false, limit = 4 } = {}) {
  if (!parent) return;
  const activities = getUserActivities(user).slice(0, limit);
  for (const activity of activities) {
    const row = activity.url ? document.createElement("a") : document.createElement("div");
    row.className = compact ? "person-activity activity-entry activity-entry-compact" : "activity-entry";
    if (activity.url) {
      row.href = activity.url;
      row.target = "_blank";
      row.rel = "noopener noreferrer";
    }
    if (activity.image_url) {
      const image = document.createElement("img");
      image.className = "activity-entry-image";
      image.src = activity.image_url;
      image.alt = "";
      image.loading = "lazy";
      row.append(image);
    } else {
      const icon = document.createElement("span");
      icon.className = "activity-entry-icon";
      icon.textContent = activityIcon(activity.kind);
      row.append(icon);
    }
    const copy = document.createElement("span");
    copy.className = "activity-entry-copy";
    const name = document.createElement("strong");
    name.textContent = activity.name;
    copy.append(name);
    if (activity.details) {
      const details = document.createElement("small");
      details.textContent = activity.details;
      copy.append(details);
    }
    row.append(copy);
    row.title = activity.details ? activity.name + " · " + activity.details : activity.name;
    parent.append(row);
  }
}

function renderCustomActivityEditor() {
  if (!profileCustomActivities) return;
  profileCustomActivities.replaceChildren();
  if (!customActivities.length) {
    const empty = document.createElement("span");
    empty.className = "settings-help";
    empty.textContent = "No custom statuses yet.";
    profileCustomActivities.append(empty);
    return;
  }
  customActivities.forEach((activity, index) => {
    const row = document.createElement("div");
    row.className = "profile-status-editor-row";
    const text = document.createElement("span");
    text.textContent = activity.name;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "ghost";
    remove.textContent = "×";
    remove.title = "Remove status";
    remove.addEventListener("click", () => {
      customActivities.splice(index, 1);
      renderCustomActivityEditor();
    });
    row.append(text, remove);
    profileCustomActivities.append(row);
  });
}

async function syncActivities(next = null) {
  if (!me) return;
  const source = next || [
    ...customActivities,
    ...(musicActivityEnabled && localMusicActivity ? [localMusicActivity] : []),
  ];
  const activities = source
    .filter((item) => item?.name?.trim())
    .slice(0, 5)
    .map((item) => ({
      kind: item.kind || "custom",
      name: item.name.trim(),
      details: item.details || "",
      url: item.url || null,
      image_url: item.image_url || null,
    }));
  try {
    const result = await api("POST", "/api/me/activities", { activities });
    me = { ...me, activity: result.activity || null, activities: result.activities || activities };
    if (next) {
      customActivities = [];
      localMusicActivity = null;    }
    updateOwnProfileCard(me);
    renderUsers();
  } catch {
    return;
  }
}

function updateOwnProfileCard(profile) {
  profileCardName.textContent = profile.display_name || "";
  profileCardHandle.textContent = profile.username ? `@${profile.username}` : "";
  profileCardActivity.replaceChildren();
  renderActivityEntries(profileCardActivity, profile, { compact: false, limit: 5 });
  if (!getUserActivities(profile).length) {
    const empty = document.createElement("span");
    empty.className = "profile-activity-empty";
    empty.textContent = "No activity";
    profileCardActivity.append(empty);
  }
}

function updateMusicActivity(trackName, playing) {
  localMusicActivity = playing
    ? { kind: "music", name: "Listening to " + trackName, details: "Larptrix Music Library", url: null, image_url: null }
    : null;
  void syncActivities();
}

window.larptixMusicStatus = {
  update: updateMusicActivity,
  refresh() {
    const player = document.getElementById("music-audio");
    const name = document.getElementById("music-player-name")?.textContent?.trim();
    updateMusicActivity(name || "", Boolean(player && !player.paused && musicActivityEnabled));
  },
};


function chatWallpaperKey(id) {
  return `larptrix_chat_wallpaper_${me.user_id}_${id}`;
}

function applyChatWallpaper(id) {
  const wallpaper = localStorage.getItem(chatWallpaperKey(id));
  const tint = localStorage.getItem("larptrix_wallpaper_tint") || "theme";
  if (!wallpaper) {
    logEl.style.backgroundImage = "";
    logEl.style.backgroundBlendMode = "";
    return;
  }
  if (tint === "none") {
    logEl.style.backgroundImage = `url("${wallpaper}")`;
    logEl.style.backgroundBlendMode = "normal";
    return;
  }
  const accent = "color-mix(in srgb, var(--accent) 18%, transparent)";
  logEl.style.backgroundImage = `linear-gradient(${accent}, ${accent}), url("${wallpaper}")`;
  logEl.style.backgroundBlendMode = "normal, normal";
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
    const group = groups.find((item) => item.user_id === id && item.is_group);
    const avatar = document.getElementById("peer-profile-avatar");
    const nameEl = document.getElementById("peer-profile-name");
    const usernameEl = document.getElementById("peer-profile-username");
    const aboutEl = document.getElementById("peer-profile-about");
    const tagsEl = document.getElementById("peer-profile-tags");
    const serverEl = document.getElementById("peer-profile-server");
    const activityEl = document.getElementById("peer-profile-activity");

    if (group) {
      avatar.hidden = !group.avatar_url;
      if (group.avatar_url) {
        avatar.src = group.avatar_url + "?v=" + Date.now();
      } else {
        avatar.removeAttribute("src");
      }
      avatar.alt = `${group.display_name} avatar`;
      peerProfileBanner.style.backgroundImage = group.banner_url
        ? 'url("' + group.banner_url + '?v=' + Date.now() + '")'
        : "";
      nameEl.textContent = group.display_name;
      usernameEl.textContent = group.is_channel
        ? `${group.subscriber_count || group.group_member_ids.length} subscribers`
        : `${group.group_member_ids.length} members`;
      aboutEl.textContent = group.group_description || "No description";
      tagsEl?.replaceChildren();
      const groupTag = document.createElement("span");
      groupTag.className = "profile-tag";
      groupTag.textContent = group.is_channel ? "Channel" : "Group";
      tagsEl?.append(groupTag);
      if (serverEl) serverEl.textContent = "Server: " + location.host;
      if (activityEl) activityEl.textContent = "";
      peerProfileDialog.showModal();
      return;
    }

    const profile = await api("GET", `/api/users/${encodeURIComponent(id)}/profile`);
    avatar.hidden = !profile.avatar_url;
    if (profile.avatar_url) avatar.src = profile.avatar_url;
    avatar.alt = `${profile.display_name} profile photo`;
    peerProfileBanner.style.backgroundImage = profile.banner_url
      ? 'url("' + profile.banner_url + '?v=' + Date.now() + '")'
      : "";
    nameEl.textContent = profile.display_name;
    usernameEl.textContent = profile.username ? `@${profile.username}` : "";
    aboutEl.textContent = profile.about || "No profile description";
    tagsEl?.replaceChildren();
    for (const tag of Array.isArray(profile.tags) ? profile.tags : []) {
      const chip = document.createElement("span");
      chip.className = "profile-tag";
      chip.textContent = tag;
      tagsEl?.append(chip);
    }
    if (serverEl) serverEl.textContent = "Server: " + (profile.server || location.host);
    if (activityEl) {
      activityEl.replaceChildren();
      renderActivityEntries(activityEl, profile, { compact: false, limit: 5 });
      if (!getUserActivities(profile).length) {
        activityEl.textContent = "No activity";
      }
    }
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

async function uploadProfileBanner(file) {
  const form = new FormData();
  form.append("file", file);
  const response = await fetch("/api/me/banner", {
    method: "POST",
    credentials: "same-origin",
    body: form,
  });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(data.error || response.statusText);
  const url = data.banner_url + "?v=" + Date.now();
  if (profileBanner) profileBanner.style.backgroundImage = 'url("' + url + '")';
  return url;
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