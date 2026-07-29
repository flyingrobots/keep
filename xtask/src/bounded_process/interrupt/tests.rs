//! This module owns terminal-signal guard state-transition laws.

use signal_hook::consts::{SIGINT, SIGTERM};

use super::{InterruptState, SignalAdmission};

#[test]
fn first_terminal_signal_becomes_one_observed_refusal() {
    let state = InterruptState::new();

    assert!(matches!(state.interrupt(SIGINT), SignalAdmission::First));
    assert_eq!(state.observe(), Some(SIGINT));
    assert_eq!(state.unobserved(), None);
}

#[test]
fn retirement_detects_an_unobserved_terminal_signal() {
    let state = InterruptState::new();

    assert!(matches!(state.interrupt(SIGTERM), SignalAdmission::First));
    assert_eq!(state.unobserved(), Some(SIGTERM));
}

#[test]
fn a_second_terminal_signal_requires_default_termination() {
    let state = InterruptState::new();

    assert!(matches!(state.interrupt(SIGINT), SignalAdmission::First));
    assert!(matches!(
        state.interrupt(SIGTERM),
        SignalAdmission::Repeated
    ));
    assert_eq!(state.observe(), Some(SIGINT));
}
