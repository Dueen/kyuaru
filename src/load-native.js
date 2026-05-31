// @ts-check
/// <reference path="../ffi.d.ts" />
"use strict";

import { suffix } from "node:ffi";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { arch, platform } from "node:process";
import { fileURLToPath } from "node:url";

import { KyuaruError } from "./errors.js";

const dir = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

function platformPackageName() {
  if (platform === "darwin" && arch === "arm64") {
    return "@kyuaru/darwin-arm64";
  }
  return null;
}

/**
 * Resolve the absolute path to the native binary for the current platform.
 *
 * @returns {string} absolute path to the shared library
 * @throws {KyuaruError} when no supported native binary is available.
 */
export function resolveNativeBinary() {
  const dev = join(dir, "native", "target", "release", `libkyuaru.${suffix}`);
  if (existsSync(dev)) return dev;

  const pkg = platformPackageName();
  if (pkg) {
    try {
      return require.resolve(pkg);
    } catch {
      throw new KyuaruError({
        code: "UNSUPPORTED_PLATFORM",
        message: `kyuaru: ${pkg} is not installed. Reinstall kyuaru, or build from source with \`pnpm build:native\`.`,
        details: { package: pkg },
      });
    }
  }

  throw new KyuaruError({
    code: "UNSUPPORTED_PLATFORM",
    message: `kyuaru: no native binary is available for ${process.platform}-${process.arch}. Supported platform package: @kyuaru/darwin-arm64.`,
    details: {
      platform: process.platform,
      arch: process.arch,
      supported: ["darwin-arm64"],
    },
  });
}
