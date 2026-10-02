# KeepTalking fork

`main` of this fork is the vendored copy of `iroh-ffi` that the KeepTalking
SDK builds against (`.package(path: "../iroh-ffi")`, product `IrohLib`).
`upstream` is `n0-computer/iroh-ffi`.

## Differences from upstream

Meant to stay a small superset of upstream.

Upstreamable as they are:

- `EndpointOptions.clear_ip_transports`: drop the UDP/IP transports, so an
  endpoint uses only its relay and custom transports.
- `set_log_sink(directives, sink, stderr)`: route filtered Rust `tracing`
  output to a foreign `LogSink`. `set_log_level` no longer panics when a
  subscriber is already installed.
- `Endpoint.network_change()`: tell iroh the network may have changed. On
  Apple platforms iroh misses most changes by itself (its sleep check reads
  a clock that stops during sleep), so the host calls this from its path
  monitor and on return from the background. Upstream's
  `watch_network_change` is broken (it loops on this trigger and fires
  continuously); it is left as is and must not be used.

Fork-only:

- iroh pinned to **1.3.0** in `Cargo.lock`, matching the Rust `kt-sfu`
  (`KeepTalkingSFU`, branch `iroh-sfu`).
- **Bluetooth LE** — the `ble` feature, off by default and turned on by
  `make_swift.sh` (override with `IROH_FFI_FEATURES`).
  - `EndpointOptions.ble` adds `iroh-ble-transport` as a custom transport,
    with its dedup hook and address lookup. It requires `relay_mode`
    disabled and `clear_ip_transports`: a Bluetooth-only endpoint, because
    the dedup hook only promotes a pipe whose handshake ran over it.
  - `Endpoint.bleStatus()` reports the adapter, the radio state, byte
    counters and nearby devices with their advertised key prefix.
  - `Endpoint.bleSetRadioActive(_:)` pauses or resumes scanning and
    advertising. The transport can't be torn down, so this is how a
    Bluetooth endpoint goes quiet; a failed bind pauses it too.
  - The Swift package links CoreBluetooth.
- `Package.swift` always uses the local `Iroh.xcframework`.

`iroh-ble-transport` is a sibling checkout (`../iroh-ble-transport`) of a
fork of [mcginty/iroh-ble-transport], branch `keeptalking`. That branch
starts at the 0.5.1-beta.5 release (`77b5eb9`) and adds:

- a read-only identity characteristic (`69726f06-…`) serving the full
  32-byte endpoint id. Adverts carry only a 12-byte prefix, which can't be
  dialled, so a peer that never learned the id elsewhere reads it there;
- each device's advertised key prefix in `BlePeerInfo`;
- `pause_radio` / `resume_radio`.

These are candidates for upstream PRs there. Once the fork is published,
`Cargo.toml` pins it by git rev instead of the sibling path.

`iroh-ble-transport` and its `blew` backend are **AGPL-3.0**. KeepTalking
complies with the AGPL, so shipped builds may include them.

[mcginty/iroh-ble-transport]: https://github.com/mcginty/iroh-ble-transport

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
