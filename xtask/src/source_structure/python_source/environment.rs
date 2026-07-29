//! This module owns deterministic `env` shebang utility selection.

mod long_options;
mod short_options;
mod word_split;

use std::collections::VecDeque;

use long_options::{LongOptionAction, action as long_option_action};
use short_options::{ShortOptionAction, action as short_option_action};
use word_split::split_words;

/// The selected `env` utility when repository bytes determine it safely.
#[derive(Debug, Eq, PartialEq)]
pub(super) enum UtilitySelection {
    /// The exact utility word selected after options and assignments.
    Known(Vec<u8>),
    /// Runtime environment substitution can change the selected utility.
    Ambiguous,
}

enum OptionAction {
    Consumed,
    End,
    Utility,
}

/// Selects the utility executed by an `env` shebang.
///
/// Invalid word framing returns absence. A selected word containing unresolved
/// `${VAR}` syntax remains explicitly ambiguous so callers can fail closed.
pub(super) fn selected_utility(arguments: &[u8]) -> Option<UtilitySelection> {
    let mut words = VecDeque::from(split_words(arguments)?);
    let mut options = true;
    let mut split_budget = arguments.len().checked_add(1)?;
    while let Some(word) = words.pop_front() {
        if options {
            match option_action(&word, &mut words, &mut split_budget)? {
                OptionAction::Consumed => continue,
                OptionAction::End => {
                    options = false;
                    continue;
                }
                OptionAction::Utility => {}
            }
        }
        if word.contains(&b'=') {
            options = false;
            continue;
        }
        if word.windows(2).any(|window| window == b"${") {
            return Some(UtilitySelection::Ambiguous);
        }
        return Some(UtilitySelection::Known(word));
    }
    None
}

fn option_action(
    word: &[u8],
    words: &mut VecDeque<Vec<u8>>,
    split_budget: &mut usize,
) -> Option<OptionAction> {
    if word == b"--" {
        return Some(OptionAction::End);
    }
    if word == b"-" {
        return Some(OptionAction::Consumed);
    }
    if let Some(action) = long_option_action(word) {
        return match action {
            LongOptionAction::Consumed => Some(OptionAction::Consumed),
            LongOptionAction::TakesNext => {
                words.pop_front()?;
                Some(OptionAction::Consumed)
            }
            LongOptionAction::Split(split) => expand_split(split, words, split_budget),
            LongOptionAction::Invalid => None,
        };
    }
    if let Some(split) = short_split_value(word) {
        return expand_split(split, words, split_budget);
    }
    match short_option_action(word) {
        Some(ShortOptionAction::Consumed) => Some(OptionAction::Consumed),
        Some(ShortOptionAction::TakesNext) => {
            words.pop_front()?;
            Some(OptionAction::Consumed)
        }
        Some(ShortOptionAction::Split(split)) => expand_split(split, words, split_budget),
        Some(ShortOptionAction::Invalid) => None,
        None if word.starts_with(b"-") => None,
        None => Some(OptionAction::Utility),
    }
}

fn expand_split(
    split: &[u8],
    words: &mut VecDeque<Vec<u8>>,
    split_budget: &mut usize,
) -> Option<OptionAction> {
    *split_budget = split_budget.checked_sub(1)?;
    *words = expanded_words(split, std::mem::take(words))?;
    Some(OptionAction::Consumed)
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

fn short_split_value(word: &[u8]) -> Option<&[u8]> {
    if word == b"-S" {
        Some(b"")
    } else {
        word.strip_prefix(b"-S").filter(|value| !value.is_empty())
    }
}

#[cfg(test)]
mod tests;
