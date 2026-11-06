#!/bin/sh
# Builds Horizon for an iPhone, signs it with your Apple ID's team, installs it on the
# phone connected to this Mac, copies the prepared game files into it and starts it.
#
#   ios/run-device.sh                      # the one connected iPhone
#   HORIZON_DEVICE="Jan's iPhone" ios/run-device.sh
#   HORIZON_TEAM=ABCDE12345 ios/run-device.sh
#
# Needs: Xcode signed in to your Apple ID (Settings > Accounts; a free one gives a Personal
# Team whose signature lasts 7 days: run this again to renew it), the phone trusted by this
# Mac and in Developer Mode, and prepared game files in .local/horizon-client/ (docs/ios.md).
# Only changed game files are copied, so after the first run this is quick.
set -eu

cd "$(dirname "$0")/.."
BUNDLE_ID=dev.dblgq.horizon
RESOURCES="${HORIZON_RESOURCES:-.local/horizon-client}"
DERIVED=target/ios-xcode

# The team: HORIZON_TEAM, else the first team of the Apple ID signed in to Xcode.
TEAM="${HORIZON_TEAM:-$(defaults export com.apple.dt.Xcode - 2>/dev/null | python3 -I -c '
import plistlib, sys
prefs = plistlib.loads(sys.stdin.buffer.read())
for teams in prefs.get("IDEProvisioningTeamByIdentifier", {}).values():
    for team in teams:
        if team.get("teamID"):
            print(team["teamID"]); raise SystemExit
' || true)}"
# Else the team of the Apple Development certificate Xcode made at its first signing (its OU).
if [ -z "$TEAM" ]; then
    TEAM=$(security find-certificate -c "Apple Development" -p 2>/dev/null \
        | openssl x509 -noout -subject -nameopt multiline 2>/dev/null \
        | sed -n 's/^ *organizationalUnitName *= *//p' | head -1)
fi
if [ -z "$TEAM" ]; then
    echo "No Apple ID team yet: open ios/Horizon.xcodeproj in Xcode once, pick your Personal Team" >&2
    echo "under Signing & Capabilities and run it on the phone (docs/ios.md), or set HORIZON_TEAM." >&2
    exit 1
fi

# The phone: HORIZON_DEVICE, else the one connected iPhone.
DEVICE="${HORIZON_DEVICE:-$(xcrun devicectl list devices --json-output /dev/stdout 2>/dev/null | python3 -I -c '
import json, sys
text = sys.stdin.read()
data = json.loads(text[text.index("{"):])
phones = [d for d in data["result"]["devices"]
          if d.get("hardwareProperties", {}).get("platform") == "iOS"
          and d.get("hardwareProperties", {}).get("reality") == "physical"
          and d.get("connectionProperties", {}).get("tunnelState") != "unavailable"]
if phones:
    print(phones[0]["identifier"])
' || true)}"
if [ -z "$DEVICE" ]; then
    echo "No iPhone connected: plug it in, unlock it and trust this Mac." >&2
    exit 1
fi

(cd ios && xcodegen generate --quiet --spec project.yml)
xcodebuild -quiet -project ios/Horizon.xcodeproj -scheme Horizon -configuration Release \
    -destination 'generic/platform=iOS' -derivedDataPath "$DERIVED" \
    -allowProvisioningUpdates -allowProvisioningDeviceRegistration \
    HORIZON_TEAM="$TEAM" build
APP="$DERIVED/Build/Products/Release-iphoneos/horizon-ios.app"

xcrun devicectl device install app --device "$DEVICE" "$APP"
if ls "$RESOURCES"/*/horizon-resources.json >/dev/null 2>&1; then
    echo "Copying the game files (only the changed ones)..."
    # The phone's file service runs out of open files partway through some 28,000 files
    # (POSIX error 24); each try skips what is already there, so a few tries finish it.
    tries=0
    until xcrun devicectl device copy to --device "$DEVICE" --domain-type appDataContainer \
        --domain-identifier "$BUNDLE_ID" --source "$RESOURCES" \
        --destination Documents/.local/horizon-client >/dev/null 2>&1; do
        tries=$((tries + 1))
        if [ "$tries" -ge 20 ]; then
            echo "Copying the game files failed 20 times." >&2
            exit 1
        fi
        echo "  partway, again ($tries)..."
    done
else
    echo "No prepared game files in $RESOURCES: the game shows where to get them."
fi
xcrun devicectl device process launch --device "$DEVICE" --terminate-existing "$BUNDLE_ID"
