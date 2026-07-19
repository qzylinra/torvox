#!/usr/bin/env bash
# Wrapper for dylint-link that ensures RUSTUP_TOOLCHAIN is set.
# dylint unsets RUSTUP_TOOLCHAIN when building lint libraries, but
# dylint-link needs it to find the nightly sysroot.
#
# This wrapper walks up from the cwd to find a rust-toolchain or
# rust-toolchain.toml file and sets RUSTUP_TOOLCHAIN from it.

find_toolchain() {
    local dir="$1"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/rust-toolchain" ]; then
            head -1 "$dir/rust-toolchain" | tr -d '\n'
            return 0
        fi
        if [ -f "$dir/rust-toolchain.toml" ]; then
            grep 'channel' "$dir/rust-toolchain.toml" 2>/dev/null | \
                sed 's/.*"\(.*\)".*/\1/' | tr -d '\n'
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

if [ -z "${RUSTUP_TOOLCHAIN:-}" ]; then
    TOOLCHAIN=$(find_toolchain "$PWD")
    if [ -n "$TOOLCHAIN" ]; then
        export RUSTUP_TOOLCHAIN="$TOOLCHAIN"
    fi
fi

exec dylint-link "$@"
