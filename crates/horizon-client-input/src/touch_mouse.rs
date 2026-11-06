//! A touchscreen as a mouse, where touches are the only pointer (iOS): the first finger down
//! moves the cursor and holds the left button until it lifts, so every screen made for a mouse
//! (buttons, sliders, lists, the inventory) works by tapping and dragging. Other fingers are
//! ignored while it is down. Elsewhere touches pass through unchanged.

use winit::event::{ElementState, MouseButton, Touch, TouchPhase, WindowEvent};

/// The events the game sees for one window event: a touch becomes the cursor's move and,
/// when it starts or ends, the left button's press or release.
pub fn events(event: WindowEvent) -> impl Iterator<Item = WindowEvent> {
    let [first, second] = if cfg!(target_os = "ios") {
        translate(event)
    } else {
        [Some(event), None]
    };
    first.into_iter().chain(second)
}

thread_local! {
    /// The finger acting as the mouse, while it is down.
    static PRIMARY: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// The world is in play: new touches reach the game as touches, for its touch controls.
static PLAY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether new touches go to the game's touch controls (play) instead of the mouse (menus).
/// A finger already acting as the mouse stays the mouse until it lifts.
pub fn set_play(play: bool) {
    PLAY.store(play, std::sync::atomic::Ordering::Relaxed);
}

fn translate(event: WindowEvent) -> [Option<WindowEvent>; 2] {
    let WindowEvent::Touch(touch) = event else {
        return [Some(event), None];
    };
    let mouse = PRIMARY.with(|p| p.get());
    if mouse != Some(touch.id) && PLAY.load(std::sync::atomic::Ordering::Relaxed) {
        return [Some(event), None];
    }
    let Some(state) = PRIMARY.with(|p| step(p, &touch)) else {
        return [None, None];
    };
    let moved = WindowEvent::CursorMoved {
        device_id: touch.device_id,
        position: touch.location,
    };
    let button = state.map(|state| WindowEvent::MouseInput {
        device_id: touch.device_id,
        state,
        button: MouseButton::Left,
    });
    [Some(moved), button]
}

/// What `touch` does to the mouse: `None` when it is not the primary finger, else the button
/// change it makes (`None` for a move).
fn step(primary: &std::cell::Cell<Option<u64>>, touch: &Touch) -> Option<Option<ElementState>> {
    match (touch.phase, primary.get()) {
        (TouchPhase::Started, None) => {
            primary.set(Some(touch.id));
            Some(Some(ElementState::Pressed))
        }
        (TouchPhase::Moved, Some(id)) if id == touch.id => Some(None),
        (TouchPhase::Ended | TouchPhase::Cancelled, Some(id)) if id == touch.id => {
            primary.set(None);
            Some(Some(ElementState::Released))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::DeviceId;

    fn touch(id: u64, phase: TouchPhase, x: f64) -> Touch {
        Touch {
            device_id: DeviceId::dummy(),
            phase,
            location: PhysicalPosition::new(x, 10.0),
            force: None,
            id,
        }
    }

    fn mouse(id: u64, phase: TouchPhase, x: f64) -> Vec<WindowEvent> {
        translate(WindowEvent::Touch(touch(id, phase, x)))
            .into_iter()
            .flatten()
            .collect()
    }

    fn press(state: ElementState) -> WindowEvent {
        WindowEvent::MouseInput {
            device_id: DeviceId::dummy(),
            state,
            button: MouseButton::Left,
        }
    }

    fn at(x: f64) -> WindowEvent {
        WindowEvent::CursorMoved {
            device_id: DeviceId::dummy(),
            position: PhysicalPosition::new(x, 10.0),
        }
    }

    #[test]
    fn a_tap_moves_the_cursor_then_clicks_and_a_drag_holds_the_button() {
        assert_eq!(
            mouse(1, TouchPhase::Started, 5.0),
            [at(5.0), press(ElementState::Pressed)]
        );
        assert_eq!(mouse(1, TouchPhase::Moved, 8.0), [at(8.0)]);
        assert_eq!(
            mouse(1, TouchPhase::Ended, 9.0),
            [at(9.0), press(ElementState::Released)]
        );
    }

    #[test]
    fn a_second_finger_is_ignored_until_the_first_lifts() {
        mouse(1, TouchPhase::Started, 5.0);
        assert!(mouse(2, TouchPhase::Started, 50.0).is_empty());
        assert!(mouse(2, TouchPhase::Moved, 60.0).is_empty());
        assert_eq!(
            mouse(1, TouchPhase::Cancelled, 5.0),
            [at(5.0), press(ElementState::Released)]
        );
        assert!(mouse(2, TouchPhase::Ended, 60.0).is_empty());
        assert_eq!(
            mouse(3, TouchPhase::Started, 7.0),
            [at(7.0), press(ElementState::Pressed)]
        );
    }

    #[test]
    fn other_events_pass_through() {
        let focused = WindowEvent::Focused(true);
        assert_eq!(translate(focused.clone()), [Some(focused), None]);
    }
}
