#!/bin/bash
set -euo pipefail

echo "Uninstalling KeySpeak..."

# Remove the app
if [ -d "/Applications/KeySpeak.app" ]; then
    rm -rf "/Applications/KeySpeak.app"
    echo "  Removed /Applications/KeySpeak.app"
else
    echo "  KeySpeak.app not found in Applications"
fi

# Reset Accessibility permission (requires password)
echo ""
echo "  Resetting Accessibility permission for KeySpeak..."
echo "  (You may be prompted for your password)"
tccutil reset Accessibility app.keyspeak.mac 2>/dev/null || true

echo ""
echo "Done. You can now install a new version."
echo "After installing, grant Accessibility access again in:"
echo "  System Settings > Privacy & Security > Accessibility"
