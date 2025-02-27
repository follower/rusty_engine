use crate::prelude::EngineState;
use bevy::{prelude::*, utils::HashSet};

// Re-export some Bevy types to use
pub use bevy::{
    input::{
        mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
        ElementState,
    },
    window::CursorMoved,
};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.insert_resource(MouseState::default())
            .add_system(sync_mouse_state.system().before("game_logic_sync"))
            .add_system(sync_mouse_events.system().before("game_logic_sync"));
    }
}

/// `MouseState` represents the end-state of the mouse during the last frame. This should be used
/// for "real time" processing of most input (except mousewheel scrolling), where you only care
/// about the final state of buttons or mouse position for your logic.
///
/// If you need to process all mouse events that occurred during a single frame, use the
/// `mouse_button_events`, `mouse_location_events`, `mouse_motion_events`, or `mouse_wheel_events`
/// fields on [`EngineState`](crate::prelude::EngineState).
#[derive(Clone, Debug, Default)]
pub struct MouseState {
    location: Option<Vec2>,
    motion: Vec2,
    wheel: MouseWheelState,
    pressed: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
}

/// A simplification of mouse wheel events for a frame into a single state. Unless you are treating
/// the mouse wheel as if scrolling in a direction were equivalent to clicking a mouse button, you
/// probably want to use
/// [`EngineState::mouse_wheel_events`](crate::prelude::EngineState::mouse_wheel_events) instead.
#[derive(Clone, Copy, Debug, Default)]
pub struct MouseWheelState {
    /// The y component of the mouse wheel movement. This is the "normal" scrolling direction of a
    /// typical mouse wheel. This will be either `-1.0`, `0.0`, or `1.0`. For fine-grained
    /// mouse wheel info, see
    /// [`EngineState::mouse_wheel_events`](crate::prelude::EngineState::mouse_wheel_events) instead.
    pub y: f32,
    /// The x component of the mouse wheel movement. This is usually caused by holding the shift key
    /// while scrolling the mouse on regular mice, or by a horizontal scroll wheel on exotic mice.
    /// This will be either `-1.0`, `0.0`, or `1.0`. For fine-grained mouse wheel info, see
    /// [`EngineState::mouse_wheel_events`](crate::prelude::EngineState::mouse_wheel_events) instead.
    pub x: f32,
}

impl MouseState {
    /// Final location of the mouse this frame. If you want to process _all_ the locations the mouse
    /// was at during this frame, see
    /// [`EngineState::mouse_location_events`](crate::prelude::EngineState::mouse_location_events)
    /// instead.
    pub fn location(&self) -> Option<Vec2> {
        self.location
    }
    /// The cumulative relative motion of the mouse this frame. If you want to process _all_ the
    /// individual relative motions, see
    /// [`EngineState::mouse_motion_events](crate::prelude::EngineState::mouse_motion_events) instead.
    pub fn motion(&self) -> Vec2 {
        self.motion
    }
    /// Returns a [`MouseWheelState], which is a simplified version of cumulative mouse wheel
    /// events. When dealing with mouse wheel movement, you _usually_ don't want this...you want
    /// [`EngineState::mouse_wheel_events`](crate::prelude::EngineState::mouse_wheel_events) instead.
    pub fn wheel(&self) -> MouseWheelState {
        self.wheel
    }
    /// Returns true if the mouse button was pressed
    pub fn pressed(&self, mouse_button: MouseButton) -> bool {
        self.pressed.contains(&mouse_button)
    }
    /// Returns true if the mouse button started being pressed during the last frame
    pub fn just_pressed(&self, mouse_button: MouseButton) -> bool {
        self.just_pressed.contains(&mouse_button)
    }
    /// Returns true if the mouse button started being released during the last frame
    pub fn just_released(&self, mouse_button: MouseButton) -> bool {
        self.just_released.contains(&mouse_button)
    }
    /// Returns true if any of the indicated mouse buttons were pressed
    pub fn pressed_any(&self, mouse_buttons: &[MouseButton]) -> bool {
        mouse_buttons.iter().any(|k| self.pressed(*k))
    }
    /// Returns true if any of the indicated mouse buttons were just pressed this frame
    pub fn just_pressed_any(&self, mouse_buttons: &[MouseButton]) -> bool {
        mouse_buttons.iter().any(|k| self.just_pressed(*k))
    }
    /// Returns true if any of the indicated mouse buttons were just released this frame
    pub fn just_released_any(&self, mouse_buttons: &[MouseButton]) -> bool {
        mouse_buttons.iter().any(|k| self.just_released(*k))
    }
}

