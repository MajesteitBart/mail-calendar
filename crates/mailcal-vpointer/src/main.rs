//! Pointer input for the Linux client's headless test compositor.
//!
//! ```text
//! mailcal-vpointer move <x> <y>
//! mailcal-vpointer click <x> <y> [left|right|middle]
//! mailcal-vpointer drag <x1> <y1> <x2> <y2> [steps]
//! mailcal-vpointer scroll <x> <y> <dy> [dx]
//! ```
//!
//! Coordinates are output pixels, resolved against the compositor's own output, so a caller states
//! what it reads off a screenshot and nothing has to agree about a size.
//!
//! **Why this exists, when `wlrctl` speaks the same protocol.** A seat with no physical pointer,
//! which is every headless compositor, gains its pointer capability only while a virtual pointer
//! is alive. `wlrctl` creates one, sends a single action and exits, so:
//!
//! - the device is gone before a client has bound `wl_pointer` and can be sent anything, and
//! - the cursor position dies with it, so a move in one invocation and a click in the next are two
//!   unrelated pointers and the click lands wherever the compositor's cursor happened to be.
//!
//! Measured on this repository's harness: three `wlrctl pointer` invocations to move and click a
//! control changed **zero** pixels, and one connection doing the same move and click changed
//! 702,956. That is the whole difference. The upstream report
//! (<https://github.com/cage-kiosk/cage/issues/305>) reads it as a wlroots bug, and its
//! reproduction is the same invocation-per-action shape, so treat "virtual pointer does not work
//! on headless" as a claim about that shape rather than about the protocol.
//!
//! So: one connection holds one pointer for the whole gesture, waits for the compositor to attach
//! it before sending anything, and lingers afterwards so the last events are dispatched.

#[cfg(target_os = "linux")]
mod wayland;

#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    wayland::main()
}

// Built on every host, so a `--workspace` build stays green where the Wayland crates cannot be
// compiled, Windows among them.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!("error: mailcal-vpointer drives a Wayland compositor, so it runs on Linux only");
    std::process::ExitCode::FAILURE
}
