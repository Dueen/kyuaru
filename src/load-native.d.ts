/**
 * Resolve the absolute path to the native binary for the current platform.
 *
 * @returns {string} absolute path to the shared library
 * @throws {KyuaruError} when no supported native binary is available.
 */
export function resolveNativeBinary(): string;
