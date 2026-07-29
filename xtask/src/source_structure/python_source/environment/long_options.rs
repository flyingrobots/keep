//! This module owns GNU `env` long-option abbreviation admission.

#[derive(Clone, Copy)]
enum ValueMode {
    None,
    Required,
    Optional,
    Split,
}

#[derive(Clone, Copy)]
struct LongOption {
    name: &'static [u8],
    value: ValueMode,
}

struct LongOptionWord<'a> {
    name: &'a [u8],
    attached: Option<&'a [u8]>,
}

const OPTIONS: [LongOption; 12] = [
    LongOption::new(b"--argv0", ValueMode::Required),
    LongOption::new(b"--block-signal", ValueMode::Optional),
    LongOption::new(b"--chdir", ValueMode::Required),
    LongOption::new(b"--debug", ValueMode::None),
    LongOption::new(b"--default-signal", ValueMode::Optional),
    LongOption::new(b"--help", ValueMode::None),
    LongOption::new(b"--ignore-environment", ValueMode::None),
    LongOption::new(b"--ignore-signal", ValueMode::Optional),
    LongOption::new(b"--list-signal-handling", ValueMode::None),
    LongOption::new(b"--null", ValueMode::None),
    LongOption::new(b"--split-string", ValueMode::Split),
    LongOption::new(b"--unset", ValueMode::Required),
];

/// The effect of one admitted GNU `env` long option.
pub(super) enum LongOptionAction<'a> {
    /// The option and any attached or optional value are complete.
    Consumed,
    /// The option requires the next word as its value.
    TakesNext,
    /// The option replaces the remaining words with split-string bytes.
    Split(&'a [u8]),
    /// The option forbids its attached value.
    Invalid,
}

/// Admits an exact or unambiguous abbreviated GNU `env` long option.
pub(super) fn action(word: &[u8]) -> Option<LongOptionAction<'_>> {
    let word = split_value(word)?;
    if !word.name.starts_with(b"--") {
        return None;
    }
    let option = unique_match(word.name)?;
    Some(match (option.value, word.attached) {
        (ValueMode::None, None) | (ValueMode::Optional, _) | (ValueMode::Required, Some(_)) => {
            LongOptionAction::Consumed
        }
        (ValueMode::Required, None) => LongOptionAction::TakesNext,
        (ValueMode::Split, value) => LongOptionAction::Split(value.unwrap_or_default()),
        (ValueMode::None, Some(_)) => LongOptionAction::Invalid,
    })
}

fn unique_match(name: &[u8]) -> Option<LongOption> {
    if let Some(option) = OPTIONS.iter().find(|option| option.name == name) {
        return Some(*option);
    }
    let mut matches = OPTIONS
        .iter()
        .filter(|option| option.name.starts_with(name))
        .copied();
    let option = matches.next()?;
    matches.next().is_none().then_some(option)
}

fn split_value(word: &[u8]) -> Option<LongOptionWord<'_>> {
    match word.iter().position(|byte| *byte == b'=') {
        Some(index) => {
            let name = word.get(..index)?;
            let value_start = index.checked_add(1)?;
            let value = word.get(value_start..)?;
            Some(LongOptionWord {
                name,
                attached: Some(value),
            })
        }
        None => Some(LongOptionWord {
            name: word,
            attached: None,
        }),
    }
}

impl LongOption {
    const fn new(name: &'static [u8], value: ValueMode) -> Self {
        Self { name, value }
    }
}
