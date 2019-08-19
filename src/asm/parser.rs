use std::iter::Peekable;
use asm::lexer;

// Use the reg and csr func here for now
use asm::ast;
use std::str::FromStr;

// TODO: parse macro definition and usage here

#[derive(Debug, PartialEq, Clone)]
pub enum LabelType { Global, Local }

#[derive(Debug, PartialEq, Clone)]
pub enum AddrRefType { Global, LocalBackward, LocalForward }

#[derive(Debug, PartialEq, Clone)]
pub enum DataType { Byte, Half, Word }

impl FromStr for DataType {
    type Err = ParseDataTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BYTE"  => Ok(DataType::Byte),
            "HALF"  => Ok(DataType::Half),
            "WORD"  => Ok(DataType::Word),
            _           => Err(ParseDataTypeError { _priv: () }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDataTypeError { _priv: () }

#[derive(Debug, PartialEq)]
pub enum Arg {
    Num(u32),
    Reg(ast::Reg),
    Csr(ast::Csr),
    AddrRef(String, AddrRefType),
    MemRef(String),
}

#[derive(Debug, PartialEq)]
pub enum PToken {
    Label(String, LabelType),
    Inst(String, Vec<Arg>),
    // Clean up the actual bytes at a later stage
    Data(DataType, Vec<u32>),
}


// Parser
pub struct Parser<'a> {
    input_iter: Peekable<lexer::Lexer<'a>>,
}


impl<'a> Parser<'a> {
    pub fn new(input: lexer::Lexer<'a>) -> Parser<'a> {
        Parser { input_iter: input.peekable() }
    }

    fn discard_token(&mut self) {
        let _ = self.input_iter.next();
    }

    fn read_token(&mut self) -> Option<lexer::Token> {
        self.input_iter.next()
    }

    fn peek_token(&mut self) -> Option<&lexer::Token> {
        self.input_iter.peek()
    }

    fn collect_till_eol(&mut self) -> Vec<lexer::Token> {
        let mut args = Vec::new();
        while let Some(t) = self.read_token() {
            match t {
                lexer::Token::Newline => break,
                _ => args.push(t),
            }
        }
        args
    }

    pub fn next_token(&mut self) -> Option<PToken> {
        if let Some(t) = self.read_token() {
            // Check if its a label
            if let Some(&lexer::Token::Colon) = self.peek_token() {
                match t {
                    lexer::Token::Str(s) => {
                        // Is a Global Label
                        self.discard_token();
                        Some(PToken::Label(s, LabelType::Global))
                    },
                    lexer::Token::Num(n) => {
                        // Is a Local Label
                        self.discard_token();
                        Some(PToken::Label(n.to_string(), LabelType::Local))
                    },
                    lexer::Token::Colon => panic!("Should not see a colon"),
                    lexer::Token::Dot => panic!("Should not see a dot"),
                    lexer::Token::Newline => panic!("Should not see a newline"),
                    lexer::Token::AddrRef(_, _) => panic!("Should not see an addref outside of a instruction"),
                    lexer::Token::MemRef(_) => panic!("Should not see a memref outside of a instruction"),
                }
            } else {
                match t {
                    lexer::Token::Str(s) => {
                        // Instruction
                        let mut args = Vec::new();

                        for t in self.collect_till_eol() {
                            match t {
                                lexer::Token::Str(s) => {
                                    // Check if CSRR or registers
                                    if let Result::Ok(c) = ast::Csr::from_str(&s) {
                                        args.push(Arg::Csr(c));
                                    } else if let Result::Ok(r) = ast::Reg::from_str(&s) {
                                        args.push(Arg::Reg(r));
                                    } else {
                                        // Global Label
                                        args.push(Arg::AddrRef(s, AddrRefType::Global));
                                    }
                                },
                                lexer::Token::Num(i) => args.push(Arg::Num(i)),
                                lexer::Token::AddrRef(s, lexer::AddrRefType::Forward) => {
                                    args.push(Arg::AddrRef(s, AddrRefType::LocalForward))
                                },
                                lexer::Token::AddrRef(s, lexer::AddrRefType::Backward) => {
                                    args.push(Arg::AddrRef(s, AddrRefType::LocalBackward))
                                },
                                lexer::Token::MemRef(s) => {
                                    args.push(Arg::MemRef(s))
                                },
                                lexer::Token::Colon => panic!("Should not see a colon"),
                                lexer::Token::Newline => panic!("Should not see a newline"),
                                lexer::Token::Dot => panic!("Should not see a dot"),
                            }
                        }

                        // Should be reading tokens .... till some limit (new line, or eof?)
                        // Should ? do some sort of basic instruction validation here possibly (ie
                        // args count)
                        // Later on would need (ie in the assemblier stage) code to handle labels
                        // vs numbers/etc when one or the other is expected (particularly for
                        // addresses/variables)
                        Some(PToken::Inst(s.to_ascii_uppercase(), args))
                    },

                    // Its a dot, we need at least a Str to identify type of data
                    // then collect to end of line Num into a vec
                    lexer::Token::Dot => {
                        if let Some(lexer::Token::Str(s)) = self.read_token() {
                            if let Result::Ok(dt) = DataType::from_str(&s.to_ascii_uppercase()) {
                                let mut dat = Vec::new();

                                for t in self.collect_till_eol() {
                                    match t {
                                        lexer::Token::Num(n) => dat.push(n),
                                        _ => panic!("Not a number"),
                                    }
                                }

                                Some(PToken::Data(dt, dat))
                            } else {
                                panic!("Isn't a DataType: {:?}", s);
                            }
                        } else {
                            panic!("Isn't a string right after a dot")
                        }
                    },
                    // We skip newline here its only required when handling PToken::Inst
                    lexer::Token::Newline => self.next_token(),

                    lexer::Token::Num(_) => panic!("Should not see Num outside a data token or instruction"),
                    lexer::Token::Colon => panic!("Should not see a colon outside of a label"),
                    lexer::Token::AddrRef(_, _) => panic!("Should not see an addref outside of a instruction"),
                    lexer::Token::MemRef(_) => panic!("Should not see a memref outside of a instruction"),
                }
            }
        } else {
            None
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = PToken;
    fn next(&mut self) -> Option<PToken> {
        self.next_token()
    }
}


#[cfg(test)]
pub mod parser_ast {
    use super::*;

    #[test]
    fn test_line() {
        let input = "la: 2: addi x0 fp 1 -1 0xAF 2f 2b asdf MSTATUS [qwer]\n .BYTE 0xDE\n .HALF 0xDF\n .WORD 0xEA 0xEB // Comments";
        let mut parser = Parser::new(lexer::Lexer::new(input));

        let neg: i32 = -1;
        let expected = vec![
            Some(PToken::Label("la".to_string(), LabelType::Global)),
            Some(PToken::Label("2".to_string(), LabelType::Local)),
            Some(PToken::Inst("ADDI".to_string(), vec![
                Arg::Reg(ast::Reg::X0),
                Arg::Reg(ast::Reg::X8),
                Arg::Num(1),
                Arg::Num(neg as u32),
                Arg::Num(0xAF),
                // Should have more data here
                Arg::AddrRef("2".to_string(), AddrRefType::LocalForward),
                Arg::AddrRef("2".to_string(), AddrRefType::LocalBackward),
                Arg::AddrRef("asdf".to_string(), AddrRefType::Global),
                Arg::Csr(ast::Csr::MSTATUS),
                Arg::MemRef("qwer".to_string()),
            ])),
            Some(PToken::Data(DataType::Byte, vec![0xDE])),
            Some(PToken::Data(DataType::Half, vec![0xDF])),
            Some(PToken::Data(DataType::Word, vec![0xEA, 0xEB])),
            None,
        ];

        // Assert
        for e in expected.iter() {
            let t = &parser.next_token();
            println!("expected {:?}, parsed {:?} ", e, t);
            assert_eq!(e, t);
        }
    }

    #[test]
    fn test_fencei_line() {
        let input = "la: 2: fence.i x0";
        let mut parser = Parser::new(lexer::Lexer::new(input));

        let expected = vec![
            Some(PToken::Label("la".to_string(), LabelType::Global)),
            Some(PToken::Label("2".to_string(), LabelType::Local)),
            Some(PToken::Inst("FENCE.I".to_string(), vec![
                Arg::Reg(ast::Reg::X0),
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
