//! Touch controls in play, on iOS: the on-screen joystick, buttons and camera drag of
//! `horizon_client_render::touch` (from Vasysik/horizon-android, MIT) drive the game as a controller
//! does. Their state merges with the pads' in `poll_pads`, so every button reaches the game
//! through the controller mappings. Outside play, touches are the mouse
//! (`horizon_client_input::touch_mouse`).
//!
//! Text fields get the system keyboard: the game allows the input method while one has focus
//! (`sync_ime`), which winit shows the keyboard for, and `touch_mouse` turns its keys into text.
//!
//! The view aims from the screen's centre (crosshair mode): tapping a block to aim at it needs
//! the picking to follow the touch, which is not wired yet.
use super::*;
use horizon_client_input::gamepad::PadState;
use horizon_client_render::touch::{camera_delta, hotbar_slot};
use winit::event::{Touch, TouchPhase};

/// The back button's left, top and side, in GUI pixels.
const BACK: [f32; 3] = [6.0, 6.0, 24.0];

thread_local! {
    /// A press began on the back button: its release is Escape, never the screen's click.
    static BACK_HELD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The touch controls, not a real controller, played last.
    static TOUCH_DROVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

impl Game<'_> {
    /// Touches play the game, rather than moving the mouse: the world is on screen with no
    /// screen or menu over it.
    pub(super) fn touch_in_play(&self) -> bool {
        self.title.is_none()
            && self.screen == Screen::Playing
            && self.player.is_some()
            && !self.pad_in_menu()
    }

    /// One touch in play: a hotbar slot, a control, or a camera drag.
    pub(super) fn touch_play_event(&mut self, touch: Touch) {
        let Some(size) = self
            .window
            .as_ref()
            .map(|w| horizon_client::window::surface_size(w))
        else {
            return;
        };
        self.last_input = Instant::now();
        self.pad.touch.force_crosshair = true;
        self.pad.touch.viewport(size.width, size.height);
        if touch.phase == TouchPhase::Cancelled || !self.touch_in_play() {
            self.pad.touch.clear();
            return;
        }
        TOUCH_DROVE.set(true);
        let started = touch.phase == TouchPhase::Started;
        let ended = touch.phase == TouchPhase::Ended;
        let point = [
            touch.location.x as f32 / size.width.max(1) as f32,
            touch.location.y as f32 / size.height.max(1) as f32,
        ];
        if started {
            let batch = HudBatch::new([size.width, size.height]);
            let at = [
                touch.location.x as f32 / batch.scale,
                touch.location.y as f32 / batch.scale,
            ];
            if let Some(slot) = hotbar_slot(batch.width, batch.height, at)
                && !self.pad.touch.control_at_point(point)
            {
                self.selected = slot;
                self.pad.touch.ignore(touch.id);
                return;
            }
        }
        self.pad.touch.projectile_crosshair = self.touch_projectile_crosshair();
        self.pad.touch.hold_uses_item = self.touch_item_held_use();
        let delta = self.pad.touch.update(touch.id, point, started, ended);
        let delta = camera_delta(delta, size.width, size.height, &self.pad.touch.settings);
        if delta != [0.0; 2] {
            // Degrees, where Entity.turn takes mouse units of 0.15 degrees.
            self.look_turn(f64::from(delta[0] / 0.15), f64::from(delta[1] / 0.15));
        }
        self.touch_play_requests();
    }

    /// The buttons that open something rather than hold a controller button.
    fn touch_play_requests(&mut self) {
        let touch = &mut self.pad.touch;
        if std::mem::take(&mut touch.chat_requested) {
            self.audio.play_button_click();
            self.pad.touch.clear();
            if let Err(error) = self.key_pressed(KeyCode::KeyT, KeyCode::KeyT) {
                log::warn!("Touch chat: {error}");
            }
        } else if std::mem::take(&mut touch.settings_requested) {
            self.audio.play_button_click();
            self.pad.touch.clear();
            self.open_options(false);
        } else if std::mem::take(&mut touch.debug_requested) {
            self.audio.play_button_click();
            use horizon_client::debug_keys::KeyAction;
            self.debug_key(KeyCode::F3, KeyAction::Press);
            self.debug_key(KeyCode::F3, KeyAction::Release);
        } else if std::mem::take(&mut touch.perspective_requested) {
            self.audio.play_button_click();
            if let Err(error) = self.key_pressed(KeyCode::F5, KeyCode::F5) {
                log::warn!("Touch perspective: {error}");
            }
        }
    }

    /// The pads' state with the touch controls' merged in, once per poll (`poll_pads`).
    pub(super) fn touch_play_state(&mut self, state: PadState) -> PadState {
        let in_play = self.touch_in_play();
        horizon_client_input::touch_mouse::set_play(in_play);
        if state != PadState::default() {
            TOUCH_DROVE.set(false);
        }
        if !in_play {
            self.pad.touch.clear();
            // The touch controls drove the pad's state: a screen opened from play (chat, a
            // sign) is played by touch, as with the mouse, so the controller's own keyboard
            // stays shut and the system keyboard types into it.
            if TOUCH_DROVE.get() {
                self.pad.in_use = false;
            }
            return state;
        }
        self.pad.touch.force_crosshair = true;
        self.pad.touch.shield_available = self.touch_shield_available();
        let mut touch = self.pad.touch.poll_state();
        if self.pad.touch.holding && (self.pad.touch.holding_use || self.touch_item_held_use()) {
            // A long press on the world with an item that has a use: use it, don't mine.
            touch.triggers = [1.0, 0.0];
            self.pad.touch.latch_use();
        }
        if let Some(held_use) = self.pad.touch.take_tap() {
            if self.entity_targeted() && self.using.is_none() {
                // A short tap on a mob attacks it: the attack trigger for this one poll.
                touch.triggers[1] = 1.0;
            } else if !held_use {
                // A short tap on a block places or uses, as a right click does.
                touch.triggers[0] = 1.0;
            }
        }
        state.merge(&touch)
    }

    /// The controls over the HUD, in play; the back button over a screen.
    pub(super) fn touch_play_draw(&mut self, hud: &mut HudBatch) {
        if self.touch_back_shown() {
            let [x, y, size] = BACK;
            let atlas = &self.gui.atlas;
            hud.fill(atlas, x, y, size, size, rgba(0, 0, 0, 150));
            let width = HudBatch::text_width(atlas, "<");
            hud.text(
                atlas,
                "<",
                x + (size - width) / 2.0,
                y + (size - 8.0) / 2.0,
                rgba(255, 255, 255, 255),
                true,
            );
        }
        if !self.touch_in_play() || self.ui.hide_gui {
            return;
        }
        let player = self.player.as_ref();
        self.pad.touch.vertical_movement = player.is_some_and(|p| p.flying);
        self.pad.touch.swimming = player.is_some_and(|p| p.swimming);
        self.pad.touch.mounted = self.riding.is_some();
        self.pad.touch.sneaking = self.keys.held(Mapping::Sneak);
        self.pad.touch.break_progress = self.mining.as_ref().map_or(0.0, |m| m.progress);
        self.pad.touch.draw(hud, &self.gui.atlas);
    }

    /// A phone has no Escape key or back button, and chat, the inventory and containers have no
    /// button of their own to close them: over any screen in a world, a back button in the
    /// top-left corner does what Escape does.
    fn touch_back_shown(&self) -> bool {
        self.title.is_none() && self.player.is_some() && self.screen != Screen::Playing
    }

    /// A click (a tap) on the back button: Escape, on its release. True when it took the event.
    pub(super) fn touch_back_click(&mut self, event: &WindowEvent) -> bool {
        let WindowEvent::MouseInput {
            state,
            button: MouseButton::Left,
            ..
        } = event
        else {
            return false;
        };
        let [x, y, size] = BACK;
        let (cx, cy) = self.cursor;
        let inside = (x..x + size).contains(&cx) && (y..y + size).contains(&cy);
        match state {
            ElementState::Pressed if inside && self.touch_back_shown() => {
                BACK_HELD.set(true);
                true
            }
            ElementState::Released if BACK_HELD.replace(false) => {
                if inside {
                    self.audio.play_button_click();
                    if let Err(error) = self.key_pressed(KeyCode::Escape, KeyCode::Escape) {
                        log::warn!("Touch back: {error}");
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn touch_shield_available(&self) -> bool {
        self.held()
            .into_iter()
            .chain(self.offhand.as_ref())
            .any(|s| self.blocks_attacks(s).is_some())
    }

    fn touch_projectile_crosshair(&self) -> bool {
        self.held()
            .is_some_and(|s| horizon_client_render::touch::projectile_crosshair(&s.item))
    }

    /// A hold on the world uses the held item instead of mining (Vasysik's rules).
    fn touch_item_held_use(&self) -> bool {
        self.held().is_some_and(|stack| {
            if horizon_client_render::touch::item_uses_on_tap(&stack.item) {
                return false;
            }
            let Some(info) = self.items().get(&stack.item) else {
                return false;
            };
            remote::item_has_use(info)
                || info.bundle
                || matches!(
                    &*stack.item,
                    "minecraft:firework_rocket" | "minecraft:lead" | "minecraft:name_tag"
                )
                || info.block.is_some_and(|state| {
                    let block = self.res.facts().get(state);
                    ["CropBlock", "StemBlock", "NetherWartBlock", "CocoaBlock"]
                        .iter()
                        .any(|class| block.has_class(class))
                })
                || self.blocks_attacks(stack).is_some()
        })
    }
}
