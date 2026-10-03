#!/usr/bin/env bash
set -euo pipefail

required=(
  APPLE_CERTIFICATE
  APPLE_CERTIFICATE_PASSWORD
  APPLE_SIGNING_IDENTITY
  APPLE_API_ISSUER
  APPLE_API_KEY
  APPLE_API_KEY_CONTENT
)

missing=0
for name in "${required[@]}"; do
  if [ -z "${!name:-}" ]; then
    echo "::error::${name} is required in the production-release environment."
    missing=1
  fi
done
if [ "$missing" -ne 0 ]; then
  exit 1
fi

case "$APPLE_SIGNING_IDENTITY" in
  "Developer ID Application:"*) ;;
  *)
    echo "::error::APPLE_SIGNING_IDENTITY must be a Developer ID Application identity for outside-the-App-Store distribution."
    exit 1
    ;;
esac

command -v openssl >/dev/null
command -v security >/dev/null
xcrun --find notarytool >/dev/null
xcrun --find stapler >/dev/null

keychain="$RUNNER_TEMP/opendownloader-signing.keychain-db"
certificate="$RUNNER_TEMP/opendownloader-signing.p12"
api_key="$RUNNER_TEMP/AuthKey_${APPLE_API_KEY}.p8"
keychain_password="$(openssl rand -hex 24)"

rm -f "$keychain" "$certificate" "$api_key"
umask 077

if ! printf '%s' "$APPLE_CERTIFICATE" | tr -d '[:space:]' | openssl base64 -d -A -out "$certificate"; then
  echo "::error::APPLE_CERTIFICATE must be a single base64-encoded .p12 containing the Developer ID Application certificate and private key."
  exit 1
fi

printf '%s\n' "$APPLE_API_KEY_CONTENT" > "$api_key"
if ! grep -q '^-----BEGIN PRIVATE KEY-----' "$api_key"; then
  echo "::error::APPLE_API_KEY_CONTENT must contain the raw App Store Connect AuthKey_<KEY_ID>.p8 contents, not base64."
  exit 1
fi

security create-keychain -p "$keychain_password" "$keychain"
security default-keychain -s "$keychain"
security unlock-keychain -p "$keychain_password" "$keychain"
security set-keychain-settings -lut 21600 "$keychain"
security import "$certificate" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -f pkcs12 -k "$keychain"
security list-keychains -d user -s "$keychain"
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$keychain_password" "$keychain"

if ! security find-identity -v -p codesigning "$keychain" | grep -F -- "$APPLE_SIGNING_IDENTITY" >/dev/null; then
  echo "::error::The imported .p12 does not expose the configured APPLE_SIGNING_IDENTITY."
  exit 1
fi

# Apple recommends the notarytool history command as a non-destructive credential test.
if ! xcrun notarytool history --key "$api_key" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER" >/dev/null; then
  echo "::error::App Store Connect notarization credentials were rejected by notarytool."
  exit 1
fi

rm -f "$certificate"
echo "APPLE_API_KEY_PATH=$api_key" >> "$GITHUB_ENV"
echo "Apple signing and notarization credential preflight passed."
