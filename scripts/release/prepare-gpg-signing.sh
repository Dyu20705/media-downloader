#!/usr/bin/env bash
set -euo pipefail

if [ -z "${RELEASE_GPG_PRIVATE_KEY:-}" ]; then
  echo "::error::RELEASE_GPG_PRIVATE_KEY is required in the production-release environment."
  exit 1
fi
if [ -z "${RELEASE_GPG_PASSPHRASE:-}" ]; then
  echo "::error::RELEASE_GPG_PASSPHRASE is required in the production-release environment."
  exit 1
fi

command -v gpg >/dev/null

gnupg_home="$RUNNER_TEMP/opendownloader-release-gnupg"
rm -rf "$gnupg_home"
install -d -m 700 "$gnupg_home"
export GNUPGHOME="$gnupg_home"

printf '%s' "$RELEASE_GPG_PRIVATE_KEY" | gpg --batch --import

primary_count="$(gpg --batch --with-colons --list-secret-keys | awk -F: '$1 == "sec" { count++ } END { print count + 0 }')"
if [ "$primary_count" -ne 1 ]; then
  echo "::error::RELEASE_GPG_PRIVATE_KEY must contain exactly one primary secret key."
  exit 1
fi

fingerprint="$(gpg --batch --with-colons --list-secret-keys | awk -F: '$1 == "sec" { want=1; next } want && $1 == "fpr" { print $10; exit }')"
if [ -z "$fingerprint" ]; then
  echo "::error::Unable to determine the release GPG key fingerprint."
  exit 1
fi

probe="$RUNNER_TEMP/opendownloader-gpg-preflight.txt"
signature="$probe.sig"
printf 'openDownloader release signing preflight\n' > "$probe"
printf '%s' "$RELEASE_GPG_PASSPHRASE" | gpg --batch --yes --pinentry-mode loopback --passphrase-fd 0 --local-user "$fingerprint" --detach-sign --output "$signature" "$probe"
gpg --verify "$signature" "$probe"

echo "GNUPGHOME=$gnupg_home" >> "$GITHUB_ENV"
echo "RELEASE_GPG_FINGERPRINT=$fingerprint" >> "$GITHUB_ENV"
echo "Release GPG signing preflight passed."
