use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug)]
pub struct Tokenizer {
    line: Vec<char>,
    cursor: usize,
}

impl Tokenizer {
    pub fn new(line: &str) -> Self {
        Self {
            line: line.to_string().chars().collect(),
            cursor: 0,
        }
    }

    pub fn read_to(&mut self, ch: char) -> Option<String> {
        let maybe_ch_idx = self.line[self.cursor..].iter().position(|&c| c == ch);
        let ch_idx = match maybe_ch_idx {
            // TODO(PT): Distinct states for 'not found' vs. 'EOF'?
            None => return None,
            Some(idx) => idx,
        };
        let part = self.line[self.cursor..self.cursor + ch_idx].iter().collect::<String>();
        // Skip the delimiter as well
        self.cursor += ch_idx + 1;
        Some(part)
    }

    /// Read the remainder of the buffer
    pub fn read(&mut self) -> Option<String> {
        if self.cursor == self.line.len() {
            return None;
        }
        let remainder = self.line[self.cursor..].iter().collect::<String>();
        self.cursor += remainder.len();
        Some(remainder)
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;
    use crate::irc::Tokenizer;

    #[test]
    fn test_tokenizer() {
        let mut t = Tokenizer::new(&"This is a test");
        assert_eq!(t.read_to(' '), Some("This".to_string()));
        assert_eq!(t.read_to(' '), Some("is".to_string()));
        assert_eq!(t.read_to(' '), Some("a".to_string()));
        assert_eq!(t.read_to(' '), None);
        assert_eq!(t.read(), Some("test".to_string()));
        assert_eq!(t.read(), None);
    }
}

