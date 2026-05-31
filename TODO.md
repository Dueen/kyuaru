# TODO

## Deferred native performance work

### Auto-mask scoring

Current code: `src/native/lib.rs`, `QrCode::encode_codewords`.

The auto-mask path currently evaluates each candidate by mutating the full QR
matrix with `apply_mask`, drawing format bits, scoring the full matrix, then
reverting the same mask before testing the next candidate.

Keep this deferred for v0.1.0 unless benchmarks show mask scoring is release
blocking. Before changing it, add a benchmark that isolates auto-mask selection
for representative small, medium, and version-40 payloads.

Candidate approaches:

- Score candidates against a scratch matrix instead of mutating and reverting
  the output matrix.
- Make penalty scoring evaluate a virtual mask for data modules, leaving the
  matrix unchanged during candidate evaluation.

Constraints:

- Preserve exact output for explicit masks.
- Preserve deterministic auto-mask choices for the current test fixtures unless
  a measured penalty tie makes the previous choice arbitrary.
- Keep format-bit handling correct during scoring.
- Do not allocate per row or per module in the scoring loop.

Acceptance criteria:

- `cargo test --manifest-path src/native/Cargo.toml`
- `cargo clippy --manifest-path src/native/Cargo.toml -- -D warnings`
- Node FFI tests still pass after a release build.
- Benchmark shows the new path improves auto-mask selection or reduces mutation
  work without regressing small payloads.
