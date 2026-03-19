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
hdiutil create \
  -volname "${APP}" \
  -srcfolder "${APP_DIR}" \
  -ov -format UDRW "dist/rw.dmg"

MOUNT=$(hdiutil attach "dist/rw.dmg" | tail -1 | awk '{print $NF}')
ln -s /Applications "${MOUNT}/Applications"
cp "${ICON_SRC}" "${MOUNT}/.VolumeIcon.icns"
SetFile -a C "${MOUNT}"
hdiutil detach "${MOUNT}"
hdiutil convert "dist/rw.dmg" -format UDZO -o "${DMG_OUT}"
rm "dist/rw.dmg"

echo ""
echo "Done: ${DMG_OUT}"
echo "   $(du -sh ${DMG_OUT} | cut -f1)"
