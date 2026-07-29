//! This module owns bounded shell-word splitting for `env -S`.

pub(super) fn split_words(input: &[u8]) -> Option<Vec<Vec<u8>>> {
    let mut splitter = WordSplitter::default();
    for byte in input {
        splitter.observe(*byte);
    }
    splitter.finish()
}

#[derive(Default)]
struct WordSplitter {
    words: Vec<Vec<u8>>,
    word: Vec<u8>,
    started: bool,
    quote: Quote,
    escaped: bool,
}

impl WordSplitter {
    fn observe(&mut self, byte: u8) {
        if self.escaped {
            self.word.push(byte);
            self.escaped = false;
            return;
        }
        match (self.quote, byte) {
            (Quote::None, byte) if byte.is_ascii_whitespace() => self.complete_word(),
            (Quote::None, b'\'') => self.start_quote(Quote::Single),
            (Quote::None, b'"') => self.start_quote(Quote::Double),
            (Quote::None | Quote::Double, b'\\') => {
                self.escaped = true;
                self.started = true;
            }
            (Quote::Single, b'\'') | (Quote::Double, b'"') => self.quote = Quote::None,
            (_, byte) => {
                self.word.push(byte);
                self.started = true;
            }
        }
    }

    fn complete_word(&mut self) {
        if self.started {
            self.words.push(std::mem::take(&mut self.word));
            self.started = false;
        }
    }

    const fn start_quote(&mut self, quote: Quote) {
        self.quote = quote;
        self.started = true;
    }

    fn finish(mut self) -> Option<Vec<Vec<u8>>> {
        if self.escaped || self.quote != Quote::None {
            return None;
        }
        self.complete_word();
        Some(self.words)
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum Quote {
    #[default]
    None,
    Single,
    Double,
}