fn sync_mouse_events(
    mut game_state: ResMut<EngineState>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    // Clear any events that weren't used last frame
    game_state.mouse_button_events.clear();
    game_state.mouse_location_events.clear();
    game_state.mouse_motion_events.clear();
    game_state.mouse_wheel_events.clear();

    // Populate this frame's events
    for ev in mouse_button_events.iter() {
        game_state.mouse_button_events.push(ev.clone());
    }
    for ev in cursor_moved_events.iter() {
        let mut new_event = ev.clone();
        // Convert from screen space to game space
        // TODO: Check to see if this needs to be adjusted for different DPIs
        new_event.position -= game_state.window_dimensions * 0.5;
        game_state.mouse_location_events.push(new_event);
    }
    for ev in mouse_motion_events.iter() {
        let mut ev2 = ev.clone();
        ev2.delta.y *= -1.0;
        game_state.mouse_motion_events.push(ev2.clone());
    }
    for ev in mouse_wheel_events.iter() {
        game_state.mouse_wheel_events.push(ev.clone());
    }
}

fn sync_mouse_state(
    mouse_button_input: Res<Input<MouseButton>>,
    mut mouse_state: ResMut<MouseState>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    game_state: Res<EngineState>,
) {
    // Sync the current mouse location, which will be the last cursor_moved event that occurred.
    // Only changes when we get a new event, otherwise we preserve the last location.
    if let Some(event) = cursor_moved_events.iter().last() {
        // Convert from bevy's window space to our game space
        let location = event.position - game_state.window_dimensions * 0.5;
        mouse_state.location = Some(location);
    }
    // Sync the relative mouse motion. This is the cumulative relative motion during the last frame.
    mouse_state.motion = Vec2::ZERO;
    for ev in mouse_motion_events.iter() {
        // Convert motion to game space direction (positive y is up, not down)
        // TODO: Check to see if this needs to be adjusted for different DPIs
        let mut ev2 = ev.clone();
        ev2.delta.y *= -1.0;
        mouse_state.motion += ev2.delta;
    }
    // Sync the mouse wheel state (which is heavily simplified from mouse wheel events)
    mouse_state.wheel = MouseWheelState::default();
    let mut cumulative_x = 0.0;
    let mut cumulative_y = 0.0;
    for ev in mouse_wheel_events.iter() {
        cumulative_x += ev.x;
        cumulative_y += ev.y;
    }
    mouse_state.wheel.x = match cumulative_x {
        x if x > 0.0 => 1.0,
        x if x < 0.0 => -1.0,
        _ => 0.0,
    };
    mouse_state.wheel.y = match cumulative_y {
        y if y > 0.0 => 1.0,
        y if y < 0.0 => -1.0,
        _ => 0.0,
    };
    // Lucky for us, Bevy processes mouse button events into mouse state already, so we just need to
    // transfer various state over.
    //
    // First, the buttons which are currently pressed
    mouse_state.pressed.clear();
    for mouse_button in mouse_button_input.get_pressed() {
        mouse_state.pressed.insert(*mouse_button);
    }
    // Next, the buttons which were just pressed
    mouse_state.just_pressed.clear();
    for mouse_button in mouse_button_input.get_just_pressed() {
        mouse_state.just_pressed.insert(*mouse_button);
    }
    // Finally, the buttons which were just released
    mouse_state.just_released.clear();
    for mouse_button in mouse_button_input.get_just_released() {
        mouse_state.just_released.insert(*mouse_button);
    }
}
