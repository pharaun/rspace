use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum AddrRefType { Forward, Backward }

// Plain label == AddrRef
// [label] == MemRef
#[derive(Debug, PartialEq)]
pub enum Token {
    Str(String),
    Num(u32), // Only decimals or hex
    Colon,
    Newline,
    // TODO: support data decl (.byte 0x08 0x08 0x08) for eg
    //Dot,
    MemRef(String), // Only global labels can be a memref
    AddrRef(String, AddrRefType), // Only support local labels for now
}

// Lexer
pub struct Lexer<'a> {
    input_iter: Peekable<Chars<'a>>,
    emit_newline: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { input_iter: input.chars().peekable(), emit_newline: false}
    }

    fn discard_char(&mut self) {
        let _ = self.input_iter.next();
    }

    fn read_char(&mut self) -> Option<char> {
        self.input_iter.next()
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.input_iter.peek()
    }

    // Dont skip newlines
    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek_char() {
            if c.is_whitespace() && c != '\n' {
                self.discard_char();
            } else {
                break;
            }
        }
    }

    fn skip_newline(&mut self) {
        while let Some(&c) = self.peek_char() {
            match c {
                '\n' => self.discard_char(),
                _ => break,
            }
        }
    }

    fn skip_till_and_eat_eol(&mut self) {
        while let Some(&c) = self.peek_char() {
            match c {
                '\n' => break,
                _ => self.discard_char(),
            }
        }
        self.skip_newline();
    }

    fn read_ident(&mut self, c: char) -> String {
        let mut ident = String::new();
        ident.push(c);

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() {
                ident.push(self.read_char().unwrap());
            // Allow . in ident name for fence.i
            } else if c == '.' {
                ident.push(self.read_char().unwrap());
            } else {
                break;
            }
        }
        ident
    }

    fn read_memref(&mut self) -> String {
        let mut memref = String::new();

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() {
                memref.push(self.read_char().unwrap());
            // When find ']' the memref is closed
            } else if c == ']' {
                self.discard_char();
                break;
            } else {
                // We found something else, this is bad
                panic!("Found {}, should be alphanumeric or a ]", c);
            }
        }
        memref
    }

    fn read_digits(&mut self, c: char, radix: u32) -> u32 {
        let mut digits = String::new();
        digits.push(c);

        while let Some(&c) = self.peek_char() {
            if c.is_digit(radix) {
                digits.push(self.read_char().unwrap());
            } else {
                break;
            }
        }
        u32::from_str_radix(&digits, radix).unwrap()
    }

    fn read_digits_or_label(&mut self, c: char) -> Token {
        let mut label = None;
        let mut maybe_digits = String::new();
        maybe_digits.push(c);

        while let Some(&c) = self.peek_char() {
            if c.is_digit(10) {
                maybe_digits.push(self.read_char().unwrap());
            } else if c == 'f' {
                self.discard_char();
                label = Some(AddrRefType::Forward);
                break;
            } else if c == 'b' {
                self.discard_char();
                label = Some(AddrRefType::Backward);
                break;
            } else {
                break;
            }
        }

        match label {
            Some(l) => Token::AddrRef(maybe_digits, l),
            None    => Token::Num(u32::from_str_radix(&maybe_digits, 10).unwrap()),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(c) = self.read_char() {
            match c {
                '/' => {
                    if let Some(&'/') = self.peek_char() {
                        // Comment, eat it
                        self.skip_till_and_eat_eol();
                        self.next_token()
                    } else {
                        panic!("Comment - Illegal");
                    }
                },
                '\n' => {
                    // Ingest multiple newlines
                    self.skip_newline();

                    if let None = self.peek_char() {
                        // Will EOF, set eof_newline to true so we don't duplicate newline
                        self.emit_newline = true;
                    }
                    Some(Token::Newline)
                },
                ':' => Some(Token::Colon),
                '-' => Some(Token::Num((self.read_digits('0', 10) as i32 * -1) as u32)),
                '[' => Some(Token::MemRef(self.read_memref())),

                // Possibly Hex
                '0' => if let Some(&'x') = self.peek_char() {
                    self.discard_char();
                    Some(Token::Num(self.read_digits('0', 16)))
                } else {
                    Some(self.read_digits_or_label('0'))
                },
                _ => {
                    if c.is_alphabetic() {
                        Some(Token::Str(self.read_ident(c)))
                    } else if c.is_digit(10) {
                        Some(self.read_digits_or_label(c))
                    } else {
                        println!("Unknown characters: {:?}", c);
                        panic!("Isn't an alphabetic or digits")
                    }
                }
            }
        } else {
            // Always emit a newline before the eof (unless one was already emitted)
            if !self.emit_newline {
                self.emit_newline = true;
                Some(Token::Newline)
            } else {
                None
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        self.next_token()
    }
}

#[cfg(test)]
pub mod lexer_token {
    use super::*;

    #[test]
    fn test_line() {
        let input = "la: 2: addi x0 fp 1 -1 0xAF 2f 2b asdf [qwer] // Comments";
        let mut lexer = Lexer::new(input);

        let neg: i32 = -1;
        let expected = vec![
            Some(Token::Str("la".to_string())),
            Some(Token::Colon),
            Some(Token::Num(2)),
            Some(Token::Colon),
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x0".to_string())),
            Some(Token::Str("fp".to_string())),
            Some(Token::Num(1)),
            Some(Token::Num(neg as u32)),
            Some(Token::Num(0xAF)),
            Some(Token::AddrRef("2".to_string(), AddrRefType::Forward)),
            Some(Token::AddrRef("2".to_string(), AddrRefType::Backward)),
            Some(Token::Str("asdf".to_string())),
            Some(Token::MemRef("qwer".to_string())),
            // Comments are discarded
            Some(Token::Newline),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &lexer.next_token();
            println!("expected {:?}, lexed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }

    #[test]
    fn test_multiline() {
        let input = "addi x0\n// Comments\n\nla:\naddi x1\n\n\naddi x2";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x0".to_string())),
            Some(Token::Newline),
            Some(Token::Str("la".to_string())),
            Some(Token::Colon),
            Some(Token::Newline),
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x1".to_string())),
            Some(Token::Newline),
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x2".to_string())),
            // Always have a newline before EOF
            Some(Token::Newline),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &lexer.next_token();
            println!("expected {:?}, lexed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }

    #[test]
    fn test_eof_newline() {
        let input = "addi x1\n";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x1".to_string())),
            // Always have exactly one newline before EOF
            Some(Token::Newline),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &lexer.next_token();
            println!("expected {:?}, lexed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }

    #[test]
    fn test_dot_asm() {
        let input = "fence.i\n";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Some(Token::Str("fence.i".to_string())),
            // Always have exactly one newline before EOF
            Some(Token::Newline),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &lexer.next_token();
            println!("expected {:?}, lexed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }
}
