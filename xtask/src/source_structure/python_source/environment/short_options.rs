//! This module owns deterministic combined short-option decoding for `env`.

pub(super) enum ShortOptionAction<'a> {
    Consumed,
    TakesNext,
    Split(&'a [u8]),
    Invalid,
}

pub(super) fn action(word: &[u8]) -> Option<ShortOptionAction<'_>> {
    let mut options = word.strip_prefix(b"-")?;
    if options.is_empty() || options.starts_with(b"-") {
        return None;
    }
    loop {
        let (option, remaining) = options.split_first()?;
        match option {
            b'i' | b'v' | b'0' if remaining.is_empty() => {
                return Some(ShortOptionAction::Consumed);
            }
            b'i' | b'v' | b'0' => options = remaining,
            b'u' | b'C' | b'a' if remaining.is_empty() => {
                return Some(ShortOptionAction::TakesNext);
            }
            b'u' | b'C' | b'a' => return Some(ShortOptionAction::Consumed),
            b'S' => return Some(ShortOptionAction::Split(remaining)),
            _ => return Some(ShortOptionAction::Invalid),
        }
    }
}
