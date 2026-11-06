# Horizon for iPhone

An unofficial iOS port of [Horizon](https://github.com/horizon-rust/horizon), the Rust rewrite of
Minecraft: Java Edition 26.3. The `ios` branch is upstream Horizon plus a thin iOS layer, and it
merges upstream every week, so it keeps up with the game.

It is experimental. It runs on an iPhone 16 Pro (iOS 26.5) at a steady 60 fps on the 120 Hz
screen: the GPU takes about 11 ms a frame, or 15 ms with far terrain (LOD) at 128 chunks.

Not affiliated with Mojang or Microsoft. You need your own copy of Minecraft: Java Edition. No
Mojang files are in this repository: you make the game's files on your Mac from your own copy.

## What works

- Metal rendering, full screen in landscape, 120 Hz where the GPU keeps up.
- Menus by touch: tap and drag as with a mouse.
- In a world: an on-screen joystick, buttons and camera drag (from
  [Vasysik/horizon-android](https://github.com/Vasysik/horizon-android)), the hotbar by tapping,
  and a back button in the top-left corner over any screen (chat, the inventory, containers).
- The system keyboard in chat, on signs, in books and in search fields.
- Single player, multiplayer and sound.

What's missing and what comes next: [ROADMAP.md](ROADMAP.md).

## What you need

- A Mac with Apple Silicon, [Xcode](https://apps.apple.com/app/xcode/id497799835),
  [Rust](https://rustup.rs) and [XcodeGen](https://github.com/yonaskolb/XcodeGen)
  (`brew install xcodegen`).
- An iPhone with a USB cable. A free Apple ID is enough to sign the app.
- Minecraft: Java Edition 26.3, installed with the official launcher or Prism Launcher, and
  played once so its files are downloaded.

## Install

1. **Get the code.**

   ```sh
   git clone -b ios https://github.com/dblgq/horizon.git
   cd horizon
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim
   ```

2. **Make the game's files** from your Minecraft (once per Minecraft version, about a minute).
   Follow [The game files](../docs/ios.md#the-game-files) in `docs/ios.md`.

3. **Sign in to Xcode** with your Apple ID: Xcode > Settings > Accounts.

4. **Connect the iPhone**, unlock it and tap "Trust". Then turn on Developer Mode: Settings >
   Privacy & Security > Developer Mode. The phone restarts.

5. **The first install goes through Xcode, once:**

   ```sh
   (cd ios && xcodegen generate)
   open ios/Horizon.xcodeproj
   ```

   Select the **Horizon** target, open **Signing & Capabilities** and pick your **Personal
   Team**. Choose your iPhone at the top and press **Run**. If the phone says "Untrusted
   Developer", trust your Apple ID in Settings > General > VPN & Device Management and run again.

6. **Copy the game's files and start it:**

   ```sh
   ios/run-device.sh
   ```

   The first run copies about 700 MB over the cable. Later runs copy only what changed.

From then on, `ios/run-device.sh` builds, installs and starts the game in one step.

**A free Apple ID's signature lasts 7 days.** After that the app won't open: connect the phone
and run `ios/run-device.sh` again. Your worlds stay.

## Without a phone

`ios/run-sim.sh` runs the game in the iOS Simulator. Clicks are touches there. See
[docs/ios.md](../docs/ios.md#run-in-the-simulator).

## Updates

This branch merges upstream Horizon weekly. To update:

```sh
git pull
ios/run-device.sh
```

How the port is built, and how the weekly merge works: [docs/ios.md](../docs/ios.md).

## Credits and licenses

- [Horizon](https://github.com/horizon-rust/horizon) by Ludovic Marie and the Horizon
  contributors, MIT.
- Touch controls from [Vasysik/horizon-android](https://github.com/Vasysik/horizon-android), MIT.
- The touch controls' button art is Mojang's, from
  [Mojang/bedrock-samples](https://github.com/Mojang/bedrock-samples) under Mojang's terms.
  `tools/ios_controls.py` downloads it on your Mac. It is not in this repository.
- The iOS layer is MIT, like Horizon.
