//! This module owns terminal-signal refusal for active child groups.

use std::io;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::thread;

use signal_hook::consts::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook::iterator::Signals;

use super::ProcessError;

const HANDLED_SIGNALS: [i32; 4] = [SIGINT, SIGTERM, SIGHUP, SIGQUIT];

static CONTROLLER: OnceLock<InterruptController> = OnceLock::new();
static CONTROLLER_START: Mutex<()> = Mutex::new(());

/// Registers one active child operation for typed terminal-signal refusal.
///
/// Dropping an unobserved guard restores the operating system's default
/// handling for the pending signal rather than silently consuming it.
pub(super) struct InterruptGuard {
    registry: Arc<Mutex<Vec<Weak<InterruptState>>>>,
    state: Arc<InterruptState>,
}

struct InterruptController {
    registry: Arc<Mutex<Vec<Weak<InterruptState>>>>,
}

struct InterruptState {
    observed: AtomicBool,
    signal: AtomicI32,
}

enum SignalAdmission {
    First,
    Repeated,
}

enum DispatchOutcome {
    Admitted,
    UseDefault,
}

impl InterruptGuard {
    /// Registers `program` with the shared signal controller.
    ///
    /// Initialization and registry-lock failures remain typed process I/O
    /// failures. Registration does not block on signal delivery.
    pub(super) fn begin(program: &'static str) -> Result<Self, ProcessError> {
        interrupt_controller(program)?.register(program)
    }

    /// Returns the first terminal signal observed for this operation.
    ///
    /// Observation is nonblocking. The returned refusal consumes the guard's
    /// obligation to restore default handling for that signal on drop.
    pub(super) fn refusal(&self, program: &'static str) -> Option<ProcessError> {
        let signal = self.state.observe()?;
        let signal_name = signal_hook::low_level::signal_name(signal).unwrap_or("unknown signal");
        Some(ProcessError::Interrupted {
            program,
            signal: signal_name,
        })
    }
}

fn interrupt_controller(
    program: &'static str,
) -> Result<&'static InterruptController, ProcessError> {
    if let Some(controller) = CONTROLLER.get() {
        return Ok(controller);
    }
    let start_guard = CONTROLLER_START.lock().map_err(|_| ProcessError::Io {
        program,
        action: "serialize terminal signal guard initialization for",
        source: io::Error::other("terminal signal initializer is poisoned"),
    })?;
    if CONTROLLER.get().is_none() {
        let controller = InterruptController::start().map_err(|source| ProcessError::Io {
            program,
            action: "initialize terminal signal guard for",
            source,
        })?;
        CONTROLLER.set(controller).map_err(|_| ProcessError::Io {
            program,
            action: "publish terminal signal guard for",
            source: io::Error::new(
                io::ErrorKind::AlreadyExists,
                "terminal signal controller was already initialized",
            ),
        })?;
    }
    let controller = CONTROLLER.get().ok_or_else(|| ProcessError::Io {
        program,
        action: "read terminal signal guard for",
        source: io::Error::other("terminal signal controller was not initialized"),
    })?;
    drop(start_guard);
    Ok(controller)
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        if let Ok(mut registry) = self.registry.lock() {
            registry.retain(|candidate| {
                candidate
                    .upgrade()
                    .is_some_and(|state| !Arc::ptr_eq(&state, &self.state))
            });
        }
        if let Some(signal) = self.state.unobserved() {
            terminate_by_default(signal);
        }
    }
}

impl InterruptController {
    fn start() -> Result<Self, io::Error> {
        let signals = Signals::new(HANDLED_SIGNALS)?;
        let registry = Arc::new(Mutex::new(Vec::new()));
        let signal_registry = Arc::clone(&registry);
        let worker = thread::Builder::new()
            .name(String::from("xtask-signal-guard"))
            .spawn(move || dispatch(signals, &signal_registry))?;
        drop(worker);
        Ok(Self { registry })
    }

    fn register(&self, program: &'static str) -> Result<InterruptGuard, ProcessError> {
        let state = Arc::new(InterruptState::new());
        self.registry
            .lock()
            .map_err(|_| ProcessError::Io {
                program,
                action: "register active child for",
                source: io::Error::other("terminal signal registry is poisoned"),
            })?
            .push(Arc::downgrade(&state));
        Ok(InterruptGuard {
            registry: Arc::clone(&self.registry),
            state,
        })
    }
}

impl InterruptState {
    const fn new() -> Self {
        Self {
            observed: AtomicBool::new(false),
            signal: AtomicI32::new(0),
        }
    }

    fn interrupt(&self, signal: i32) -> SignalAdmission {
        if self
            .signal
            .compare_exchange(0, signal, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            SignalAdmission::First
        } else {
            SignalAdmission::Repeated
        }
    }

    fn observe(&self) -> Option<i32> {
        let signal = self.signal.load(Ordering::Acquire);
        if signal == 0 {
            return None;
        }
        self.observed.store(true, Ordering::Release);
        Some(signal)
    }

    fn unobserved(&self) -> Option<i32> {
        let signal = self.signal.load(Ordering::Acquire);
        (signal != 0 && !self.observed.load(Ordering::Acquire)).then_some(signal)
    }
}

fn dispatch(mut signals: Signals, registry: &Mutex<Vec<Weak<InterruptState>>>) {
    for signal in signals.forever() {
        if !matches!(
            dispatch_signal(registry, signal),
            Ok(DispatchOutcome::Admitted)
        ) {
            terminate_by_default(signal);
        }
    }
}

fn dispatch_signal(
    registry: &Mutex<Vec<Weak<InterruptState>>>,
    signal: i32,
) -> Result<DispatchOutcome, ()> {
    let mut registry = registry.lock().map_err(|_| ())?;
    let mut has_active_operation = false;
    let mut repeated = false;
    registry.retain(|candidate| {
        candidate.upgrade().is_some_and(|state| {
            has_active_operation = true;
            repeated |= matches!(state.interrupt(signal), SignalAdmission::Repeated);
            true
        })
    });
    drop(registry);
    if !has_active_operation || repeated {
        Ok(DispatchOutcome::UseDefault)
    } else {
        Ok(DispatchOutcome::Admitted)
    }
}

fn terminate_by_default(signal: i32) {
    if signal_hook::low_level::emulate_default_handler(signal).is_err() {
        signal_hook::low_level::abort();
    }
}

#[cfg(test)]
#[path = "interrupt/tests.rs"]
mod tests;
