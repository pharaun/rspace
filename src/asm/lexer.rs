#[derive(Debug, PartialEq)]
enum Token <'input> {
    Str(&'input str),
    Num(u32), // Only decimals or hex
    Colon,
}

// TODO: add data/memory labels
#[derive(Debug, PartialEq)]
enum LabelType { Global, Local }

#[derive(Debug, PartialEq)]
enum Arg <'input> {
    Num(u32),
    Label(&'input str, LabelType),
    Str(&'input str), // For now - Reg or CSR
}

#[derive(Debug, PartialEq)]
enum Ast <'input> {
    Label(&'input str, LabelType),
    Inst(&'input str, Vec<Arg <'input>>),
}



fn str_lex(input: &str) -> Vec<Token> {
    let mut ret: Vec<Token> = Vec::new();


    ret
}



//pub fn parse_asm(input: &str) -> Vec<u32> {
//    // First pass -> Vec<(u32, or entry to retrify on 2nd pass (for labels))>
//    let mut first_pass: Vec<ast::AsmLine> = Vec::new();
//
//    // This symbol table will be a list of (label, location)
//    // Will handle duplicate entries by just listing it
//    let mut position: usize = 0; // Per u32 word
//    let mut label_acc: Vec<ast::Labels> = Vec::new();
//    let mut symbol_table: Vec<(ast::Labels, usize)> = Vec::new();
//
//    // Assembly output
//    // Second pass -> Vec<u32>
//    let mut second_pass: Vec<u32> = Vec::new();
//
//    for line in input.lines() {
//        let line = line.trim();
//        let line = match line.find(r#"//"#) {
//            Some(x) => &line[..x],
//            None => line,
//        };
//
//        if !line.is_empty() {
//            // 2. parse it via lalrpop (parse_AsmLine)
//            let parse = parse::parse_AsmLine(line);
//
//            }
//        }
//    }
//}


#[cfg(test)]
pub mod lexer_token {
    use super::*;

    #[test]
    fn test_line() {
        let test = "la: addi x0 fp 1 -1 0xAF 2f asdf";
        let neg: i32 = -1;
        let res = vec![
            Token::Str("la"),
            Token::Colon,
            Token::Str("addi"),
            Token::Str("x0"),
            Token::Str("fp"),
            Token::Num(1),
            Token::Num(neg as u32),
            Token::Num(0xAF),
            Token::Str("2f"),
            Token::Str("asdf"),
        ];

        assert_eq!(res, str_lex(test));
    }

    fn test_multiline() {
        let test = r#"
la:
    addi x1 x1 1
lb: addi x2 x2 2
    addi x3 x3 3
        "#;

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
