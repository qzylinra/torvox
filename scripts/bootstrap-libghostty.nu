#!/usr/bin/env -S nix develop --command nu

const GHOSTTY_REPO: string = "https://github.com/ghostty-org/ghostty.git"
const GHOSTTY_COMMIT: string = "bfe633a9487892ff3d27ed727db540267f22ef90"

def main [] {
    let root = (git rev-parse --show-toplevel | str trim)
    let src_dir = $"($root)/.ghostty"
    let stamp = $src_dir + "/.ghostty-commit"
    let patch = $"($root)/patches/libghostty-vt-correctness.patch"

    let skip = ($stamp | path exists) and ((open $stamp | str trim) == $GHOSTTY_COMMIT)

    if not $skip {
        if ($src_dir | path exists) { rm -rf $src_dir }
        print $"Cloning ghostty ($GHOSTTY_COMMIT)..." >&2
        mkdir $src_dir
        git clone --filter=blob:none --no-checkout $GHOSTTY_REPO $src_dir
        git -C $src_dir checkout $GHOSTTY_COMMIT

        if ($patch | path exists) {
            print "Applying zig correctness patches..." >&2
            patch -d $src_dir -p1 --input $patch --forward
        }
        $GHOSTTY_COMMIT | save -f $stamp
    }

    print $src_dir
}
