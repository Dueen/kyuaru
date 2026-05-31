import { KyuaruError } from "./errors.js";

/**
 * Error-correction level. Higher levels recover more occluded modules at the
 * cost of larger QR codes for the same payload.
 *
 * - `"L"` — recovers ~7% of damaged modules.
 * - `"M"` — ~15% *(default)*.
 * - `"Q"` — ~25%.
 * - `"H"` — ~30%.
 *
 * @typedef {"L" | "M" | "Q" | "H"} ErrorCorrectionLevel
 */

/**
 * User-facing encoder options. Every field is optional; defaults are
 * documented per field.
 *
 * @typedef {object} EncodeOptions
 * @property {ErrorCorrectionLevel} [ecl] Error-correction level. Default: `"M"`.
 * @property {number} [minVersion] Minimum QR version to consider. 1..40, default 1.
 * @property {number} [maxVersion] Maximum QR version to consider. 1..40, default 40.
 * @property {0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | null} [mask] Force a
 *   specific mask pattern. `null` or omitted = auto-select for lowest
 *   penalty score.
 * @property {boolean} [boostEcl] If true and the input fits at a higher ECL
 *   within the chosen version, automatically use the higher ECL. Default: false.
 */

/**
 * Internal normalised form of {@link EncodeOptions}, with defaults applied
 * and types converted for the native binding.
 *
 * @typedef {object} NormalizedOptions
 * @property {number} ecl
 * @property {number} minVersion
 * @property {number} maxVersion
 * @property {number} mask
 * @property {number} boostEcl
 */

/** Minimum supported QR version (smallest: 21×21 modules). */
export const MIN_VERSION = 1;
/** Maximum supported QR version (largest: 177×177 modules). */
export const MAX_VERSION = 40;

/** @type {Record<ErrorCorrectionLevel, number>} */
const ECL_MAP = { L: 0, M: 1, Q: 2, H: 3 };

/**
 * @param {EncodeOptions} options
 * @returns {NormalizedOptions}
 */
export function normalizeOptions(options) {
  if (options === null || typeof options !== "object") {
    throw new TypeError("options must be an object");
  }

  const minVersion = options.minVersion ?? MIN_VERSION;
  const maxVersion = options.maxVersion ?? MAX_VERSION;

  if (!Number.isInteger(minVersion) || minVersion < MIN_VERSION || minVersion > MAX_VERSION) {
    throw new KyuaruError({
      code: "INVALID_VERSION",
      message: `minVersion must be an integer in [${MIN_VERSION}, ${MAX_VERSION}], got ${minVersion}`,
    });
  }
  if (!Number.isInteger(maxVersion) || maxVersion < MIN_VERSION || maxVersion > MAX_VERSION) {
    throw new KyuaruError({
      code: "INVALID_VERSION",
      message: `maxVersion must be an integer in [${MIN_VERSION}, ${MAX_VERSION}], got ${maxVersion}`,
    });
  }
  if (minVersion > maxVersion) {
    throw new KyuaruError({
      code: "INVALID_VERSION",
      message: `minVersion (${minVersion}) must be <= maxVersion (${maxVersion})`,
    });
  }

  const maskInput = options.mask;
  let mask = -1;
  if (maskInput !== null && maskInput !== undefined) {
    if (!Number.isInteger(maskInput) || maskInput < 0 || maskInput > 7) {
      throw new KyuaruError({
        code: "INVALID_MASK",
        message: `mask must be null or an integer in [0, 7]; got ${maskInput}`,
      });
    }
    mask = maskInput;
  }

  const eclKey = options.ecl ?? "M";
  const ecl = ECL_MAP[eclKey];
  if (ecl === undefined) {
    throw new KyuaruError({
      code: "INVALID_ECC",
      message: `ecl must be 'L', 'M', 'Q', or 'H'; got ${eclKey}`,
    });
  }

  return {
    ecl,
    minVersion,
    maxVersion,
    mask,
    boostEcl: Number(Boolean(options.boostEcl)),
  };
}
