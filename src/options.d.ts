/**
 * @param {EncodeOptions} options
 * @returns {NormalizedOptions}
 */
export function normalizeOptions(options: EncodeOptions): NormalizedOptions;
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
export const MIN_VERSION: 1;
/** Maximum supported QR version (largest: 177×177 modules). */
export const MAX_VERSION: 40;
/**
 * Error-correction level. Higher levels recover more occluded modules at the
 * cost of larger QR codes for the same payload.
 *
 * - `"L"` — recovers ~7% of damaged modules.
 * - `"M"` — ~15% *(default)*.
 * - `"Q"` — ~25%.
 * - `"H"` — ~30%.
 */
export type ErrorCorrectionLevel = "L" | "M" | "Q" | "H";
/**
 * User-facing encoder options. Every field is optional; defaults are
 * documented per field.
 */
export type EncodeOptions = {
    /**
     * Error-correction level. Default: `"M"`.
     */
    ecl?: ErrorCorrectionLevel | undefined;
    /**
     * Minimum QR version to consider. 1..40, default 1.
     */
    minVersion?: number | undefined;
    /**
     * Maximum QR version to consider. 1..40, default 40.
     */
    maxVersion?: number | undefined;
    /**
     * Force a
     * specific mask pattern. `null` or omitted = auto-select for lowest
     * penalty score.
     */
    mask?: 0 | 2 | 1 | 3 | 4 | 5 | 6 | 7 | null | undefined;
    /**
     * If true and the input fits at a higher ECL
     * within the chosen version, automatically use the higher ECL. Default: false.
     */
    boostEcl?: boolean | undefined;
};
/**
 * Internal normalised form of {@link EncodeOptions}, with defaults applied
 * and types converted for the native binding.
 */
export type NormalizedOptions = {
    ecl: number;
    minVersion: number;
    maxVersion: number;
    mask: number;
    boostEcl: number;
};
