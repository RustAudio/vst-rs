//! All VST plugin editor related functionality.

use libc::c_void;

/// Implemented by plugin editors.
#[allow(unused_variables)]
pub trait Editor {
    /// Get the size of the editor window.
    fn size(&self) -> (i32, i32);

    /// Get the coordinates of the editor window.
    fn position(&self) -> (i32, i32);


    /// Editor idle call. Called by host.
    fn idle(&mut self) {}

    /// Called when the editor window is closed.
    fn close(&mut self) {}

    /// Called when the editor window is opened. `window` is a platform dependant window pointer
    /// (e.g. `HWND` on Windows, `WindowRef` on OSX, `Window` on X11/Linux).
    fn open(&mut self, window: *mut c_void);

    /// Return whether the window is currently open.
    fn is_open(&mut self) -> bool;


    /// Set the knob mode for this editor (if supported by host).
    ///
    /// Return true if the knob mode was set.
    fn set_knob_mode(&mut self, mode: KnobMode) -> bool { false }

    /// Recieve key up event. Return true if the key was used.
    fn key_up(&mut self, keycode: KeyCode) -> bool { false }

    /// Receive key down event. Return true if the key was used.
    fn key_down(&mut self, keycode: KeyCode) -> bool { false }
}

/// Rectangle used to specify dimensions of editor window.
#[doc(hidden)]
pub struct Rect {
    /// Y value in pixels of top side.
    pub top: i16,
    /// X value in pixels of left side.
    pub left: i16,
    /// Y value in pixels of bottom side.
    pub bottom: i16,
    /// X value in pixels of right side.
    pub right: i16
}

/// A platform independent key code. Includes modifier keys.
pub struct KeyCode {
    /// ASCII character for key pressed (if applicable).
    pub character: char,
    /// Key pressed. See `enums::Key`.
    pub key: Key,
    /// Modifier key bitflags. See `enums::flags::modifier_key`.
    pub modifier: u8
}

/// Allows host to set how a parameter knob works.
#[repr(usize)]
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum KnobMode {
    Circular,
    CircularRelative,
    Linear
}
impl_clike!(KnobMode);

/// Platform independent key codes.
#[allow(missing_docs)]
#[repr(usize)]
#[derive(Debug, Copy, Clone)]
pub enum Key {
	Back = 1,
	Tab,
	Clear,
	Return,
	Pause,
	Escape,
	Space,
	Next,
	End,
	Home,
	Left,
	Up,
	Right,
	Down,
	PageUp,
	PageDown,
	Select,
	Print,
	Enter,
	Snapshot,
	Insert,
	Delete,
	Help,
	Numpad0,
	Numpad1,
	Numpad2,
	Numpad3,
	Numpad4,
	Numpad5,
	Numpad6,
	Numpad7,
	Numpad8,
	Numpad9,
	Multiply,
	Add,
	Separator,
	Subtract,
	Decimal,
	Divide,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	Numlock,
	Scroll,
	Shift,
	Control,
	Alt,
	Equals
}
impl_clike!(Key);
