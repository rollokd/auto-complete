#!/usr/bin/env bash
# Creates a self-signed code signing identity so macOS keeps Accessibility/IME permissions across rebuilds.
# Usage: scripts/create-signing-cert.sh, then build with AUTO_SIGN_IDENTITY="Auto Local Signing"
set -euo pipefail

NAME="Auto Local Signing"
KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

if security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "\"$NAME\" already exists"
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

cat > "$TMP/cert.cnf" <<EOF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $NAME
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
EOF

openssl req -x509 -newkey rsa:2048 -nodes -days 3650 -config "$TMP/cert.cnf" \
  -keyout "$TMP/key.pem" -out "$TMP/cert.pem" 2>/dev/null
openssl pkcs12 -export -legacy -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
  -name "$NAME" -passout pass:auto -out "$TMP/cert.p12" 2>/dev/null \
  || openssl pkcs12 -export -inkey "$TMP/key.pem" -in "$TMP/cert.pem" \
    -name "$NAME" -passout pass:auto -out "$TMP/cert.p12"

security import "$TMP/cert.p12" -k "$KEYCHAIN" -P auto -T /usr/bin/codesign
# Prompts for your login password to trust the cert for code signing
security add-trusted-cert -p codeSign -k "$KEYCHAIN" "$TMP/cert.pem"

security find-identity -v -p codesigning | grep "$NAME"
echo "Created. Build with: AUTO_SIGN_IDENTITY=\"$NAME\" pnpm app"
