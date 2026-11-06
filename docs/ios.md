# Horizon on iOS (experimental)

This fork's `ios` branch runs Horizon on iPhone. It is upstream Horizon plus a thin iOS layer,
merged with upstream every week.

## Status

- Builds for `aarch64-apple-ios` (iPhone) and `aarch64-apple-ios-sim` (Simulator on Apple Silicon).
- Starts in the Simulator, renders through Metal and shows the game's own screens.
- Not yet: touch controls, on-screen keyboard, the system clipboard, first-run download of the game
  files, a signed build on a phone.

## Run in the Simulator

Needs Xcode (with the iOS platform) and Rust with the iOS targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/run-sim.sh                           # iPhone 16 Pro simulator
HORIZON_SIM="iPhone Air" ios/run-sim.sh  # another one
```

The script builds `horizon-ios`, wraps it in `Horizon.app` (`ios/Info.plist`), installs it and
prints the app's game folder. The game looks for its prepared files in
`Documents/.local/horizon-client/<version>/`. Without them it shows the "start from the launcher"
screen.

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
- The window does not yet match the screen's size and safe areas.

## Weekly update

```sh
git fetch upstream
git switch main && git merge --ff-only upstream/main && git push origin main
git switch ios  && git merge main      # a real merge, never squash or rebase
ios/run-sim.sh                         # builds and starts: fix any hook upstream moved
git push origin ios
```
