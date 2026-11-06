# Roadmap: Horizon for iPhone

Where the iOS port stands and what comes next. Horizon's own roadmap is upstream's
[ROADMAP.md](../ROADMAP.md).

## Done

- **Builds for iOS.** Device and Simulator, from fresh upstream, with small hooks in upstream's
  code (clipboard, memory counter, HDR, key names, Metal on iOS).
- **Runs in the Simulator.** Its Apple2-family GPU has no indirect draws, so terrain falls back to
  per-section draws there.
- **Full screen.** The surface matches the screen, with no stretching.
- **Game files.** Made on the Mac with `horizon-prepare` and copied into the app.
- **Touch.** Menus work as a mouse. In a world there are Vasysik's joystick and buttons, the
  hotbar by tapping, and a back button over screens.
- **System keyboard.** It types into text fields, and Return sends.
- **On an iPhone.** Signed with a free Apple ID, installed with `ios/run-device.sh`. Main thread
  stack is 8 MB (the renderer overflowed iOS's 1 MB).

## Next

1. **120 fps.** A render scale: the GPU needs under 8.3 ms a frame and takes about 11 ms at full
   resolution (2622×1206).
2. **Safe areas.** The HUD and the controls keep clear of the Dynamic Island and the home
   indicator.
3. **Aim by touch.** Break or place the block you tap, not the one at the centre.
4. **Touch settings.** Vasysik's layout editor (move, resize, fade buttons) and the touch options
   in the settings.
5. **Background and audio.** Stop rendering when the app goes to the background, so iOS doesn't
   end it. Set the audio session so the silent switch and other apps' sound behave.
6. **Weekly check in CI.** A GitHub Actions job merges upstream every week, builds for iOS and
   reports when a hook needs fixing.

## Later

- **iOS 27.** winit 0.30 has no UIScene life cycle, which apps built with the iOS 27 SDK need.
  For now the binary is stamped as built with SDK 26. Move to winit 0.31, or add scenes.
- **System clipboard** (UIPasteboard). Copy and paste work only inside the game for now.
- **Game files without a cable.** Import them through the Files app, or download them on first
  start.
- **App icon** and a launch screen.
- **Upstream the generic fixes.** Some hooks fix upstream bugs: flat draws without indirect
  execution, Vulkan-only code built on every Apple OS. Small PRs to horizon-rust/horizon shrink
  the fork.
- **Memory.** Watch the limit on 8 GB phones with far terrain on.
