# Horizon on iOS (experimental)

This fork's `ios` branch runs Horizon on iPhone. It is upstream Horizon plus a thin iOS layer,
merged with upstream every week.

## Status

- Builds for `aarch64-apple-ios` (iPhone) and `aarch64-apple-ios-sim` (Simulator on Apple Silicon).
- Starts in the Simulator, renders through Metal, loads the prepared game files and plays sound up
  to the title screen.
- Not yet: touch controls, on-screen keyboard, the system clipboard, getting the game files onto a
  phone, a signed build on a phone.

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
```

If the game is installed through Prism Launcher, the Horizon launcher's
`~/Library/Application Support/horizon-launcher/linked-minecraft` links its files in the official
layout, and `horizon-launcher/server-jars/26.3.jar` is the server jar.

The Simulator's GPU is an Apple2-family GPU with no indirect draws, so it draws terrain per
section. Phones from the A11 on draw indirectly, as on a Mac.

## Layout

| Path | What |
|---|---|
| `crates/horizon-ios/` | The app's program: points the game at `Documents/`, then runs `horizon_client::entry::main` |
| `ios/Info.plist` | Landscape, full screen, 120 Hz, `Documents/` shown in the Files app |
| `ios/run-sim.sh` | Build, bundle, sign ad hoc, install and launch in the Simulator |
| `crates/horizon-client-input/src/clipboard_ios.rs` | Stand-in for `arboard`, which has no iOS backend |

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
