#!/bin/sh
# gh-pages 브랜치(GitHub Pages)를 갱신하고 푸시한다. CI 전용.
# 사용: publish.sh           사이트(site/, 설치 스크립트, 공개키)만 갱신
#       publish.sh <.deb>    deb도 서명된 flat APT 저장소에 추가 (환경: APT_SIGNING_KEY, ASCII armor 비밀키)
set -eu
repo=$(git rev-parse --show-toplevel)
site=$(mktemp -d)
deb=${1:+$(realpath "$1")}

trap 'git -C "$repo" worktree remove --force "$site"' EXIT
if git fetch -q origin gh-pages 2>/dev/null; then
    git worktree add -q -B gh-pages "$site" FETCH_HEAD
else
    git worktree add -q --orphan -b gh-pages "$site"
fi

cp "$repo"/site/* "$repo/packaging/apt/hangul-setting.gpg" "$repo/packaging/apt/install.sh" "$site/"
touch "$site/.nojekyll"

if [ -n "$deb" ]; then
    export GNUPGHOME=$(mktemp -d)
    mkdir -p "$site/pool"
    cp "$deb" "$site/pool/"
    cd "$site"
    apt-ftparchive packages pool > Packages
    gzip -9kf Packages
    apt-ftparchive \
        -o APT::FTPArchive::Release::Origin=hangul-setting \
        -o APT::FTPArchive::Release::Label=hangul-setting \
        -o APT::FTPArchive::Release::Suite=stable \
        -o APT::FTPArchive::Release::Codename=stable \
        -o APT::FTPArchive::Release::Architectures=amd64 \
        release . > "$GNUPGHOME/Release"
    mv "$GNUPGHOME/Release" Release
    printf '%s\n' "$APT_SIGNING_KEY" | gpg --batch --quiet --import
    gpg --batch --yes --clearsign -o InRelease Release
    gpg --batch --yes -abs -o Release.gpg Release
fi

cd "$site"
git add -A
if git diff --cached --quiet; then
    echo "gh-pages: 바뀐 것 없음"
    exit 0
fi
git -c user.name="github-actions[bot]" -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
    commit -q -m "pages: ${deb:+$(basename "$deb") }$(git -C "$repo" rev-parse --short HEAD)"
git push -q origin gh-pages
