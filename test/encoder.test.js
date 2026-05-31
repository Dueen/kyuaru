// @ts-check
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  encodeBinary,
  encodeText,
  isKyuaruError,
  KyuaruError,
  QrCode,
} from "../src/index.js";

test("encodeText returns a QrCode with native data", () => {
  const qr = encodeText("HELLO WORLD");
  assert.ok(qr instanceof QrCode);
  assert.ok(Buffer.isBuffer(qr.data));
  assert.equal(qr.size, 21);
  assert.equal(qr.version, 1);
  assert.equal(qr.data[0], qr.size);
  assert.equal(qr.data.length, bufferLengthForSize(qr.size));
  assert.equal(qr.data.buffer.byteLength, qr.data.length);
});

test("encodeText with ECL 'H' yields a larger version than ECL 'L' for the same input", () => {
  const long = "x".repeat(200);
  const low = encodeText(long, { ecl: "L" });
  const high = encodeText(long, { ecl: "H" });
  assert.ok(high.version > low.version, `expected H>${low.version}, got H=${high.version}`);
});

test("encodeText accepts Uint8Array input", () => {
  const bytes = new TextEncoder().encode("HELLO WORLD");
  const qr = encodeText(bytes);
  assert.equal(qr.size, 21);
});

test("encodeText respects minVersion", () => {
  const qr = encodeText("hi", { minVersion: 10 });
  assert.ok(qr.version >= 10);
});

test("encodeText throws when invalid UTF-8 supplied", () => {
  const invalid = new Uint8Array([0xff, 0xfe]);
  assert.throws(() => encodeText(invalid), KyuaruError);
});

test("encodeBinary accepts the same bytes encodeText rejects as non-UTF-8", () => {
  const qr = encodeBinary(new Uint8Array([0xff, 0xfe]));
  assert.ok(qr instanceof QrCode);
  assert.ok(Buffer.isBuffer(qr.data));
  assert.ok(qr.size >= 21 && qr.size <= 177);
});

test("encodeBinary rejects string input", () => {
  assert.throws(() => encodeBinary(/** @type {any} */ ("hi")), TypeError);
});

test("encodeBinary handles empty input", () => {
  const qr = encodeBinary(new Uint8Array());
  assert.equal(qr.size, 21);
});

test("native buffer can be read as a bit-packed matrix", () => {
  const qr = encodeText("HELLO WORLD");
  assert.equal(getModule(qr, 0, 0), true);
  assert.equal(getModule(qr, 7, 7), false);
});

test("encodeText rejects minVersion > maxVersion", () => {
  assert.throws(() => encodeText("hi", { minVersion: 5, maxVersion: 3 }), KyuaruError);
});

test("encodeText rejects non-object options", () => {
  assert.throws(() => encodeText("hi", /** @type {any} */ (null)), TypeError);
});

test("encodeText rejects mask outside [0, 7]", () => {
  // @ts-expect-error - allow mask outside valid range for testing
  assert.throws(() => encodeText("hi", { mask: 8 }), KyuaruError);
  // @ts-expect-error - allow mask outside valid range for testing
  assert.throws(() => encodeText("hi", { mask: -1 }), KyuaruError);
});

test("encodeText throws when input exceeds capacity for the chosen version range", () => {
  assert.throws(() => encodeText("x".repeat(8000), { maxVersion: 5 }), KyuaruError);
});

test("encodeText treats null and undefined masks as auto mask", () => {
  const omitted = encodeText("hi");
  const explicitUndefined = encodeText("hi", { mask: undefined });
  const explicitNull = encodeText("hi", { mask: null });
  assert.equal(omitted.size, explicitUndefined.size);
  assert.equal(omitted.size, explicitNull.size);
});

test("QrCode validates and copies public constructor input", () => {
  const source = encodeText("HELLO WORLD").data;
  const qr = new QrCode(source);
  assert.deepEqual(qr.data, source);
  source[1] ^= 1;
  assert.notDeepEqual(qr.data, source);
  assert.throws(() => {
    // @ts-expect-error - attempt to mutate read-only property
    qr.size = 25;
  }, TypeError);
});

test("QrCode rejects malformed wire data", () => {
  assert.throws(() => new QrCode(new Uint8Array()), KyuaruError);
  assert.throws(() => new QrCode(new Uint8Array([22])), KyuaruError);
  assert.throws(() => new QrCode(new Uint8Array([21])), KyuaruError);
});

test("isKyuaruError detects KyuaruError instances", () => {
  assert.throws(
    () => encodeText("hi", { maxVersion: 41 }),
    (error) => isKyuaruError(error),
  );
  assert.equal(isKyuaruError(new Error("not a KyuaruError")), false);
});

/**
 * @param {QrCode} qr
 * @param {number} x
 * @param {number} y
 * @returns {boolean}
 */
function getModule(qr, x, y) {
  const index = y * qr.size + x;
  const byte = qr.data[1 + (index >> 3)];
  return byte !== undefined && ((byte >> (index & 7)) & 1) !== 0;
}

/**
 * @param {number} size
 * @returns {number}
 */
function bufferLengthForSize(size) {
  return Math.ceil((size * size) / 8) + 1;
}
