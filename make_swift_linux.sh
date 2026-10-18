#!/bin/sh
set -eu

# Builds what the IrohLib Swift package needs on Linux: the Rust static
# library in IrohLib/Linux/lib and the uniffi C header beside the module map in
# IrohLib/Linux/Iroh. The Apple equivalent is make_swift.sh's xcframework.
#
# The Swift binding is not regenerated here. IrohLib.swift is checked in and
# shared with Apple; this checks that the Rust here still generates it, so a
# Linux build never links a binding against a library it doesn't match.
#
# `ble` stays off unless IROH_FFI_FEATURES asks for it: its backend drives
# CoreBluetooth. Without it the Bluetooth API is still there, and binding a
# Bluetooth endpoint fails at runtime. Cargo still reads the optional
# dependency's manifest, so ../iroh-ble-transport must be checked out (see
# KEEPTALKING.md); its upstream release commit is enough for that.

FEATURES="${IROH_FFI_FEATURES:-}"
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys;print(json.load(sys.stdin)["target_directory"])')
BINDINGS="$TARGET_DIR/swift-linux"

cargo build --release --lib ${FEATURES:+--features "$FEATURES"}
cargo run --bin uniffi-bindgen generate --language swift --out-dir "$BINDINGS" \
    --library "$TARGET_DIR/release/libiroh_ffi.so" --config uniffi.toml

if ! sed "s/iroh_ffiFFI/Iroh/g" "$BINDINGS/iroh_ffi.swift" \
    | cmp -s - IrohLib/Sources/IrohLib/IrohLib.swift; then
    echo "error: IrohLib/Sources/IrohLib/IrohLib.swift is not what this Rust generates." >&2
    echo "Regenerate it (make_swift.sh on a Mac) before building for Linux." >&2
    exit 1
fi

cp "$BINDINGS/iroh_ffiFFI.h" IrohLib/Linux/Iroh/iroh_ffiFFI.h
# The static library only: with a shared one beside it the linker would take
# that instead, and the daemon would need it installed next to it.
rm -rf IrohLib/Linux/lib
mkdir -p IrohLib/Linux/lib
cp "$TARGET_DIR/release/libiroh_ffi.a" IrohLib/Linux/lib/libiroh_ffi.a
echo "IrohLib for Linux: IrohLib/Linux/lib/libiroh_ffi.a"
