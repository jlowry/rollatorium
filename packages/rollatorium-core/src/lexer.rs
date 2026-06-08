use crate::{error::RollatoriumError, token::Token};

pub(crate) struct Lexer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    annotation_mode: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            chars: input.char_indices().collect(),
            pos: 0,
            annotation_mode: false,
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> char {
        self.peek_offset(0)
    }

    fn peek_offset(&self, offset: usize) -> char {
        let idx = self.pos + offset;
        self.chars.get(idx).map(|&(_, c)| c).unwrap_or('\0')
    }

    /// Byte offset of the char at `idx`, or the end of the input when `idx` is
    /// at or past the final char. Used to slice borrowed sub-strings of `input`.
    fn byte_at(&self, idx: usize) -> usize {
        self.chars
            .get(idx)
            .map(|&(b, _)| b)
            .unwrap_or(self.input.len())
    }

    fn advance(&mut self) {
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
    }

    fn advance_by(&mut self, count: usize) {
        for _ in 0..count {
            self.advance();
        }
    }

    fn skip_ws(&mut self) {
        while !self.is_at_end() && self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn starts_with(&self, pattern: &str) -> bool {
        pattern
            .chars()
            .enumerate()
            .all(|(idx, ch)| self.peek_offset(idx) == ch)
    }

    fn number(&mut self) -> crate::Result<Token<'a>> {
        let start = self.pos;
        let mut seen_digit = false;
        let mut seen_dot = false;

        while !self.is_at_end() {
            let c = self.peek();
            if c.is_ascii_digit() {
                seen_digit = true;
                self.advance();
            } else if c == '.' && !seen_dot {
                let next = self.peek_offset(1);
                if !next.is_ascii_digit() {
                    return Err(RollatoriumError::Lexer(format!(
                        "Invalid decimal literal starting at position {}",
                        start
                    )));
                }
                seen_dot = true;
                self.advance();
            } else {
                break;
            }
        }

        if !seen_digit {
            return Err(RollatoriumError::Lexer(format!(
                "Number literal missing digits at position {}",
                start
            )));
        }

        let num_str = &self.input[self.byte_at(start)..self.byte_at(self.pos)];
        match num_str.parse::<f64>() {
            // A literal that overflows `f64` parses to an infinity; reject it so
            // both `parse` and the `dice!` macro fail cleanly instead of carrying
            // a non-finite value forward (the macro asserts finiteness).
            Ok(value) if value.is_finite() => Ok(Token::Number(value)),
            Ok(_) => Err(RollatoriumError::Lexer(format!(
                "Number literal '{}' is too large to represent",
                num_str
            ))),
            Err(_) => Err(RollatoriumError::Lexer(format!(
                "Failed to parse number literal '{}'",
                num_str
            ))),
        }
    }

    pub fn next_token(&mut self) -> crate::Result<Token<'a>> {
        if !self.annotation_mode {
            self.skip_ws();
        }
        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        if self.annotation_mode {
            let start = self.pos;
            while !self.is_at_end() {
                let c = self.peek();
                if c == ']' {
                    break;
                }
                self.advance();
            }

            if self.is_at_end() {
                return Err(RollatoriumError::Lexer(
                    "Unterminated annotation; missing closing ']'".into(),
                ));
            }

            let raw = &self.input[self.byte_at(start)..self.byte_at(self.pos)];
            self.annotation_mode = false;
            // `str::trim` returns a borrowed sub-slice of `input`; no allocation.
            return Ok(Token::AnnotationText(raw.trim()));
        }

        if self.starts_with("//") {
            self.advance_by(2);
            return Ok(Token::DoubleSlash);
        }
        if self.starts_with("==") {
            self.advance_by(2);
            return Ok(Token::EqualEqual);
        }
        if self.starts_with("!=") {
            self.advance_by(2);
            return Ok(Token::NotEqual);
        }
        if self.starts_with(">=") {
            self.advance_by(2);
            return Ok(Token::GreaterEqual);
        }
        if self.starts_with("<=") {
            self.advance_by(2);
            return Ok(Token::LessEqual);
        }
        if self.starts_with("rr") {
            self.advance_by(2);
            return Ok(Token::Reroll);
        }
        if self.starts_with("ro") {
            self.advance_by(2);
            return Ok(Token::RerollOnce);
        }
        if self.starts_with("ra") {
            self.advance_by(2);
            return Ok(Token::RerollAdd);
        }
        if self.starts_with("mi") {
            self.advance_by(2);
            return Ok(Token::Min);
        }
        if self.starts_with("ma") {
            self.advance_by(2);
            return Ok(Token::Max);
        }

        if self.starts_with("d%") {
            self.advance_by(2);
            return Ok(Token::DicePercent);
        }

        let c = self.peek();
        match c {
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                Ok(Token::Minus)
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '/' => {
                self.advance();
                Ok(Token::Slash)
            }
            '%' => {
                self.advance();
                Ok(Token::Percent)
            }
            '>' => {
                self.advance();
                Ok(Token::Greater)
            }
            '<' => {
                self.advance();
                Ok(Token::Less)
            }
            '(' => {
                self.advance();
                Ok(Token::LParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RParen)
            }
            '{' => {
                self.advance();
                Ok(Token::SetStart)
            }
            '}' => {
                self.advance();
                Ok(Token::SetEnd)
            }
            '[' => {
                self.advance();
                self.annotation_mode = true;
                Ok(Token::AnnotationStart)
            }
            ']' => {
                self.advance();
                Ok(Token::AnnotationEnd)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            'd' => {
                self.advance();
                Ok(Token::Dice)
            }
            'k' => {
                self.advance();
                Ok(Token::Keep)
            }
            'p' => {
                self.advance();
                Ok(Token::Drop)
            }
            'e' => {
                self.advance();
                Ok(Token::Explode)
            }
            '!' => {
                self.advance();
                Ok(Token::Explode)
            }
            'h' => {
                self.advance();
                Ok(Token::SelectorHigh)
            }
            'l' => {
                self.advance();
                Ok(Token::SelectorLow)
            }
            '=' => Err(RollatoriumError::Lexer(format!(
                "Unexpected '=' at position {}. Did you mean '=='?",
                self.pos
            ))),
            c if c.is_ascii_digit() || (c == '.' && self.peek_offset(1).is_ascii_digit()) => {
                self.number()
            }
            _ => Err(RollatoriumError::Lexer(format!(
                "Unexpected character '{}' at position {}",
                c, self.pos
            ))),
        }
    }
}
