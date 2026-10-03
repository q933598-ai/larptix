import {
  DecryptionSettings,
  DeviceId,
  DeviceLists,
  EncryptionSettings,
  OlmMachine,
  RequestType,
  RoomId,
  TrustRequirement,
  UserId,
  initAsync,
} from "/matrix-crypto-pkg/index.mjs";

const matrixCryptoReady = initAsync();

const MATRIX_ROOM_ALGORITHM = "m.megolm.v1.aes-sha2";

function formatMatrixDecryptionError(err) {
  const code = err?.code;
  const description = err?.description;
  const withheld = err?.withheldCode || err?.maybe_withheld;
  const parts = [
    code && "code=" + code,
    description,
    withheld && "withheld=" + withheld,
  ].filter(Boolean);
  return parts.length ? parts.join(": ") : (err?.message || String(err));
}

function encodedStoreName(userId, deviceId) {
  return "larptrix-matrix-" + encodeURIComponent(userId) + "-" + encodeURIComponent(deviceId);
}

function responseJson(value) {
  return JSON.stringify(value ?? {});
}

function parseJson(raw, label) {
  try {
    return JSON.parse(raw);
  } catch {
    throw new Error(label + " returned invalid JSON.");
  }
}

function requestTypeName(type) {
  for (const [name, value] of Object.entries(RequestType)) {
    if (type === value) return name;
  }
  return String(type);
}

export class LarptrixMatrixCrypto {
  constructor({ api, userId, serverName, deviceId, storePassphrase }) {
    this.api = api;
    this.internalUserId = userId;
    this.serverName = serverName;
    this.matrixUserId = "@" + userId + ":" + serverName;
    this.deviceIdValue = deviceId;
    this.storePassphrase = storePassphrase;
    this.machine = null;
    this.processingRequests = null;
    this.processingToDevice = Promise.resolve();
    this.processedToDeviceIds = new Set();
    this.decryptionSettings = null;
    this.encryptionSettings = null;
  }

  get deviceId() {
    return this.deviceIdValue;
  }

  get userId() {
    return this.matrixUserId;
  }

  async initialize() {
    await matrixCryptoReady;
    this.decryptionSettings = new DecryptionSettings(TrustRequirement.Untrusted);
    this.encryptionSettings = new EncryptionSettings();
    this.machine = await OlmMachine.initialize(
      new UserId(this.matrixUserId),
      new DeviceId(this.deviceIdValue),
      encodedStoreName(this.matrixUserId, this.deviceIdValue),
      this.storePassphrase,
    );

    this.machine.roomKeyRequestsEnabled = true;
    this.machine.roomKeyForwardingEnabled = true;

    // The first device-key upload is generated when the SDK processes a
    // regular sync with zero known one-time-key counts. Using only the
    // MSC4186 helper here treats a missing count as "unchanged", so a brand
    // new device may never publish itself to the server.
    await this.machine.receiveSyncChanges(
      JSON.stringify([]),
      new DeviceLists(),
      new Map(),
      undefined,
    );
    await this.processOutgoingRequests();
    await this.processPendingToDevice();
    return this;
  }

  async close() {
    this.machine?.close();
    this.machine = null;
  }

  async processOutgoingRequests() {
    if (!this.machine) return;
    if (this.processingRequests) return this.processingRequests;

    this.processingRequests = (async () => {
      for (;;) {
        const requests = await this.machine.outgoingRequests();
        if (!requests.length) break;

        for (const request of requests) {
          const response = await this.sendOutgoingRequest(request);
          await this.machine.markRequestAsSent(request.id, request.type, response);
        }
      }
    })();

    try {
      await this.processingRequests;
    } finally {
      this.processingRequests = null;
    }
  }

