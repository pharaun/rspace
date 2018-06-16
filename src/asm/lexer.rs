use std::str::Chars;
use std::iter::Peekable;


#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Num(u32), // Only decimals or hex
    Colon,
}


// Lexer
struct Lexer<'a> {
    input_iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer { input_iter: input.chars().peekable() }
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
            if c.is_whitespace() {
                self.discard_char();
            } else {
                break;
            }
        }
    }

    fn skip_till_eol(&mut self) {
        while let Some(&c) = self.peek_char() {
            match c {
                '\n' => break,
                _ => self.discard_char(),
            }
        }
    }

    fn read_ident(&mut self, c: char) -> String {
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
            match c {
                '/' => {
                    if let Some(&'/') = self.peek_char() {
                        // Comment, eat it
                        self.skip_till_eol();
                        self.next_token()
                    } else {
                        panic!("Comment - Illegal");
                    }
                },
                ':' => Some(Token::Colon),
                '-' => Some(Token::Num((self.read_digits('0', 10) as i32 * -1) as u32)),

                // Possibly Hex
                '0' => if let Some(&'x') = self.peek_char() {
                    self.discard_char();
                    Some(Token::Num(self.read_digits('0', 16)))
                } else {
                    Some(Token::Num(self.read_digits('0', 10)))
                },
                _ => {
                    if c.is_alphabetic() {
                        Some(Token::Str(self.read_ident(c)))
                    } else if c.is_digit(10) {
                        Some(Token::Num(self.read_digits(c, 10)))
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        self.next_token()
    }
}

// TODO: add tests to handle parse fail for hex+numbers and have it fall back to str (to handle the 2f case possibly)
#[cfg(test)]
pub mod lexer_token {
    use super::*;

    #[test]
    fn test_line() {
        let input = "la: 2: addi x0 fp 1 -1 0xAF 2f asdf // asdf";
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
            // TODO: Do we want this block to specifically be a string?
            Some(Token::Num(2)),
            Some(Token::Str("f".to_string())),
            // End
            Some(Token::Str("asdf".to_string())),
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
        let input = "addi x0\naddi x1\n";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x0".to_string())),
            Some(Token::Str("addi".to_string())),
            Some(Token::Str("x1".to_string())),
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



// TODO: add data/memory labels
#[derive(Debug, PartialEq)]
enum LabelType { Global, Local }

#[derive(Debug, PartialEq)]
enum Arg {
    Num(u32),
    Label(String, LabelType),
    Str(String), // For now - Reg or CSR
}

#[derive(Debug, PartialEq)]
enum PToken {
    Label(String, LabelType),
    Inst(String, Vec<Arg>),
}


// Parser
struct Parser<'a> {
    input_iter: Peekable<Lexer<'a>>,
}


impl<'a> Parser<'a> {
    pub fn new(input: Lexer<'a>) -> Parser<'a> {
        Parser { input_iter: input.peekable() }
    }

    fn discard_token(&mut self) {
        let _ = self.input_iter.next();
    }

    fn read_token(&mut self) -> Option<Token> {
        self.input_iter.next()
    }

    fn peek_token(&mut self) -> Option<&Token> {
        self.input_iter.peek()
    }

    pub fn next_token(&mut self) -> Option<PToken> {
        if let Some(t) = self.read_token() {
            match t {
                Token::Str(s) => {
                    if let Some(&Token::Colon) = self.peek_token() {
                        // Is a Global Label
                        self.discard_token();
                        Some(PToken::Label(s, LabelType::Global))
                    } else {
                        None
                    }
                },
                Token::Num(n) => {
                    if let Some(&Token::Colon) = self.peek_token() {
                        // Is a Local Label
                        self.discard_token();
                        Some(PToken::Label(n.to_string(), LabelType::Local))
                    } else {
                        None
                    }
                },
                Token::Colon => panic!("Should not see a colon"),
            }
        } else {
            None
        }
    }
}



#[cfg(test)]
pub mod parser_ast {
    use super::*;

    #[test]
    fn test_line() {
        let input = "la: 2: addi x0 fp 1 -1 0xAF 2f asdf // asdf";
        let mut parser = Parser::new(Lexer::new(input));

        let neg: i32 = -1;
        let expected = vec![
            Some(PToken::Label("la".to_string(), LabelType::Global)),
            Some(PToken::Label("2".to_string(), LabelType::Local)),
            Some(PToken::Inst("addi".to_string(), vec![
                Arg::Str("x0".to_string()),
                Arg::Str("fp".to_string()),
                Arg::Num(1),
                Arg::Num(neg as u32),
                Arg::Num(0xAF),
                // Should have more data here
                Arg::Label("2f".to_string(), LabelType::Local),
                Arg::Str("asdf".to_string()),
            ])),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &parser.next_token();
            println!("expected {:?}, parsed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }
}


//use std::str::FromStr;
//use asm::parse;
//use asm::ast;
//
//grammar;
//
//match {
//    r"(zero|ra|[sgtf]p|[tsax][0-9]+)"   => REG,
//} else {
//    r"-?[0-9]+"                         => NUM,
//    r"0x[0-9A-F-a-f]+"                  => HEX,
//    r"[0-9]+[BFbf]"                     => NUMLAB,
//    r"[A-Za-z.]+"                       => STR,
//    _
//}
//
//// number == digits + Ox{digits} (does not handle negative number) unsigned int32/64) (later can
////                              handle negative by converting it to 2's compat and  storing it as unsigned)
//pub Number = { Dec, Hex };
//
//Dec: u32 = <s:NUM> => u32::from_str(s).unwrap();
//Hex: u32 = <s:HEX> => u32::from_str_radix(&s[2..], 16).unwrap();
//
//// Register == letter + digits
//pub Register: ast::Reg = {
//    <n:Reg> => ast::Reg::from_str(n).unwrap(),
//};
//
//Reg: &'input str = <s:REG> => s;
//
//
//// args = register | number | csr | label
//pub Arguments: ast::Args <'input> = {
//    <n:Register> => ast::Args::Reg(n),
//    <n:Number>   => ast::Args::Num(n),
//    <n:LLabel>   => ast::Args::Lab(ast::Labels::NLabel(n)),
//    <n:Str>      => {
//        // Parse CSR else panic
//        if parse::is_csr(n) {
//            ast::Args::Csr(n)
//        } else {
//            ast::Args::Lab(ast::Labels::WLabel(n))
//        }
//    },
//};
//
//Str: &'input str = <s:STR> => s;
//LLabel: &'input str = <s:NUMLAB> => s;
//WLabel: &'input str = <s:STR> => s;
//
//// [0-4] args
//pub VecArgs: Vec<ast::Args <'input>> = {
//    <Arguments*>
//};
//
//// Instruction == letter + .
//Instruction: &'input str = <s:STR> => s;
//
//// Non Instruction Labels
//pub Label: ast::Labels <'input> = {
//    <l:NUM> ":"     => ast::Labels::NLabel(l),
//    <l:WLabel> ":"  => ast::Labels::WLabel(l),
//};
//
//// Asm Line = Instruction [0-4] args
//// TODO:
////  gcc as assemblier uses a slightly different syntax:
////      - add x0, x1, x3
////      - lw x0, 0x0(x3)
////      - sw x0, 0x0(x3)
////      - csr{rw, rs, rc} a0, cycle, x0
////      - csr{rw, rs, rc}i a1, sscratch, 1
//pub AsmLine: ast::AsmLine <'input> = {
//    <l:Label> <i:Instruction> <v:VecArgs>  => ast::AsmLine::Lns(l, i, v),
//    <i:Instruction> <v:VecArgs>            => ast::AsmLine::Ins(i, v),
//    <l:Label>                              => ast::AsmLine::Lab(l),
//};
