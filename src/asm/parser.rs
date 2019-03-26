use std::iter::Peekable;
use asm::lexer;

// Use the reg and csr func here for now
use asm::ast;
use std::str::FromStr;

// TODO: identify data/mem label
// TODO: parse macro definition and usage here

// TODO: add data/memory labels
// TODO: better way to do labels? (def duplicated)
#[derive(Debug, PartialEq, Clone)]
pub enum LabelType { Global, Local }

#[derive(Debug, PartialEq)]
pub enum Arg {
    Num(u32),
    Label(String, LabelType),
    Reg(ast::Reg),
    Csr(ast::Csr),
}

#[derive(Debug, PartialEq)]
pub enum PToken {
    Label(String, LabelType),
    // TODO:: upper case the instructions
    Inst(String, Vec<Arg>),
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
                    lexer::Token::Newline => panic!("Should not see a newline"),
                    lexer::Token::Label(_) => panic!("Should not see a local label outside of a instruction"),
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
                                        args.push(Arg::Label(s, LabelType::Global));
                                    }
                                },
                                lexer::Token::Num(i) => args.push(Arg::Num(i)),
                                lexer::Token::Label(s) => args.push(Arg::Label(s, LabelType::Local)),
                                _ => panic!("Shouldn't see Colon or Newline here"),
                            }
                        }

                        // Should be reading tokens .... till some limit (new line, or eof?)
                        // Should ? do some sort of basic instruction validation here possibly (ie
                        // args count)
                        // Later on would need (ie in the assemblier stage) code to handle labels
                        // vs numbers/etc when one or the other is expected (particularly for
                        // addresses/variables)
                        Some(PToken::Inst(s, args))
                    },
                    // We skip newline here its only required when handling PToken::Inst
                    lexer::Token::Newline => self.next_token(),
                    lexer::Token::Num(_) => panic!("Should not see a number outside of a instruction"),
                    lexer::Token::Colon => panic!("Should not see a colon outside of a label"),
                    lexer::Token::Label(_) => panic!("Should not see a local label outside of a instruction"),
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
        let input = "la: 2: addi x0 fp 1 -1 0xAF 2f asdf CYCLE // Comments";
        let mut parser = Parser::new(lexer::Lexer::new(input));

        let neg: i32 = -1;
        let expected = vec![
            Some(PToken::Label("la".to_string(), LabelType::Global)),
            Some(PToken::Label("2".to_string(), LabelType::Local)),
            Some(PToken::Inst("addi".to_string(), vec![
                Arg::Reg(ast::Reg::X0),
                Arg::Reg(ast::Reg::X8),
                Arg::Num(1),
                Arg::Num(neg as u32),
                Arg::Num(0xAF),
                // Should have more data here
                Arg::Label("2f".to_string(), LabelType::Local),
                Arg::Label("asdf".to_string(), LabelType::Global),
                Arg::Csr(ast::Csr::CYCLE),
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

    #[test]
    fn test_fencei_line() {
        let input = "la: 2: fence.i x0";
        let mut parser = Parser::new(lexer::Lexer::new(input));

        let expected = vec![
            Some(PToken::Label("la".to_string(), LabelType::Global)),
            Some(PToken::Label("2".to_string(), LabelType::Local)),
            Some(PToken::Inst("fence.i".to_string(), vec![
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
