import assert from "node:assert/strict";
import { createCipheriv, createDecipheriv, createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const fixtureUrl = new URL("./fixtures/matrix-interop.json", import.meta.url);
const fixture = JSON.parse(await readFile(fileURLToPath(fixtureUrl), "utf8"));

function browserDmRoomId(serverName, localUserId, peerUserId) {
  const pair = [localUserId, peerUserId].sort().join("\n");
  const digest = createHash("sha256")
    .update(serverName + "\n" + pair, "utf8")
    .digest("hex");
  return `!larpdm_${digest}:${serverName}`;
}

function browserGroupRoomId(groupId, serverName) {
  return `!larpgrp_${groupId}:${serverName}`;
}

function browserEnvelope(senderDeviceId, roomId, ciphertext) {
  return {
    version: 3,
    message_type: "matrix",
    sender_device_id: senderDeviceId,
    room_id: roomId,
    ciphertext,
  };
}

function browserEncryptedEvent(messageId, senderId, createdAt, roomId, serverName, ciphertext) {
  return {
    event_id: "$" + messageId,
    sender: "@" + senderId + ":" + serverName,
    origin_server_ts: createdAt,
    room_id: roomId,
    type: "m.room.encrypted",
    content: ciphertext,
  };
}

// Golden vectors shared with the native Rust unit tests. These tests verify
// the browser/native wire contract, not a live Olm/Megolm network exchange.
assert.equal(
  browserGroupRoomId(fixture.group_id, fixture.server_name),
  fixture.group_room_id,
);
assert.equal(
  browserDmRoomId(fixture.server_name, fixture.local_user_id, fixture.peer_user_id),
  fixture.room_id,
);
assert.equal(
  browserDmRoomId(fixture.server_name, fixture.peer_user_id, fixture.local_user_id),
  fixture.room_id,
);

assert.equal(JSON.stringify(fixture.payload), fixture.payload_json);
assert.deepEqual(JSON.parse(fixture.payload_json), fixture.payload);

assert.deepEqual(
  browserEnvelope(fixture.sender_device_id, fixture.room_id, fixture.ciphertext),
  fixture.envelope,
);
assert.deepEqual(
  browserEncryptedEvent(
    fixture.message_id,
    fixture.sender_id,
    fixture.created_at,
    fixture.room_id,
    fixture.server_name,
    fixture.ciphertext,
  ),
  fixture.encrypted_event,
);

assert.equal(Buffer.from(fixture.recovery_key, "base64").length, 32);

const attachment = fixture.attachment_crypto;
const attachmentKey = Buffer.from(attachment.key, "base64");
const attachmentIv = Buffer.from(attachment.iv, "base64");
const attachmentCiphertext = Buffer.from(attachment.ciphertext, "base64");
const decipher = createDecipheriv("aes-256-gcm", attachmentKey, attachmentIv);
decipher.setAuthTag(attachmentCiphertext.subarray(-16));
const attachmentPlaintext = Buffer.concat([
  decipher.update(attachmentCiphertext.subarray(0, -16)),
  decipher.final(),
]);
assert.equal(attachmentPlaintext.toString("utf8"), attachment.plaintext);

const cipher = createCipheriv("aes-256-gcm", attachmentKey, attachmentIv);
const browserCompatibleCiphertext = Buffer.concat([
  cipher.update(Buffer.from(attachment.plaintext, "utf8")),
  cipher.final(),
  cipher.getAuthTag(),
]);
assert.deepEqual(browserCompatibleCiphertext, attachmentCiphertext);

console.log("Matrix and AES-GCM browser/native interoperability fixtures passed.");
