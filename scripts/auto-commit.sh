#!/usr/bin/env bash
set -euo pipefail

REPO="/home/runner/work/kudzu/kudzu/repositories/torvox"
cd "$REPO"

if git diff --quiet && git diff --cached --quiet; then
    exit 0
fi

git add -A

TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")
git commit -m "auto: ${TIMESTAMP}"

REMOTE="https://x-access-token:${GITHUB_TOKEN}@github.com/qzylinra/torvox.git"
git push "$REMOTE" main
