//! This module owns `env` option and utility-selection laws.

#[test]
fn options_assignments_and_split_strings_precede_the_utility() {
    for arguments in [
        b"-i python3 -I".as_slice(),
        b"-iv python3",
        b"-u PYTHONHOME python3",
        b"NAME=value python3",
        b"-S -i python3 -I",
        b"-S \"python3 -I\"",
        b"-S-iuFOO python3",
        b"-iSpython3 -I",
        b"--split-string='python3 -I'",
    ] {
        assert_eq!(
            super::selected_utility(arguments),
            Some(super::UtilitySelection::Known(b"python3".to_vec()))
        );
    }
}

#[test]
fn arguments_after_the_utility_cannot_replace_it() {
    for arguments in [
        b"sh -c python3".as_slice(),
        b"-S sh -c 'echo python3'",
        b"-S \"sh -c 'echo python3'\"",
    ] {
        assert_eq!(
            super::selected_utility(arguments),
            Some(super::UtilitySelection::Known(b"sh".to_vec()))
        );
    }
}

#[test]
fn unresolved_selected_utility_is_ambiguous() {
    assert_eq!(
        super::selected_utility(b"-S '${UNSET_INTERPRETER}sh'"),
        Some(super::UtilitySelection::Ambiguous)
    );
}

#[test]
fn unambiguous_long_option_abbreviations_preserve_the_utility() {
    for arguments in [
        b"--spl=python3 -I".as_slice(),
        b"--ignore-e python3",
        b"--uns PYTHONHOME python3",
    ] {
        assert_eq!(
            super::selected_utility(arguments),
            Some(super::UtilitySelection::Known(b"python3".to_vec()))
        );
    }
}
