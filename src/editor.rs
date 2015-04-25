//! All VST plugin editor related functionality.

use libc::c_void;

use enums::{KnobMode, Key};

/// Implemented by plugin editors.
pub trait Editor {
    /// Get the size of the editor window.
    fn size(&self) -> (i32, i32);

    /// Get the coordinates of the editor window.
    fn position(&self) -> (i32, i32);


    /// Editor idle call. Called by host.
    fn idle(&mut self);

    /// Called when the editor window is closed.
    fn close(&mut self);

    /// Called when the editor window is opened. `window` is a platform dependant window pointer
    /// (e.g. `HWND` on Windows, `WindowRef` on OSX, `Window` on X11/Linux).
    fn open(&mut self, window: *mut c_void);

    /// Return whether the window is currently open.
    fn is_open(&mut self) -> bool;


    /// Set the knob mode for this editor (if supported by host).
    fn set_knob_mode(&mut self, mode: KnobMode);

    /// Recieve key up event. 
    fn key_up(&mut self, keycode: KeyCode) -> bool;

    /// Receive key down event. Return true if the key was used.
    fn key_down(&mut self, keycode: KeyCode) -> bool;
}

/// Rectangle used to specify dimensions of editor window.
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