  async sendOutgoingRequest(request) {
    const type = request.type;

    if (type === RequestType.KeysUpload) {
      const body = parseJson(request.body, "Matrix keys upload");
      const response = await this.api("POST", "/api/matrix/keys/upload", {
        device_id: this.deviceIdValue,
        request: body,
      });
      return responseJson(response);
    }

    if (type === RequestType.KeysQuery) {
      const body = parseJson(request.body, "Matrix keys query");
      const response = await this.api("POST", "/api/matrix/keys/query", body);
      return responseJson(response);
    }

    if (type === RequestType.KeysClaim) {
      const body = parseJson(request.body, "Matrix keys claim");
      const response = await this.api("POST", "/api/matrix/keys/claim", body);
      return responseJson(response);
    }

    if (type === RequestType.ToDevice) {
      const body = parseJson(request.body, "Matrix to-device");
      const response = await this.api("POST", "/api/matrix/to-device", {
        device_id: this.deviceIdValue,
        event_type: request.event_type,
        txn_id: request.txn_id,
        messages: body.messages ?? {},
      });
      return responseJson(response);
    }

    throw new Error("Unsupported Matrix crypto request: " + requestTypeName(type));
  }

  async processPendingToDevice() {
    this.processingToDevice = this.processingToDevice.then(async () => {
      const result = await this.api("GET", "/api/matrix/to-device/pending");
      const events = Array.isArray(result?.events) ? result.events : [];
      const ackIds = [];

      for (const event of events) {
        try {
          const processed = await this.processToDeviceEvent(event);
          if (processed) ackIds.push(event.id);
        } catch (err) {
          // Keep a failed event queued. Other pending room keys must still
          // be processed, especially when several messages were sent while
          // this device was offline.
          console.error("[E2E] pending Matrix to-device event failed", {
            eventId: event?.id,
            eventType: event?.event_type,
            error: err?.message || String(err),
          });
        }
      }

      if (ackIds.length) {
        await this.api("POST", "/api/matrix/to-device/ack", { event_ids: ackIds });
      }
    });

    return this.processingToDevice;
  }

  async processToDeviceEvent(event) {
    if (!this.machine || !event?.id) return false;
    if (event.recipient_device_id !== this.deviceIdValue) return false;
    if (this.processedToDeviceIds.has(event.id)) return true;

    const rawEvent = {
      type: event.event_type,
      sender: event.sender_id.startsWith("@")
        ? event.sender_id
        : "@" + event.sender_id + ":" + this.serverName,
      content: event.content,
      unsigned: {
        transaction_id: event.txn_id,
      },
    };

    await this.machine.receiveSyncChangesMsc4186(
      JSON.stringify([rawEvent]),
      new DeviceLists(),
      new Map(),
      new Set(),
    );

    // Do not acknowledge/mark the event as processed until every
    // crypto response generated from it has been successfully uploaded.
    // Otherwise a transient network failure can make a room key event look
    // consumed while its forwarding request was never delivered.
    await this.processOutgoingRequests();
    this.processedToDeviceIds.add(event.id);
    return true;
  }

  async handleLiveToDevice(event) {
    this.processingToDevice = this.processingToDevice.then(async () => {
      if (event?.recipient_device_id !== this.deviceIdValue) return;
      const processed = await this.processToDeviceEvent(event);
      if (processed) {
        await this.api("POST", "/api/matrix/to-device/ack", {
          event_ids: [event.id],
        });
      }
    });
    return this.processingToDevice;
  }

  async roomIdForDm(peerInternalUserId) {
    const pair = [this.internalUserId, peerInternalUserId].sort().join("\n");
    const digest = await crypto.subtle.digest(
      "SHA-256",
      new TextEncoder().encode(this.serverName + "\n" + pair),
    );
    const hex = [...new Uint8Array(digest)]
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join("");
    return "!larpdm_" + hex + ":" + this.serverName;
  }

  groupRoomId(groupId) {
    return "!larpgrp_" + groupId + ":" + this.serverName;
  }

