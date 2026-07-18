#!/usr/bin/env -S nix develop --command nu
# Auto-add, commit, and push every 30 minutes

let repo = "/home/runner/work/kudzu/kudzu/repositories/torvox"
let remote = "https://x-access-token:$($env.GITHUB_TOKEN)@github.com/qzylinra/torvox.git"

cd $repo

let has_changes = (git status --porcelain | length) > 0
if not $has_changes {
    exit 0
}

git add -A

let timestamp = (date now | format date "%Y-%m-%d %H:%M:%S")
git commit -m $"auto: ($timestamp)"

git push $remote main
