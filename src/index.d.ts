/**
 * Encode a string or UTF-8 byte buffer using the native QR encoder.
 *
 * @param {string | Uint8Array} input String or UTF-8 bytes.
 * @param {EncodeOptions} [options]
 * @returns {QrCode}
 */
export function encodeText(input: string | Uint8Array, options?: EncodeOptions): QrCode;
/**
 * Encode arbitrary bytes using QR byte mode.
 *
 * @param {Uint8Array} input
 * @param {EncodeOptions} [options]
 * @returns {QrCode}
 */
export function encodeBinary(input: Uint8Array, options?: EncodeOptions): QrCode;
/**
 * Encoded QR code returned by the native encoder.
 *
 * @property {number} size Side length in modules.
 * @property {number} version QR version, 1..40.
 * @property {Buffer} data Bit-packed native output: `[size, ...modules]`.
 */
export class QrCode {
    /**
     * @param {Uint8Array} data Bit-packed native output: `[size, ...modules]`.
     */
    constructor(data: Uint8Array);
    /** @returns {number} */
    get size(): number;
    /** @returns {number} */
    get version(): number;
    /** @returns {Buffer} */
    get data(): Buffer;
}
export { KyuaruError } from "./errors.js";
export { isKyuaruError };
export type EncodeOptions = import("./options.js").EncodeOptions;
import { Buffer } from "node:buffer";
import { isKyuaruError } from "./errors.js";
export { MAX_VERSION, MIN_VERSION } from "./options.js";
