/**
 * Determines whether the payload is a {@link KyuaruError}.
 *
 * @param {unknown} error The value to test.
 * @returns {error is KyuaruError} True if the value is a `KyuaruError`.
 */
export function isKyuaruError(error: unknown): error is KyuaruError;
/**
 * @param {number} nativeCode
 * @returns {KyuaruError}
 */
export function mapNativeError(nativeCode: number): KyuaruError;
/**
 * Stable string identifiers for every error condition `kyuaru` can produce.
 *
 * - `OUTPUT_TOO_SHORT` — internal: output buffer too small for the chosen
 *   `maxVersion`. Should never reach user code.
 * - `INVALID_ECC` — `ecl` is not `"L" | "M" | "Q" | "H"`.
 * - `INVALID_VERSION` — `minVersion` / `maxVersion` out of `[1, 40]`, or
 *   `minVersion > maxVersion`.
 * - `INVALID_MASK` — `mask` is not `null` or in `[0, 7]`.
 * - `DATA_TOO_LONG` — input doesn't fit in any version in `[minVersion,
 *   maxVersion]` at the given ECL. Either raise `maxVersion`, drop ECL, or
 *   shrink the input.
 * - `INVALID_UTF8` — `encodeText` only: byte input was not valid UTF-8. Use
 *   `encodeBinary` for raw bytes.
 * - `INPUT_TOO_LONG` — input exceeds the QR-spec cap (7089 chars for text,
 *   2953 bytes for binary).
 * - `UNSUPPORTED_PLATFORM` — the native binary could not be loaded.
 * - `NATIVE_FAILURE` — unexpected failure from the native binding.
 *
 * @typedef {(
 *   | "OUTPUT_TOO_SHORT"
 *   | "INVALID_ECC"
 *   | "INVALID_VERSION"
 *   | "INVALID_MASK"
 *   | "DATA_TOO_LONG"
 *   | "INVALID_UTF8"
 *   | "INPUT_TOO_LONG"
 *   | "UNSUPPORTED_PLATFORM"
 *   | "NATIVE_FAILURE"
 * )} KyuaruErrorCode
 */
/**
 * @typedef {object} KyuaruErrorInit
 * @property {KyuaruErrorCode} code
 * @property {string} message
 * @property {Record<string, unknown>} [details]
 */
/**
 * The single error class thrown by every public-API code path. Branch on
 * {@link KyuaruError#code} — message strings are not part of the public API
 * and may change between minor versions.
 *
 * @example
 * ```js
 * try {
 *   encodeText(input, { maxVersion: 5 });
 * } catch (error) {
 *   if (error instanceof KyuaruError && error.code === "DATA_TOO_LONG") { ... }
 * }
 * ```
 */
export class KyuaruError extends Error {
    /** @param {KyuaruErrorInit} init */
    constructor(init: KyuaruErrorInit);
    /** @type {KyuaruErrorCode} */
    code: KyuaruErrorCode;
    /** @type {Record<string, unknown> | undefined} */
    details: Record<string, unknown> | undefined;
}
/**
 * Stable string identifiers for every error condition `kyuaru` can produce.
 *
 * - `OUTPUT_TOO_SHORT` — internal: output buffer too small for the chosen
 *   `maxVersion`. Should never reach user code.
 * - `INVALID_ECC` — `ecl` is not `"L" | "M" | "Q" | "H"`.
 * - `INVALID_VERSION` — `minVersion` / `maxVersion` out of `[1, 40]`, or
 *   `minVersion > maxVersion`.
 * - `INVALID_MASK` — `mask` is not `null` or in `[0, 7]`.
 * - `DATA_TOO_LONG` — input doesn't fit in any version in `[minVersion,
 *   maxVersion]` at the given ECL. Either raise `maxVersion`, drop ECL, or
 *   shrink the input.
 * - `INVALID_UTF8` — `encodeText` only: byte input was not valid UTF-8. Use
 *   `encodeBinary` for raw bytes.
 * - `INPUT_TOO_LONG` — input exceeds the QR-spec cap (7089 chars for text,
 *   2953 bytes for binary).
 * - `UNSUPPORTED_PLATFORM` — the native binary could not be loaded.
 * - `NATIVE_FAILURE` — unexpected failure from the native binding.
 */
export type KyuaruErrorCode = ("OUTPUT_TOO_SHORT" | "INVALID_ECC" | "INVALID_VERSION" | "INVALID_MASK" | "DATA_TOO_LONG" | "INVALID_UTF8" | "INPUT_TOO_LONG" | "UNSUPPORTED_PLATFORM" | "NATIVE_FAILURE");
export type KyuaruErrorInit = {
    code: KyuaruErrorCode;
    message: string;
    details?: Record<string, unknown> | undefined;
};