  async prepareRoom(roomId, participantInternalUserIds) {
    if (!this.machine) throw new Error("Matrix E2E is not initialized.");

    // The WASM SDK takes ownership of UserId objects passed to several
    // methods and invalidates those instances afterwards. Keep a reusable
    // set of UserId objects here and pass clones to every consuming call.
    const externalUsers = participantInternalUserIds
      .filter((id) => id && id !== this.internalUserId)
      .map((id) => new UserId("@" + id + ":" + this.serverName));

    await this.machine.updateTrackedUsers(
      externalUsers.map((user) => user.clone()),
    );

    if (externalUsers.length) {
      const keyQuery = this.machine.queryKeysForUsers(
        externalUsers.map((user) => user.clone()),
      );
      const keyQueryResponse = await this.sendOutgoingRequest(keyQuery);
      await this.machine.markRequestAsSent(
        keyQuery.id,
        keyQuery.type,
        keyQueryResponse,
      );
    }

    await this.processOutgoingRequests();

    const roomUsersForMissingSessions = [
      ...externalUsers.map((user) => user.clone()),
      new UserId(this.matrixUserId),
    ];
    const missingSessions = await this.machine.getMissingSessions(
      roomUsersForMissingSessions,
    );
    if (missingSessions) {
      const response = await this.sendOutgoingRequest(missingSessions);
      await this.machine.markRequestAsSent(
        missingSessions.id,
        missingSessions.type,
        response,
      );
    }

    const room = new RoomId(roomId);
    const roomUsersForShare = [
      ...externalUsers.map((user) => user.clone()),
      new UserId(this.matrixUserId),
    ];
    const requests = await this.machine.shareRoomKey(
      room,
      roomUsersForShare,
      this.encryptionSettings,
    );

    for (const request of requests) {
      const response = await this.sendOutgoingRequest(request);
      await this.machine.markRequestAsSent(request.id, request.type, response);
    }

    await this.processOutgoingRequests();
  }

  async encrypt(roomId, payload) {
    if (!this.machine) throw new Error("Matrix E2E is not initialized.");
    const room = new RoomId(roomId);
    const content = typeof payload === "string" ? payload : JSON.stringify(payload);
    const encrypted = await this.machine.encryptRoomEvent(
      room,
      "m.room.message",
      content,
    );
    return JSON.parse(encrypted);
  }

  async decrypt(roomId, message) {
    if (!this.machine) throw new Error("Matrix E2E is not initialized.");
    const ciphertext = message.body?.ciphertext;
    if (!ciphertext || typeof ciphertext !== "object") {
      throw new Error("Matrix encrypted message is malformed.");
    }

    const event = {
      event_id: "$" + message.id,
      sender: "@" + message.sender_id + ":" + this.serverName,
      origin_server_ts: message.created_at,
      room_id: roomId,
      type: "m.room.encrypted",
      content: ciphertext,
    };

    try {
      const result = await this.machine.decryptRoomEvent(
        JSON.stringify(event),
        new RoomId(roomId),
        this.decryptionSettings,
      );
      return JSON.parse(result.event);
    } catch (err) {
      // A missing Megolm room key should enqueue Matrix-style room-key
      // recovery requests. Flush them immediately through our transport.
      await this.processOutgoingRequests().catch(() => {});
      const error = new Error(formatMatrixDecryptionError(err));
      error.code = err?.code;
      error.description = err?.description;
      error.withheldCode = err?.withheldCode;
      error.maybeWithheld = err?.maybe_withheld;
      throw error;
    }
  }

  async refreshTrackedUser(internalUserId) {
    if (!this.machine || !internalUserId || internalUserId === this.internalUserId) return;
    await this.machine.updateTrackedUsers([
      new UserId("@" + internalUserId + ":" + this.serverName),
    ]);
    await this.processOutgoingRequests();
  }

  algorithm() {
    return MATRIX_ROOM_ALGORITHM;
  }
}

export function matrixDeviceIdStorageKey(userId) {
  return "larptrix_matrix_device_id:" + userId;
}

export function getOrCreateMatrixDeviceId(userId) {
  const storageKey = matrixDeviceIdStorageKey(userId);
  let deviceId = localStorage.getItem(storageKey);
  if (!deviceId) {
    deviceId = crypto.randomUUID();
    localStorage.setItem(storageKey, deviceId);
  }
  return deviceId;
}

export function matrixUserIdFromInternalId(userId, serverName) {
  return "@" + userId + ":" + serverName;
}