use crate::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut l = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
        };
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        self.column += 1;
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let start_col = self.column;
        let start_line = self.line;

        let kind = match self.ch {
            '+' => { self.read_char(); TokenKind::Plus }
            '-' => {
                if self.peek_char() == '>' {
                    self.read_char();
                    self.read_char();
                    TokenKind::Arrow
                } else {
                    self.read_char();
                    TokenKind::Minus
                }
            }
            '*' => { self.read_char(); TokenKind::Star }
            '/' => {
                if self.peek_char() == '/' {
                    // It's a comment. Consume the '//' then read until newline or EOF.
                    self.read_char(); // consume first '/'
                    self.read_char(); // consume second '/'
                    let pos = self.position;
                    while self.ch != '\n' && self.ch != '\0' {
                        self.read_char();
                    }
                    let text: String = self.input[pos..self.position].iter().collect();
                    return Token {
                        kind: TokenKind::Comment(text.trim_end().to_string()),
                        line: start_line,
                        column: start_col,
                    };
                } else {
                    self.read_char();
                    TokenKind::Slash
                }
            }
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    TokenKind::Equals
                } else {
                    self.read_char();
                    TokenKind::Assign
                }
            }
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    TokenKind::NotEquals
                } else {
                    self.read_char();
                    TokenKind::Illegal
                }
            }
            '>' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    TokenKind::GreaterEquals
                } else {
                    self.read_char();
                    TokenKind::GreaterThan
                }
            }
            '<' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    self.read_char();
                    TokenKind::LessEquals
                } else {
                    self.read_char();
                    TokenKind::LessThan
                }
            }
            '(' => { self.read_char(); TokenKind::LParen }
            ')' => { self.read_char(); TokenKind::RParen }
            '{' => { self.read_char(); TokenKind::LBrace }
            '}' => { self.read_char(); TokenKind::RBrace }
            '[' => { self.read_char(); TokenKind::LBracket }
            ']' => { self.read_char(); TokenKind::RBracket }
            ',' => { self.read_char(); TokenKind::Comma }
            ';' => { self.read_char(); TokenKind::Semicolon }
            ':' => { self.read_char(); TokenKind::Colon }
            '.' => { self.read_char(); TokenKind::Dot }
            '"' => self.read_string(),
            '\0' => TokenKind::Eof,
            _ => {
                if self.ch.is_ascii_digit() || is_myanmar_digit(self.ch) {
                    let num_kind = self.read_number();
                    return Token {
                        kind: num_kind,
                        line: start_line,
                        column: start_col,
                    };
                } else if is_letter_or_myanmar(self.ch) && !is_myanmar_digit(self.ch) {
                    let ident = self.read_identifier();
                    return Token {
                        kind: lookup_ident(&ident),
                        line: start_line,
                        column: start_col,
                    };
                } else {
                    self.read_char();
                    TokenKind::Illegal
                }
            }
        };

        Token {
            kind,
            line: start_line,
            column: start_col,
        }
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while is_letter_or_myanmar(self.ch) {
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    fn read_number(&mut self) -> TokenKind {
        let mut value: i64 = 0;
        while self.ch.is_ascii_digit() || is_myanmar_digit(self.ch) {
            let d = if self.ch.is_ascii_digit() {
                self.ch as i64 - '0' as i64
            } else {
                self.ch as i64 - '\u{1040}' as i64
            };
            value = value * 10 + d;
            self.read_char();
        }
        // Check for float literal: digit(s) followed by '.' followed by digit
        if self.ch == '.' && (self.peek_char().is_ascii_digit() || is_myanmar_digit(self.peek_char())) {
            self.read_char(); // consume '.'
            let mut frac: f64 = 0.0;
            let mut frac_div: f64 = 1.0;
            while self.ch.is_ascii_digit() || is_myanmar_digit(self.ch) {
                let d = if self.ch.is_ascii_digit() {
                    self.ch as u32 - '0' as u32
                } else {
                    self.ch as u32 - '\u{1040}' as u32
                };
                frac = frac * 10.0 + d as f64;
                frac_div *= 10.0;
                self.read_char();
            }
            return TokenKind::FloatLiteral(value as f64 + frac / frac_div);
        }
        TokenKind::Number(value)
    }

    fn read_string(&mut self) -> TokenKind {
        self.read_char(); // skip opening quote
        let position = self.position;
        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }
        let str_val: String = self.input[position..self.position].iter().collect();
        if self.ch == '"' {
            self.read_char(); // skip closing quote
        }
        TokenKind::StringLiteral(str_val)
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            if self.ch == '\n' {
                self.line += 1;
                self.column = 0;
            }
            self.read_char();
        }
    }
}

