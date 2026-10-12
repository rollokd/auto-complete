#!/usr/bin/env bash
# Builds build/Auto.app. Env: AUTO_SIGN_IDENTITY (default "Auto Local Signing" if present, else ad-hoc), SKIP_WEB=1, SKIP_SPECS=1
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Honour rust-toolchain.toml via rustup instead of Homebrew's cargo
export PATH="$HOME/.cargo/bin:$PATH"

APP_NAME="Auto"
BUNDLE_ID="dev.rollo.auto"
VERSION="$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
DEFAULT_IDENTITY="-"
security find-identity -v -p codesigning 2>/dev/null | grep -q '"Auto Local Signing"' && DEFAULT_IDENTITY="Auto Local Signing"
SIGN_IDENTITY="${AUTO_SIGN_IDENTITY:-$DEFAULT_IDENTITY}"
APP="$ROOT/build/$APP_NAME.app"

step() { printf '\n\033[1;35m==> %s\033[0m\n' "$*"; }

if [[ -z "${SKIP_WEB:-}" ]]; then
  step "Building web apps"
  pnpm install --frozen-lockfile
  pnpm exec turbo build --ui=stream --filter=@amzn/autocomplete-app --filter=@amzn/dashboard-app
fi

if [[ -z "${SKIP_SPECS:-}" || ! -f specs/build/index.json ]]; then
  step "Compiling completion specs"
  (cd specs && pnpm --ignore-workspace install --frozen-lockfile --ignore-scripts && pnpm exec autocomplete-tools compile)
fi

step "Building Rust binaries"
cargo build --release -p fig_desktop -p figterm -p q_cli -p fig_input_method

CLI_BIN="target/release/auto"
[[ -x "$CLI_BIN" ]] || CLI_BIN="target/release/q_cli"

step "Assembling $APP"
rm -rf "$APP"
C="$APP/Contents"
mkdir -p "$C/MacOS" "$C/Resources" "$C/Helpers"

cp target/release/fig_desktop "$C/MacOS/auto_desktop"
cp "$CLI_BIN" "$C/MacOS/auto"
cp target/release/figterm "$C/MacOS/autoterm"

cp crates/fig_desktop/icons/icon.icns "$C/Resources/icon.icns"
cp -R packages/autocomplete-app/dist "$C/Resources/autocomplete"
cp -R packages/dashboard-app/dist "$C/Resources/dashboard"
cp -R specs/build "$C/Resources/specs"
cp -R specs/icons "$C/Resources/specs/icons"
cp -R themes "$C/Resources/themes"

IME="$C/Helpers/AutoInputMethod.app/Contents"
mkdir -p "$IME/MacOS"
cp target/release/fig_input_method "$IME/MacOS/fig_input_method"
cp crates/fig_input_method/Info.plist "$IME/Info.plist"
cp -R crates/fig_input_method/resources "$IME/Resources"

cat > "$C/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>$APP_NAME</string>
  <key>CFBundleExecutable</key><string>auto_desktop</string>
  <key>CFBundleIconFile</key><string>icon.icns</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>$APP_NAME</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>CFBundleURLTypes</key>
  <array>
    <dict>
      <key>CFBundleURLName</key><string>$BUNDLE_ID</string>
      <key>CFBundleURLSchemes</key><array><string>auto</string></array>
    </dict>
  </array>
</dict>
</plist>
PLIST

step "Signing (identity: $SIGN_IDENTITY)"
sign() { codesign --force --timestamp=none -s "$SIGN_IDENTITY" "$@"; }
sign "$C/Helpers/AutoInputMethod.app"
sign "$C/MacOS/auto" "$C/MacOS/autoterm"
sign --entitlements crates/fig_desktop/entitlements.plist "$APP"
codesign --verify --strict "$APP"

step "Done: $APP ($VERSION)"
