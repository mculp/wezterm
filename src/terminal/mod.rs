//! An abstraction over a terminal device

use caps::Capabilities;
use failure::Error;
use input::InputEvent;
use num::{self, NumCast};
use std::fmt::Display;
use surface::Change;

#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

pub mod buffered;

#[cfg(unix)]
pub use self::unix::UnixTerminal;
#[cfg(windows)]
pub use self::windows::WindowsTerminal;

/// Represents the size of the terminal screen.
/// The number of rows and columns of character cells are expressed.
/// Some implementations populate the size of those cells in pixels.
// On Windows, GetConsoleFontSize() can return the size of a cell in
// logical units and we can probably use this to populate xpixel, ypixel.
// GetConsoleScreenBufferInfo() can return the rows and cols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenSize {
    /// The number of rows of text
    pub rows: usize,
    /// The number of columns per row
    pub cols: usize,
    /// The width of a cell in pixels.  Some implementations never
    /// set this to anything other than zero.
    pub xpixel: usize,
    /// The height of a cell in pixels.  Some implementations never
    /// set this to anything other than zero.
    pub ypixel: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blocking {
    DoNotWait,
    Wait,
}

/// `Terminal` abstracts over some basic terminal capabilities.
/// If the `set_raw_mode` or `set_cooked_mode` functions are used in
/// any combination, the implementation is required to restore the
/// terminal mode that was in effect when it was created.
pub trait Terminal {
    /// Raw mode disables input line buffering, allowing data to be
    /// read as the user presses keys, disables local echo, so keys
    /// pressed by the user do not implicitly render to the terminal
    /// output, and disables canonicalization of unix newlines to CRLF.
    fn set_raw_mode(&mut self) -> Result<(), Error>;

    /// Queries the current screen size, returning width, height.
    fn get_screen_size(&mut self) -> Result<ScreenSize, Error>;

    /// Sets the current screen size
    fn set_screen_size(&mut self, size: ScreenSize) -> Result<(), Error>;

    /// Render a series of changes to the terminal output
    fn render(&mut self, changes: &[Change]) -> Result<(), Error>;

    /// Flush any buffered output
    fn flush(&mut self) -> Result<(), Error>;

    /// Check for a parsed input event.
    /// `blocking` indicates the behavior in the case that no input is
    /// immediately available.  If blocking == `Blocking::Wait` then
    /// `poll_input` will not return until an event is available.
    /// If blocking == `Blocking:DoNotWait` then `poll_input` will return
    /// immediately with a value of `Ok(None)`.
    ///
    /// The possible values returned as `InputEvent`s depend on the
    /// mode of the terminal.  Most modes are not returned unless
    /// the terminal is set to raw mode.
    fn poll_input(&mut self, blocking: Blocking) -> Result<Option<InputEvent>, Error>;
}

/// `SystemTerminal` is a concrete implementation of `Terminal`.
/// Ideally you wouldn't reference `SystemTerminal` in consuming
/// code.  This type is exposed for convenience if you are doing
/// something unusual and want easier access to the constructors.
#[cfg(unix)]
pub type SystemTerminal = UnixTerminal;
#[cfg(windows)]
pub type SystemTerminal = WindowsTerminal;

/// Construct a new instance of Terminal.
/// The terminal will have a renderer that is influenced by the configuration
/// in the provided `Capabilities` instance.
/// The terminal will explicitly open `/dev/tty` on Unix systems and
/// `CONIN$` and `CONOUT$` on Windows systems, so that it should yield a
/// functioning console with minimal headaches.
/// If you have a more advanced use case you will want to look to the
/// constructors for `UnixTerminal` and `WindowsTerminal` and call whichever
/// one is most suitable for your needs.
pub fn new_terminal(caps: Capabilities) -> Result<impl Terminal, Error> {
    SystemTerminal::new(caps)
}

pub(crate) fn cast<T: NumCast + Display + Copy, U: NumCast>(n: T) -> Result<U, Error> {
    num::cast(n).ok_or_else(|| format_err!("{} is out of bounds for this system", n))
}
