# kyuaru

Minimal QR encoding for Node.js, backed by a native Rust encoder through
Node's experimental `node:ffi` API.

## Requirements

- Node.js 26 or newer.
- Run Node with `--experimental-ffi`.
- v0.1.0 ships a native package for `darwin-arm64` only:
  `@kyuaru/darwin-arm64`.

`kyuaru` loads the native binding when the module is imported. On unsupported
platforms, or when the optional native package is missing, import fails with a
`KyuaruError` using `code === "UNSUPPORTED_PLATFORM"`.

For local development, the loader checks a local Rust release build first:

```sh
pnpm build:native
node --experimental-ffi your-file.js
```

If no local build exists, it falls back to the installed platform package.

## Install

```sh
npm install kyuaru
```

## Usage

```js
import { encodeText, encodeBinary, isKyuaruError } from "kyuaru";

try {
  const qr = encodeText("HELLO WORLD", { ecl: "M" });
  console.log(qr.size); // 21
  console.log(qr.version); // 1
  console.log(qr.data); // Buffer: [size, ...packed modules]
} catch (error) {
  if (isKyuaruError(error)) {
    console.error(error.code);
  }
}

const binary = encodeBinary(new Uint8Array([0xff, 0xfe]));
console.log(binary.size);
```

## API

### `encodeText(input, options?)`

Encodes a string or UTF-8 `Uint8Array`.

Invalid UTF-8 bytes throw `KyuaruError` with `code === "INVALID_UTF8"`.

### `encodeBinary(input, options?)`

Encodes raw bytes. `input` must be a `Uint8Array`.

### Options

```ts
type EncodeOptions = {
  ecl?: "L" | "M" | "Q" | "H";
  minVersion?: number;
  maxVersion?: number;
  mask?: 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | null;
  boostEcl?: boolean;
};
```

Defaults:

- `ecl`: `"M"`
- `minVersion`: `1`
- `maxVersion`: `40`
- `mask`: omitted or `null` means auto-select
- `boostEcl`: `false`

### `QrCode`

Both encoder functions return a `QrCode`.

- `qr.size`: side length in modules, from `21` to `177`
- `qr.version`: QR version, from `1` to `40`
- `qr.data`: stable packed wire format

The `qr.data` property is readonly, but the `Buffer` contents are mutable. The
first byte is the side length. The remaining bytes store modules row-major, one
bit per module, least-significant bit first within each byte. A set bit is a
dark module.

`qr.data[0] === qr.size`.

### Errors

`KyuaruError#code` is the stable field to branch on. Messages and `details` are
diagnostic.

```js
import { KyuaruError, isKyuaruError } from "kyuaru";

try {
  encodeText("too much data", { maxVersion: 1 });
} catch (error) {
  if (isKyuaruError(error) && error.code === "DATA_TOO_LONG") {
    // retry with a larger maxVersion
  }
}
```
