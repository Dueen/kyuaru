// @ts-check
/// <reference path="../ffi.d.ts" />
"use strict";

import { Buffer } from "node:buffer";
import { DynamicLibrary, types } from "node:ffi";
import { TextEncoder } from "node:util";

import { KyuaruError, isKyuaruError, mapNativeError } from "./errors.js";
import { resolveNativeBinary } from "./load-native.js";
import { normalizeOptions } from "./options.js";

/** @typedef {import("./options.js").EncodeOptions} EncodeOptions */

const NATIVE_SIGNATURE = {
  parameters: [
    types.BUFFER,
    types.UINT_32,
    types.UINT_8,
    types.UINT_8,
    types.UINT_8,
    types.INT_8,
    types.UINT_8,
    types.BUFFER,
    types.UINT_32,
  ],
  result: types.INT_32,
};

const library = new DynamicLibrary(resolveNativeBinary());
const nativeEncodeText = library.getFunction("kyuaru_encode_text_utf8", NATIVE_SIGNATURE);
const nativeEncodeBinary = library.getFunction("kyuaru_encode_binary", NATIVE_SIGNATURE);
const textEncoder = new TextEncoder();

/**
 * Encoded QR code returned by the native encoder.
 *
 * @property {number} size Side length in modules.
 * @property {number} version QR version, 1..40.
 * @property {Buffer} data Bit-packed native output: `[size, ...modules]`.
 */
export class QrCode {
  /** @returns {number} */
  get size() {
    throw new TypeError("QrCode data is only available on constructed instances");
  }

  /** @returns {number} */
  get version() {
    throw new TypeError("QrCode data is only available on constructed instances");
  }

  /** @returns {Buffer} */
  get data() {
    throw new TypeError("QrCode data is only available on constructed instances");
  }

  /**
   * @param {Uint8Array} data Bit-packed native output: `[size, ...modules]`.
   */
  constructor(data) {
    if (!(data instanceof Uint8Array)) {
      throw new TypeError("QrCode data must be a Uint8Array");
    }

    if (data.byteLength === 0) {
      throw new KyuaruError({
        code: "INVALID_VERSION",
        message: "QrCode data must start with a valid QR size byte",
      });
    }

    const buffer = Buffer.allocUnsafeSlow(data.byteLength);
    buffer.set(data);
    const size = buffer[0];
    if (!isValidSize(size)) {
      throw new KyuaruError({
        code: "INVALID_VERSION",
        message: `QrCode size byte must be one of 21, 25, ..., 177; got ${size}`,
      });
    }

    const expectedLength = bufferLengthForSize(size);
    if (buffer.byteLength !== expectedLength) {
      throw new KyuaruError({
        code: "NATIVE_FAILURE",
        message: `QrCode data length must be ${expectedLength} bytes for size ${size}; got ${buffer.byteLength}`,
      });
    }

    Object.defineProperties(this, {
      size: { enumerable: true, value: size },
      version: { enumerable: true, value: (size - 17) >> 2 },
      data: { enumerable: true, value: buffer },
    });
  }
}

/**
 * Encode a string or UTF-8 byte buffer using the native QR encoder.
 *
 * @param {string | Uint8Array} input String or UTF-8 bytes.
 * @param {EncodeOptions} [options]
 * @returns {QrCode}
 */
export function encodeText(input, options = {}) {
  const bytes = typeof input === "string" ? textEncoder.encode(input) : assertBytes(input, "input must be a string or Uint8Array");
  return new QrCode(callNative(nativeEncodeText, bytes, options));
}

/**
 * Encode arbitrary bytes using QR byte mode.
 *
 * @param {Uint8Array} input
 * @param {EncodeOptions} [options]
 * @returns {QrCode}
 */
export function encodeBinary(input, options = {}) {
  return new QrCode(callNative(nativeEncodeBinary, assertBytes(input, "input must be a Uint8Array"), options));
}

/**
 * @param {Function} native
 * @param {Uint8Array} bytes
 * @param {EncodeOptions} options
 * @returns {Buffer}
 */
function callNative(native, bytes, options) {
  const opts = normalizeOptions(options);
  const data = toBuffer(bytes);
  const out = Buffer.allocUnsafe(bufferLengthForSize(opts.maxVersion * 4 + 17));
  const size = native(
    data,
    data.byteLength,
    opts.ecl,
    opts.minVersion,
    opts.maxVersion,
    opts.mask,
    opts.boostEcl,
    out,
    out.byteLength,
  );
  if (size < 0) throw mapNativeError(size);
  return out.subarray(0, bufferLengthForSize(size));
}

/**
 * @param {number} size
 * @returns {number}
 */
function bufferLengthForSize(size) {
  return Math.ceil((size * size) / 8) + 1;
}

/**
 * @param {unknown} size
 * @returns {size is number}
 */
function isValidSize(size) {
  return typeof size === "number" && Number.isInteger(size) && size >= 21 && size <= 177 && (size - 17) % 4 === 0;
}

/**
 * @param {Uint8Array} input
 * @returns {Buffer}
 */
function toBuffer(input) {
  if (Buffer.isBuffer(input)) return input;
  return Buffer.from(input.buffer, input.byteOffset, input.byteLength);
}

/**
 * @param {unknown} input
 * @param {string} message
 * @returns {Uint8Array}
 */
function assertBytes(input, message) {
  if (input instanceof Uint8Array) return input;
  throw new TypeError(message);
}

export { KyuaruError } from "./errors.js";
export { isKyuaruError };
export { MAX_VERSION, MIN_VERSION } from "./options.js";
