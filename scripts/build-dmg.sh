#!/bin/bash
set -euo pipefail

APP="KeySpeak"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= "\(.*\)"/\1/')
APP_DIR="dist/${APP}.app"
DMG_OUT="dist/${APP}-${VERSION}.dmg"

echo "▶ Building KeySpeak v${VERSION} (release)..."
cargo build --release

rm -rf dist/
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "target/release/keyspeak"          "${APP_DIR}/Contents/MacOS/keyspeak"
cp "target/release/keyspeak-settings" "${APP_DIR}/Contents/MacOS/keyspeak-settings"
cp "build/macos/Info.plist"    "${APP_DIR}/Contents/Info.plist"
sed -i '' "s/1\.0\.0/${VERSION}/g" "${APP_DIR}/Contents/Info.plist"

ICON_SRC="assets/KeySpeak.icns"
cp "${ICON_SRC}" "${APP_DIR}/Contents/Resources/AppIcon.icns"

echo "▶ Self-signing app bundle..."
codesign --force --deep --sign "KeySpeak Developer" "${APP_DIR}"

echo "▶ Creating DMG..."
# Use HFS+ so SetFile -a C works for custom volume icon
SIZE_KB=$(du -sk "${APP_DIR}" | cut -f1)
SIZE_MB=$(( (SIZE_KB / 1024) + 10 ))
hdiutil create \
  -volname "${APP}" \
  -size "${SIZE_MB}m" \
  -fs HFS+ \
  -ov "dist/rw.dmg"

MOUNT=$(hdiutil attach "dist/rw.dmg" | tail -1 | awk '{print $NF}')
cp -R "${APP_DIR}" "${MOUNT}/"
ln -s /Applications "${MOUNT}/Applications"
cp "${ICON_SRC}" "${MOUNT}/.VolumeIcon.icns"
SetFile -a C "${MOUNT}"
hdiutil detach "${MOUNT}"
hdiutil convert "dist/rw.dmg" -format UDZO -o "${DMG_OUT}"
rm "dist/rw.dmg"

# Set custom icon on the DMG file itself
ICON_ABS="$(cd "$(dirname "${ICON_SRC}")" && pwd)/$(basename "${ICON_SRC}")"
DMG_ABS="$(cd "$(dirname "${DMG_OUT}")" && pwd)/$(basename "${DMG_OUT}")"
osascript -e "
use framework \"AppKit\"
set ws to current application's NSWorkspace's sharedWorkspace()
set iconImage to current application's NSImage's alloc()'s initWithContentsOfFile:\"${ICON_ABS}\"
ws's setIcon:iconImage forFile:\"${DMG_ABS}\" options:0
"

echo ""
echo "Done: ${DMG_OUT}"
echo "   $(du -sh ${DMG_OUT} | cut -f1)"
