//! This module owns deterministic `env` shebang utility selection.

mod word_split;

use std::collections::VecDeque;

use word_split::split_words;

pub(super) fn selected_utility(arguments: &[u8]) -> Option<Vec<u8>> {
    let mut words = VecDeque::from(split_words(arguments)?);
    let mut options = true;
    let mut split_budget = arguments.len().checked_add(1)?;
    while let Some(word) = words.pop_front() {
        if options {
            if word == b"--" {
                options = false;
                continue;
            }
            if word == b"-" || is_flag(&word) {
                continue;
            }
            if option_takes_value(&word) {
                words.pop_front()?;
                continue;
            }
            if option_has_value(&word) {
                continue;
            }
            if let Some(split) = split_value(&word) {
                split_budget = split_budget.checked_sub(1)?;
                words = expanded_words(split, words)?;
                options = true;
                continue;
            }
            if word.starts_with(b"-") {
                return None;
            }
        }
        if word.contains(&b'=') {
            options = false;
            continue;
        }
        return Some(word);
    }
    None
}

fn expanded_words(first: &[u8], remaining: VecDeque<Vec<u8>>) -> Option<VecDeque<Vec<u8>>> {
    let mut input = first.to_vec();
    for word in remaining {
        if !input.is_empty() {
            input.push(b' ');
        }
        input.extend_from_slice(&word);
    }
    Some(VecDeque::from(split_words(&input)?))
}

fn split_value(word: &[u8]) -> Option<&[u8]> {
    if word == b"-S" || word == b"--split-string" {
        Some(b"")
    } else {
        word.strip_prefix(b"-S")
            .filter(|value| !value.is_empty())
            .or_else(|| word.strip_prefix(b"--split-string="))
    }
}

fn option_takes_value(word: &[u8]) -> bool {
    matches!(
        word,
        b"-u" | b"--unset" | b"-C" | b"--chdir" | b"-a" | b"--argv0"
    )
}

fn option_has_value(word: &[u8]) -> bool {
    [b"-u".as_slice(), b"-C", b"-a"].iter().any(|prefix| {
        word.strip_prefix(*prefix)
            .is_some_and(|value| !value.is_empty())
    }) || [
        b"--unset=".as_slice(),
        b"--chdir=".as_slice(),
        b"--argv0=".as_slice(),
    ]
    .iter()
    .any(|prefix| word.starts_with(prefix))
}

fn is_flag(word: &[u8]) -> bool {
    matches!(
        word,
        b"--ignore-environment"
            | b"--debug"
            | b"--null"
            | b"--help"
            | b"--version"
            | b"--list-signal-handling"
    ) || is_short_flag_set(word)
        || [
            b"--block-signal".as_slice(),
            b"--default-signal",
            b"--ignore-signal",
        ]
        .iter()
        .any(|prefix| {
            word == *prefix
                || word
                    .strip_prefix(*prefix)
                    .is_some_and(|value| value.starts_with(b"="))
        })
}

fn is_short_flag_set(word: &[u8]) -> bool {
    word.strip_prefix(b"-").is_some_and(|flags| {
        !flags.is_empty() && flags.iter().all(|flag| matches!(flag, b'i' | b'v' | b'0'))
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn options_assignments_and_split_strings_precede_the_utility() {
        for arguments in [
            b"-i python3 -I".as_slice(),
            b"-iv python3",
            b"-u PYTHONHOME python3",
            b"NAME=value python3",
            b"-S -i python3 -I",
            b"-S \"python3 -I\"",
            b"--split-string='python3 -I'",
        ] {
            assert_eq!(
                super::selected_utility(arguments),
                Some(b"python3".to_vec())
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
            assert_eq!(super::selected_utility(arguments), Some(b"sh".to_vec()));
        }
    }
}
