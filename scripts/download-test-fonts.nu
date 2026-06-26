# Download test fonts for CJK rendering verification
# Usage: nu scripts/download-test-fonts.nu [--output_dir <path>]

def main [--output_dir: string = "/tmp/test-fonts"] {
    mkdir $output_dir

    # MapleMonoNormal-NF-CN (the specific font requested)
    let maple_nf_zip = ($output_dir | path join "MapleMonoNormal-NF-CN.zip")
    if not ($maple_nf_zip | path exists) {
        print "Downloading MapleMonoNormal-NF-CN..."
        http get "https://github.com/subframe7536/maple-font/releases/download/v7.9/MapleMonoNormal-NF-CN.zip" | save --raw $maple_nf_zip
    } else {
        print "MapleMonoNormal-NF-CN already downloaded"
    }

    let maple_nf_dir = ($output_dir | path join "maple-normal-nf-cn")
    if not ($maple_nf_dir | path exists) {
        print "Extracting MapleMonoNormal-NF-CN..."
        unzip -o $maple_nf_zip -d $maple_nf_dir
    }

    # MapleMono CN (standard variant)
    let maple_zip = ($output_dir | path join "MapleMono-CN.zip")
    if not ($maple_zip | path exists) {
        print "Downloading MapleMono CN..."
        http get "https://github.com/subframe7536/maple-font/releases/download/v7.9/MapleMono-CN.zip" | save --raw $maple_zip
    } else {
        print "MapleMono CN already downloaded"
    }

    let maple_dir = ($output_dir | path join "maple")
    if not ($maple_dir | path exists) {
        print "Extracting Maple..."
        unzip -o $maple_zip -d $maple_dir
    }

    # Source Han Sans SC
    let shan_zip = ($output_dir | path join "SourceHanSansSC.zip")
    if not ($shan_zip | path exists) {
        print "Downloading Source Han Sans SC..."
        http get "https://github.com/adobe-fonts/source-han-sans/releases/download/2.005R/09_SourceHanSansSC.zip" | save --raw $shan_zip
    } else {
        print "Source Han Sans SC already downloaded"
    }

    let shan_dir = ($output_dir | path join "source-han")
    if not ($shan_dir | path exists) {
        print "Extracting Source Han Sans..."
        unzip -o $shan_zip -d $shan_dir
    }

    print "\nFonts available:"
    let maple_nf_medium = ($maple_nf_dir | path join "MapleMonoNormal-NF-CN-Medium.ttf")
    if ($maple_nf_medium | path exists) {
        print $"  MapleMonoNormal-NF-CN-Medium: ($maple_nf_medium)"
    } else {
        print $"  WARNING: MapleMonoNormal-NF-CN-Medium not found"
    }
    let maple_ttf = ($maple_dir | path join "MapleMono-CN-Regular.ttf")
    if ($maple_ttf | path exists) {
        print $"  Maple Mono CN Regular: ($maple_ttf)"
    } else {
        print $"  WARNING: Maple Mono CN Regular not found"
    }
    let shan_otf = ($shan_dir | path join "OTF" | path join "SimplifiedChinese" | path join "SourceHanSansSC-Regular.otf")
    if ($shan_otf | path exists) {
        print $"  Source Han Sans SC: ($shan_otf)"
    } else {
        print $"  WARNING: Source Han Sans SC not found"
    }

    print "\nDone! Fonts downloaded to: $output_dir"
}
