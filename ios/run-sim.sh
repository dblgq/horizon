#!/bin/sh
# Builds Horizon for the iOS Simulator, wraps it in Horizon.app and starts it.
#
#   ios/run-sim.sh                 # iPhone 16 Pro simulator (the target phone), created on first use
#   HORIZON_SIM="iPhone Air" ios/run-sim.sh
#
# The app's game folder (Documents/) is printed at the end: put the prepared
# resources there as .local/horizon-client/<version>/.
set -eu

cd "$(dirname "$0")/.."
SIM="${HORIZON_SIM:-iPhone 16 Pro}"
BUNDLE_ID="${HORIZON_BUNDLE_ID:-dev.dblgq.horizon}"
TARGET=aarch64-apple-ios-sim
PROFILE="${HORIZON_PROFILE:-fast}"
export IPHONEOS_DEPLOYMENT_TARGET=17.0

cargo build --profile "$PROFILE" --target "$TARGET" -p horizon-ios

APP="target/$TARGET/$PROFILE/Horizon.app"
rm -rf "$APP"
mkdir -p "$APP"
# Apps linked against the iOS 27 SDK must use the UIScene life cycle, which
# winit 0.30 does not; stamped with SDK 26, iOS still starts them the old way.
# Temporary, until winit's iOS backend has scenes (docs/ios.md).
xcrun vtool -set-build-version iossim 17.0 26.0 -replace \
    -output "$APP/horizon-ios" "target/$TARGET/$PROFILE/horizon-ios"
sed -e "s/\$(HORIZON_BUNDLE_ID)/$BUNDLE_ID/" -e 's/\$(HORIZON_PLATFORM)/iPhoneSimulator/' \
    ios/Info.plist > "$APP/Info.plist"
plutil -lint -s "$APP/Info.plist"
# The simulator runs only signed code; an ad-hoc signature is enough there.
codesign --force --sign - --timestamp=none "$APP"

# The simulator device: the named one, created from its device type if missing.
UDID=$(xcrun simctl list devices available -j | python3 -I -c '
import json, sys
name = sys.argv[1]
for devices in json.load(sys.stdin)["devices"].values():
    for d in devices:
        if d["name"] == name:
            print(d["udid"]); raise SystemExit
' "$SIM")
if [ -z "$UDID" ]; then
    TYPE="com.apple.CoreSimulator.SimDeviceType.$(echo "$SIM" | tr ' ' '-')"
    UDID=$(xcrun simctl create "$SIM" "$TYPE")
fi
xcrun simctl boot "$UDID" 2>/dev/null || true
# The simulator's window: Simulator.app, or DeviceHub.app in Xcode 27 and later.
for ui in "$(xcode-select -p)/Applications/Simulator.app" "$(xcode-select -p)/../Applications/DeviceHub.app"; do
    if [ -d "$ui" ]; then
        open "$ui" || true
        break
    fi
done
xcrun simctl bootstatus "$UDID" -b >/dev/null

xcrun simctl install "$UDID" "$APP"
xcrun simctl terminate "$UDID" "$BUNDLE_ID" 2>/dev/null || true
xcrun simctl launch "$UDID" "$BUNDLE_ID"
echo "Game folder: $(xcrun simctl get_app_container "$UDID" "$BUNDLE_ID" data)/Documents"
