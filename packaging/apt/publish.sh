#!/bin/sh
# .deb를 gh-pages 브랜치의 서명된 flat APT 저장소에 추가하고 푸시한다. CI 전용.
# 사용: publish.sh <.deb>   환경: APT_SIGNING_KEY(ASCII armor 비밀키)
set -eu
deb=$(realpath "$1")
repo=$(git rev-parse --show-toplevel)
site=$(mktemp -d)
export GNUPGHOME=$(mktemp -d)

if git fetch -q origin gh-pages; then
    git worktree add -q -B gh-pages "$site" FETCH_HEAD
else
    git worktree add -q --orphan -b gh-pages "$site"
fi

mkdir -p "$site/pool"
cp "$deb" "$site/pool/"
cp "$repo/packaging/apt/hangul-setting.gpg" "$repo/packaging/apt/install.sh" "$site/"
touch "$site/.nojekyll"

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

git add -A
git -c user.name="github-actions[bot]" -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
    commit -q -m "apt: $(basename "$deb")"
git push -q origin gh-pages
