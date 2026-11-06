# Horizon on iOS (experimental)

This fork's `ios` branch runs Horizon on iPhone. It is upstream Horizon plus a thin iOS layer,
merged with upstream every week.

## Status

- Builds for `aarch64-apple-ios` (iPhone) and `aarch64-apple-ios-sim` (Simulator on Apple Silicon).
- Starts in the Simulator, renders through Metal, loads the prepared game files, plays sound and
  plays with touch: menus by tapping and dragging (touches are the mouse), the world with
  Vasysik's on-screen joystick and buttons, and a back button (Escape) over any screen in a world.
- Runs on an iPhone (16 Pro, iOS 26.5) signed with a free Apple ID: `ios/run-device.sh`.
- Text fields (chat, signs, books, searches) get the system keyboard.
- Not yet: aiming by touch (the view aims from the centre), the layout editor and touch
  settings, the system clipboard, safe areas (the Dynamic Island).

## Run in the Simulator

Needs Xcode (with the iOS platform) and Rust with the iOS targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/run-sim.sh                           # iPhone 16 Pro simulator
HORIZON_SIM="iPhone Air" ios/run-sim.sh  # another one
```

The script builds `horizon-ios`, wraps it in `Horizon.app` (`ios/Info.plist`), installs it,
copies the prepared game files into it and prints the app's game folder. The game looks for them
in `Documents/.local/horizon-client/<version>/`. Without them it shows the "start from the
launcher" screen.

### The game files

They are made on the Mac from your own Minecraft: Java Edition, once per Minecraft version, into
`.local/horizon-client/` (ignored by git: never commit them). `horizon-prepare` needs Java 25 with
`javac`. The official launcher's `java-runtime-epsilon` has both:

```sh
cargo build --profile fast -p horizon-prepare
export JAVA_HOME="$HOME/Library/Application Support/minecraft/runtime/java-runtime-epsilon/mac-os-arm64/java-runtime-epsilon/jre.bundle/Contents/Home"
MC="$HOME/Library/Application Support/minecraft"   # or the Horizon launcher's linked-minecraft
target/fast/horizon-prepare --prepare --minecraft-dir "$MC" --server-jar <26.3 server jar>
target/fast/horizon-prepare sounds --minecraft-dir "$MC"
target/fast/horizon-prepare --check                 # silent and exit 0 when complete
python3 tools/ios_controls.py                       # the touch controls' button art
```

The touch controls draw Mojang's Bedrock button art (`gui/bedrock/` sprites). `tools/ios_controls.py`
downloads it from Mojang/bedrock-samples at the revision `ios/bedrock-controls.json` pins, checks each
file's SHA-256 and puts it with the prepared files. It is Mojang's, under Mojang's terms: never
commit it. Without it the controls work but are invisible.

If the game is installed through Prism Launcher, the Horizon launcher's
`~/Library/Application Support/horizon-launcher/linked-minecraft` links its files in the official
layout, and `horizon-launcher/server-jars/26.3.jar` is the server jar.

The Simulator's GPU is an Apple2-family GPU with no indirect draws, so it draws terrain per
section. Phones from the A11 on draw indirectly, as on a Mac.

## Run on an iPhone

```sh
brew install xcodegen
ios/run-device.sh
```

It makes `ios/Horizon.xcodeproj` from `ios/project.yml`, builds and signs the app (Xcode makes
the provisioning profile), installs it on the connected iPhone, copies `.local/horizon-client/`
into it (only changed files after the first time) and starts it.

The first time, once: sign in to your Apple ID in Xcode (Settings > Accounts), connect the phone
and trust the Mac, open `ios/Horizon.xcodeproj`, choose your Personal Team under Signing &
Capabilities and press Run with the phone selected. Xcode then has a signing certificate, which
`run-device.sh` takes the team from (or set `HORIZON_TEAM`). On the phone: turn on Developer Mode
(Settings > Privacy & Security) and trust your Apple ID (Settings > General > VPN & Device
Management).

A free Apple ID's signature lasts 7 days: run `ios/run-device.sh` again to renew it. Saves stay.

## Layout

| Path | What |
|---|---|
| `crates/horizon-ios/` | The app's program: points the game at `Documents/`, then runs `horizon_client::entry::main` |
| `ios/Info.plist` | Landscape, full screen, 120 Hz, `Documents/` shown in the Files app |
| `ios/run-sim.sh` | Build, bundle, sign ad hoc, install and launch in the Simulator |
| `ios/project.yml`, `ios/run-device.sh` | The phone build: XcodeGen's project, and build, sign, install, copy the game files, launch |
| `crates/horizon-ios/build.rs` | 8 MB of main-thread stack (iOS gives 1 MB; the renderer's start overflowed it) |
| `crates/horizon-client-input/src/clipboard_ios.rs` | Stand-in for `arboard`, which has no iOS backend |
| `crates/horizon-client-input/src/touch_mouse.rs` | Touches as the mouse outside play |
| `crates/horizon-client-render/src/touch.rs`, `touch/` | Touch controls from Vasysik/horizon-android (MIT), unchanged |
| `crates/horizon-client/src/game/touch_play.rs` | The controls in play (merged with the pads' state in `poll_pads`) and the back button |
| `ios/bedrock-controls.json`, `tools/ios_controls.py` | The controls' art: names and checksums, and the downloader |

Edits to upstream files stay small `cfg(target_os = "ios")` hooks, so weekly merges rarely conflict.

## Known issues

- **UIScene.** Apps linked with the iOS 27 SDK must use the UIScene life cycle, and winit 0.30 does
  not. `run-sim.sh` stamps the binary as built with SDK 26 (`vtool -set-build-version`), so iOS starts
  it the old way. iOS 26 devices do not enforce this.
- The game draws over the whole screen; menus and the HUD do not yet keep clear of the Dynamic Island
  and the home indicator (winit's safe area).

## Weekly update

```sh
git fetch upstream
git switch main && git merge --ff-only upstream/main && git push origin main
git switch ios  && git merge main      # a real merge, never squash or rebase
ios/run-sim.sh                         # builds and starts: fix any hook upstream moved
git push origin ios
```
