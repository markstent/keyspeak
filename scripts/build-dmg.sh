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

cp "target/release/keyspeak"   "${APP_DIR}/Contents/MacOS/KeySpeak"
cp "build/macos/Info.plist"    "${APP_DIR}/Contents/Info.plist"
sed -i '' "s/1\.0\.0/${VERSION}/g" "${APP_DIR}/Contents/Info.plist"

echo "▶ Creating DMG..."
hdiutil create \
  -volname "${APP}" \
  -srcfolder "${APP_DIR}" \
  -ov -format UDRW "dist/rw.dmg"

MOUNT=$(hdiutil attach "dist/rw.dmg" | tail -1 | awk '{print $NF}')
ln -s /Applications "${MOUNT}/Applications"
hdiutil detach "${MOUNT}"
hdiutil convert "dist/rw.dmg" -format UDZO -o "${DMG_OUT}"
rm "dist/rw.dmg"

echo ""
echo "Done: ${DMG_OUT}"
echo "   $(du -sh ${DMG_OUT} | cut -f1)"
