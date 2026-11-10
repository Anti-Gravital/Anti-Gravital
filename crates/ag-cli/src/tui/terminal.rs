//! Terminal state management.
//!
//! Entering the full-screen interface switches the terminal to raw mode and
//! to the alternate screen. Both must be undone on every exit path, or the
//! user is left with a broken shell (RFC-0018, principle 5):
//!
//! - normal return and early error: the [`Guard`] restores on drop;
//! - panic with unwinding: the same drop runs;
//! - panic with `panic = "abort"` (the release profile): no destructor runs,
//!   so a panic hook restores the terminal before the process aborts.

use std::io;

/// The two operations a terminal session needs. Abstracted so the guard can be
/// tested without a real terminal.
pub(crate) trait TerminalOps {
    /// Switches to raw mode and the alternate screen.
    fn enter(&mut self) -> io::Result<()>;
    /// Restores the terminal. Must be safe to call more than once.
    fn leave(&mut self);
}

/// Restores the terminal when dropped.
pub(crate) struct Guard<T: TerminalOps> {
    ops: T,
}

impl<T: TerminalOps> Guard<T> {
    /// Enters the full-screen state. If entering fails half way, the terminal
    /// is restored before the error is returned.
    pub(crate) fn enter(mut ops: T) -> io::Result<Guard<T>> {
        if let Err(error) = ops.enter() {
            ops.leave();
            return Err(error);
        }
        Ok(Guard { ops })
    }
}

impl<T: TerminalOps> Drop for Guard<T> {
    fn drop(&mut self) {
        self.ops.leave();
    }
}

/// The real terminal, driven through crossterm.
pub(crate) struct CrosstermOps;

impl TerminalOps for CrosstermOps {
    fn enter(&mut self) -> io::Result<()> {
        use ratatui::crossterm::{execute, terminal};
        install_panic_hook();
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)
    }

    fn leave(&mut self) {
        restore();
    }
}

/// Best-effort restoration of the real terminal. Idempotent; errors are
/// ignored because there is nothing useful to do with them on the way out.
fn restore() {
    use ratatui::crossterm::{cursor, execute, terminal};
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show);
}

/// Chains a hook that restores the terminal before the previous panic hook
/// prints its message. Installed once per process.
fn install_panic_hook() {
    use std::sync::Once;
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct Log {
        entered: u32,
        left: u32,
    }

    struct Fake {
        log: Rc<RefCell<Log>>,
        fail_enter: bool,
    }

    impl TerminalOps for Fake {
        fn enter(&mut self) -> io::Result<()> {
            self.log.borrow_mut().entered += 1;
            if self.fail_enter {
                return Err(io::Error::other("cannot enter"));
            }
            Ok(())
        }

        fn leave(&mut self) {
            self.log.borrow_mut().left += 1;
        }
    }

    fn fake(fail_enter: bool) -> (Fake, Rc<RefCell<Log>>) {
        let log = Rc::new(RefCell::new(Log::default()));
        (
            Fake {
                log: Rc::clone(&log),
                fail_enter,
            },
            log,
        )
    }

    #[test]
    fn terminal_is_restored_on_normal_exit() {
        let (ops, log) = fake(false);
        {
            let _guard = Guard::enter(ops).expect("enter");
            assert_eq!(log.borrow().left, 0);
        }
        assert_eq!(log.borrow().entered, 1);
        assert_eq!(log.borrow().left, 1);
    }

    #[test]
    fn terminal_is_restored_when_entering_fails() {
        let (ops, log) = fake(true);
        assert!(Guard::enter(ops).is_err());
        assert_eq!(log.borrow().left, 1);
    }

    #[test]
    fn terminal_is_restored_when_the_event_loop_panics() {
        let (ops, log) = fake(false);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = Guard::enter(ops).expect("enter");
            panic!("simulated failure inside the event loop");
        }));
        assert!(result.is_err());
        assert_eq!(log.borrow().left, 1);
    }
}
