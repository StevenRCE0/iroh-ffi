# KeepTalking fork

`main` of this fork is the vendored copy of `iroh-ffi` that the KeepTalking
SDK builds against (`.package(path: "../iroh-ffi")`, product `IrohLib`).
`upstream` is `n0-computer/iroh-ffi`.

## Differences from upstream

- iroh pinned to **1.3.0** in `Cargo.lock`, matching the Rust `kt-sfu`
  hub + relay it talks to (`KeepTalkingSFU`, branch `iroh-sfu`).
- **Bluetooth LE** (`ble` feature, on by default): `EndpointOptions.ble`
  adds `iroh-ble-transport` as a custom transport (with its dedup hook and
  address lookup), and `Endpoint.bleStatus()` reports the adapter, byte
  counters and nearby devices (with their advertised key prefix). The
  package links CoreBluetooth.
- **Vendored `iroh-ble-transport`** in `vendor/iroh-ble-transport`
  (0.5.1-beta.5, upstream [mcginty/iroh-ble-transport] `77b5eb9`). The
  first vendor commit is verbatim; KeepTalking patches follow as separate
  commits:
  - an identity characteristic (`69726f06-…`, read-only) serving the full
    32-byte endpoint id — adverts carry only a 12-byte prefix, which can't
    be dialled, so a peer that never learned the id elsewhere reads it
    there;
  - `BlePeerInfo.prefix`: each device's advertised key prefix.

  `iroh-ble-transport` and its `blew` backend are **AGPL-3.0**. KeepTalking
  complies with the AGPL, so shipped builds may include them;
  `--no-default-features` builds without Bluetooth.

[mcginty/iroh-ble-transport]: https://github.com/mcginty/iroh-ble-transport

Future changes stay here rather than upstream: relay-only endpoints
(`clear_ip_transports`), a path-selection policy, and custom transports
(`unstable-custom-transports`) for the offline mesh.

## Building the xcframework

```bash
RUSTUP_TOOLCHAIN=stable ./make_swift.sh
```

- Needs the stable toolchain with `aarch64-apple-ios`, `aarch64-apple-ios-sim`,
  `x86_64-apple-ios`, `aarch64-apple-darwin` and `aarch64-apple-ios-macabi`.
  A nightly from early 2026 failed to load proc-macro crates when
  cross-compiling for iOS.
- `Iroh.xcframework` stays gitignored, so build it before resolving the
  package. `Package.swift` always points at the local build here (upstream
  falls back to its release zip when none exists). The v1.1.0 zip no longer
  matches: `IrohLib.swift` is regenerated from this branch's Rust and includes
  `preset_iroh_services`, which the zip lacks. Publishing our own zip is the
  fix once the SDK consumes this fork by URL.