impl Lexer {
    /// Returns the next token that is not a comment (for parser compatibility).
    pub fn next_non_comment_token(&mut self) -> Token {
        loop {
            let tok = self.next_token();
            if matches!(tok.kind, TokenKind::Comment(_)) {
                continue;
            }
            return tok;
        }
    }
}

/// Tokenizes the entire input, including comment tokens.
pub fn tokenize_all(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = tok.kind == TokenKind::Eof;
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    tokens
}

fn is_letter_or_myanmar(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ('\u{1000}' <= ch && ch <= '\u{109F}')
}

fn is_myanmar_digit(ch: char) -> bool {
    '\u{1040}' <= ch && ch <= '\u{1049}'
}

fn lookup_ident(ident: &str) -> TokenKind {
    match ident {
        // Myanglish Keywords (romanized Burmese)
        "kain" => TokenKind::Kain,
        "sar" => TokenKind::Sar,
        "sit" => TokenKind::Sit,
        "da_tha" => TokenKind::DaTha,
        "hman" => TokenKind::Hman,
        "hmar" => TokenKind::Hmar,
        "bhala" => TokenKind::Bhala,
        "hlyin" => TokenKind::Hlyin,
        "mo" => TokenKind::Mo,
        "pat" => TokenKind::Pat,
        "htae" => TokenKind::Htae,
        "loke" => TokenKind::Loke,
        "pyan" => TokenKind::Pyan,
        "pya" => TokenKind::Pya,
        "phat" => TokenKind::Phat,
        "su" => TokenKind::Su,
        "yu" => TokenKind::Yu,
        "twe" => TokenKind::Twe,
        "pone" => TokenKind::Pone,
        "nee" => TokenKind::Nee,
        "myat" => TokenKind::Myat,
        "amhar" => TokenKind::Amhar,
        _ => TokenKind::Identifier(ident.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind::*;

    #[test]
    fn test_next_token() {
        let input = r#"
            // this is a comment
            loke main() -> kain {
                kain age = ၂၀;
                su<kain> numbers = [၁, ၂];
                twe<sar, kain> dict = {"a": 1};
                phat("name?");
                yu test;
                pat item htae numbers {
                    pya(item);
                }
                hlyin (age > ၁၈) {
                    pya("adult");
                } mo {
                    pya("child");
                }
                pyan ၀;
            }
        "#;

        let mut lexer = Lexer::new(input);

        let expected = vec![
            Comment(" this is a comment".to_string()),
            Loke, Identifier("main".to_string()), LParen, RParen, Arrow, Kain, LBrace,
            Kain, Identifier("age".to_string()), Assign, Number(20), Semicolon,
            Su, LessThan, Kain, GreaterThan, Identifier("numbers".to_string()), Assign, LBracket, Number(1), Comma, Number(2), RBracket, Semicolon,
            Twe, LessThan, Sar, Comma, Kain, GreaterThan, Identifier("dict".to_string()), Assign, LBrace, StringLiteral("a".to_string()), Colon, Number(1), RBrace, Semicolon,
            Phat, LParen, StringLiteral("name?".to_string()), RParen, Semicolon,
            Yu, Identifier("test".to_string()), Semicolon,
            Pat, Identifier("item".to_string()), Htae, Identifier("numbers".to_string()), LBrace,
            Pya, LParen, Identifier("item".to_string()), RParen, Semicolon,
            RBrace,
            Hlyin, LParen, Identifier("age".to_string()), GreaterThan, Number(18), RParen, LBrace,
            Pya, LParen, StringLiteral("adult".to_string()), RParen, Semicolon,
            RBrace, Mo, LBrace,
            Pya, LParen, StringLiteral("child".to_string()), RParen, Semicolon,
            RBrace,
            Pyan, Number(0), Semicolon,
            RBrace, Eof
        ];

        for expected_tok in expected {
            let tok = lexer.next_token();
            assert_eq!(tok.kind, expected_tok);
        }
    }
}
