use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Str(String),
    Num(u32), // Decimal, Hex, Binary, Octal
    Eol,
    Colon,
    Dot,
    Comma,
    Plus,
    Minus,
    Dollar,
    OpenBrace,
    CloseBrace,
    OpenAngle,
    CloseAngle,
}

pub struct Lexer<'a> {
    input_iter: Peekable<Chars<'a>>,
    mode: LexMode,
}
enum LexMode { Emitted, Useful }

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input_iter: input.chars().peekable(),
            mode: LexMode::Emitted,
        }
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

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek_char() {
            // Dont skip newlines
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

    fn skip_to_eol(&mut self) {
        while let Some(&c) = self.peek_char() {
            match c {
                '\n' => break,
                _ => self.discard_char(),
            }
        }
    }

    fn read_str(&mut self, c: char) -> String {
        let mut ident = String::new();
        ident.push(c);

        while let Some(&c) = self.peek_char() {
            if c.is_alphanumeric() {
                ident.push(self.read_char().unwrap());
            } else {
                break;
            }
        }
        ident
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

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if let Some(c) = self.read_char() {

            // State machine bits to make it so that we can skip
            // multiple newlines & comments without mashing up
            // the useful tokens together without a new line
            match c {
                ';' => {
                    self.skip_to_eol();
                    return self.next_token();
                },
                '\n' => {
                    self.skip_newline();
                    return match self.mode {
                        LexMode::Useful => {
                            self.mode = LexMode::Emitted;
                            Some(Token::Eol)
                        }
                        LexMode::Emitted => self.next_token(),
                    }
                },
                _ => {
                    self.mode = LexMode::Useful;
                },
            }

            // Handle everything else
            match c {
                ':' => Some(Token::Colon),
                '.' => Some(Token::Dot),
                ',' => Some(Token::Comma),
                '+' => Some(Token::Plus),
                '-' => Some(Token::Minus),
                '$' => Some(Token::Dollar),
                '[' => Some(Token::OpenBrace),
                ']' => Some(Token::CloseBrace),
                '<' => Some(Token::OpenAngle),
                '>' => Some(Token::CloseAngle),

                // Possibly a number
                '0' => match self.peek_char() {
                    Some('x') => {
                        self.discard_char();
                        Some(Token::Num(self.read_digits('0', 16)))
                    },
                    Some('b') => {
                        self.discard_char();
                        Some(Token::Num(self.read_digits('0', 2)))
                    },
                    Some('o') => {
                        self.discard_char();
                        Some(Token::Num(self.read_digits('0', 8)))
                    },
                    _ => Some(Token::Num(self.read_digits('0', 10))),
                },

                // Anything else, probs a string/label/instruction
                _ => {
                    if c.is_alphabetic() {
                        Some(Token::Str(self.read_str(c)))
                    } else if c.is_digit(10) {
                        Some(Token::Num(self.read_digits(c, 10)))
                    } else {
                        panic!("Unknown characters: {:?}", c);
                    }
                }
            }
        } else {
            // If a eol wasn't already emitted, emit at least one
            match self.mode {
                LexMode::Emitted => None,
                _ => {
                    self.mode = LexMode::Emitted;
                    Some(Token::Eol)
                },
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
    use std::iter::zip;

    fn tstr(s: &str) -> Token {
        Token::Str(s.to_string())
    }

    fn assert_asm(expected: Vec<Token>, lex: Vec<Token>) {
        for (e, t) in zip(expected.clone(), lex.clone()) {
            println!("expected {:?}, lexed {:?} ", e, t);
            assert_eq!(e, t);
        }

        // Print out the lex if the length doesn't match
        if expected.len() != lex.len() {
            for t in &lex {
                println!("Lex: {:?}", t);
            }
        }
        assert_eq!(expected.len(), lex.len());
    }

    #[test]
    fn test_empty_str() {
        let input = "";
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_alway_eol() {
        let input = "ADCA 10";
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"),
            Token::Num(10),
            Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_header_comments() {
        let input = r#"
            ; Header comments
            ; Multiline
            ADCA 10
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"),
            Token::Num(10),
            Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_footer_comments() {
        let input = r#"
            ADCA 10
            ; footer comments
            ; Multiline
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"),
            Token::Num(10),
            Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_inbetween_comments() {
        let input = r#"
            ; Hdr1
            ; Hdr2
            ADCA 10 ; Inbw1
            ADCA 11 ; Inbw2
            ; Inbw3
            ADCA 12
            ; Ftr1
            ; Ftr2
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"),
            Token::Num(10),
            Token::Eol,
            tstr("ADCA"),
            Token::Num(11),
            Token::Eol,
            tstr("ADCA"),
            Token::Num(12),
            Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_mem_word() {
        let input = "dat: .BYTE 1";
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("dat"),
            Token::Colon,
            Token::Dot,
            tstr("BYTE"),
            Token::Num(1),
            Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_number_parse() {
        let input = r#"
            ADCA 10
            ADCA 0xF ; hex
            ADCA 0b10110101 ; 01010b
            ADCA 0o44 ; 44o
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"), Token::Num(10), Token::Eol, // Decimal
            tstr("ADCA"), Token::Num(0xF), Token::Eol, // Hex
            tstr("ADCA"), Token::Num(0b10110101), Token::Eol, // Binary
            tstr("ADCA"), Token::Num(0o44), Token::Eol, // Oceot
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_addr_parse() {
        let input = r#"
            ADCA $dat
            ADCA $0x1000
            ADCA $0x22
            ADCA [0x1000]
            ADCA [dat]
            ADCA >dat
            ADCA <dat
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"), Token::Dollar, tstr("dat"), Token::Eol,
            tstr("ADCA"), Token::Dollar, Token::Num(0x1000), Token::Eol,
            tstr("ADCA"), Token::Dollar, Token::Num(0x22), Token::Eol,
            tstr("ADCA"), Token::OpenBrace, Token::Num(0x1000), Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::OpenBrace, tstr("dat"), Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::CloseAngle, tstr("dat"), Token::Eol,
            tstr("ADCA"), Token::OpenAngle, tstr("dat"), Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_indexed_parse() {
        let input = r#"
            ADCA 0,Y
            ADCA B,Y
            ADCA ,Y
            ADCA ,Y+
            ADCA ,Y++
            ADCA ,-Y
            ADCA ,--Y
            ADCA [0,Y]
            ADCA [B,Y]
            ADCA [,Y]
            ADCA [,Y++]
            ADCA [,--Y]
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"), Token::Num(0), Token::Comma, tstr("Y"), Token::Eol,
            tstr("ADCA"), tstr("B"), Token::Comma, tstr("Y"), Token::Eol,
            tstr("ADCA"), Token::Comma, tstr("Y"), Token::Eol,
            tstr("ADCA"), Token::Comma, tstr("Y"), Token::Plus, Token::Eol,
            tstr("ADCA"), Token::Comma, tstr("Y"), Token::Plus, Token::Plus, Token::Eol,
            tstr("ADCA"), Token::Comma, Token::Minus, tstr("Y"), Token::Eol,
            tstr("ADCA"), Token::Comma, Token::Minus, Token::Minus, tstr("Y"), Token::Eol,

            tstr("ADCA"), Token::OpenBrace, Token::Num(0), Token::Comma, tstr("Y"), Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::OpenBrace, tstr("B"), Token::Comma, tstr("Y"), Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::OpenBrace, Token::Comma, tstr("Y"), Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::OpenBrace, Token::Comma, tstr("Y"), Token::Plus, Token::Plus, Token::CloseBrace, Token::Eol,
            tstr("ADCA"), Token::OpenBrace, Token::Comma, Token::Minus, Token::Minus, tstr("Y"), Token::CloseBrace, Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_pc_parse() {
        let input = r#"
            ADCA 0,PC
            ADCA 0,PCR
            ADCA [0,PCR]
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("ADCA"), Token::Num(0), Token::Comma, tstr("PC"), Token::Eol,
            tstr("ADCA"), Token::Num(0), Token::Comma, tstr("PCR"), Token::Eol,
            tstr("ADCA"), Token::OpenBrace, Token::Num(0), Token::Comma, tstr("PCR"), Token::CloseBrace, Token::Eol,
        ];
        assert_asm(expected, lex);
    }

    #[test]
    fn test_misc_parse() {
        let input = r#"
            RORW
            AIM 0x3F,4,U
            BAND A,5,1,0x40
            PSHS U,Y,X,DP,CC
        "#;
        let lex: Vec<Token> = Lexer::new(input).into_iter().collect();

        let expected = vec![
            tstr("RORW"), Token::Eol,
            tstr("AIM"), Token::Num(0x3F), Token::Comma, Token::Num(4), Token::Comma, tstr("U"), Token::Eol,
            tstr("BAND"), tstr("A"), Token::Comma, Token::Num(5), Token::Comma, Token::Num(1), Token::Comma,
                Token::Num(0x40), Token::Eol,
            tstr("PSHS"), tstr("U"), Token::Comma, tstr("Y"), Token::Comma, tstr("X"), Token::Comma, tstr("DP"),
                Token::Comma, tstr("CC"), Token::Eol,
        ];
        assert_asm(expected, lex);
    }
}
