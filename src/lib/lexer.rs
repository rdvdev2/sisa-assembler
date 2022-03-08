use crate::span::{FileLoc, Span};
use crate::tokens::Token::*;
use crate::tokens::*;
use plex::lexer;

pub struct Lexer<'a> {
    input: &'a str,
    remaining: &'a str,
    cur_line: usize,
    cur_col: usize,
}

impl<'a> Lexer<'a> {
    lexer! {
        fn next_token(tok) -> Token;

        r";[^\n]*\n" => IGNORE,

        r"\.text" => BEGIN_TEXT,
        r"\.data" => BEGIN_DATA,
        r"\.end" => END,

        r"\.byte" => BYTE,
        r"\.word" => WORD,
        r"\.space" => SPACE,
        r"\.even" => EVEN,

        r"\.set" => SET,

        r"," => COMMA,
        r"\(" => LPAR,
        r"\)" => RPAR,
        r":" => COLON,
        r"=" => EQUALS,

        r"AND" => AND,
        r"OR" => OR,
        r"XOR" => XOR,
        r"NOT" => NOT,
        r"ADD" => ADD,
        r"SUB" => SUB,
        r"SHA" => SHA,
        r"SHL" => SHL,
        r"CMPLT" => CMPLT,
        r"CMPLE" => CMPLE,
        r"CMPEQ" => CMPEQ,
        r"CMPLTU" => CMPLTU,
        r"CMPLEU" => CMPLEU,
        r"ADDI" => ADDI,
        r"LD" => LD,
        r"ST" => ST,
        r"LDB" => LDB,
        r"STB" => STB,
        r"JALR" => JALR,
        r"BZ" => BZ,
        r"BNZ" => BNZ,
        r"MOVI" => MOVI,
        r"MOVHI" => MOVHI,
        r"IN" => IN,
        r"OUT" => OUT,

        r"R[0-7]" => parse_reg(tok),
        r"[0-9]+" => parse_int_lit(tok),
        r"[-+][0-9]+" => parse_int_lit(tok),
        r"0(x|X)[0-9a-fA-f]+" => parse_hex_lit(tok),

        r"lo" => LO,
        r"hi" => HI,

        r"[a-zA-Z][a-zA-Z0-9\_\-]*" => IDENT(tok.into()),

        r"[\n\t\s\r ]+" => IGNORE,
        r"." => INVALID(tok.into())
    }

    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            remaining: input,
            cur_line: 1,
            cur_col: 1,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, Span);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (token, remaining) = match Lexer::next_token(self.remaining) {
                Some((t, r)) => (t, r),
                _ => return None,
            };

            let start = self.input.len() - self.remaining.len();
            let end = self.input.len() - remaining.len();
            let lo = FileLoc {
                line: self.cur_line,
                col: self.cur_col,
            };

            for c in self.input.get(start..end).unwrap().chars() {
                match c {
                    '\n' => {
                        self.cur_col = 1;
                        self.cur_line += 1;
                    }
                    _ => self.cur_col += 1,
                }
            }

            if let Token::IGNORE = token {
                self.remaining = remaining;
            } else {
                self.remaining = remaining;

                if let Token::END = token {
                    self.remaining = "";
                }

                return Some((
                    token,
                    Span {
                        lo,
                        hi: FileLoc {
                            line: self.cur_line,
                            col: self.cur_col,
                        },
                    },
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    fn lex_instruction() {
        let mut lexer = Lexer::new("LDB R1, 32(R3)");

        assert_matches!(lexer.next(), Some((Token::LDB, _)));
        assert_matches!(lexer.next(), Some((Token::REG(1), _)));
        assert_matches!(lexer.next(), Some((Token::COMMA, _)));
        assert_matches!(lexer.next(), Some((Token::LIT(32), _)));
        assert_matches!(lexer.next(), Some((Token::LPAR, _)));
        assert_matches!(lexer.next(), Some((Token::REG(3), _)));
        assert_matches!(lexer.next(), Some((Token::RPAR, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_ident() {
        let mut lexer = Lexer::new("lab: (lab)");

        match lexer.next() {
            Some((Token::IDENT(name), _)) => assert_eq!(name, "lab"),
            _ => panic!(),
        }
        assert_matches!(lexer.next(), Some((Token::COLON, _)));
        assert_matches!(lexer.next(), Some((Token::LPAR, _)));
        match lexer.next() {
            Some((Token::IDENT(name), _)) => assert_eq!(name, "lab"),
            _ => panic!(),
        }
        assert_matches!(lexer.next(), Some((Token::RPAR, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_invalid() {
        let mut lexer = Lexer::new(".");

        assert_matches!(lexer.next(), Some((Token::INVALID(_), _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_whitespace() {
        let mut lexer = Lexer::new("    \n    \t   ");

        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_functions() {
        let mut lexer = Lexer::new("lo(0) hi(0)");

        assert_matches!(lexer.next(), Some((Token::LO, _)));
        assert_matches!(lexer.next(), Some((Token::LPAR, _)));
        assert_matches!(lexer.next(), Some((Token::LIT(0), _)));
        assert_matches!(lexer.next(), Some((Token::RPAR, _)));
        assert_matches!(lexer.next(), Some((Token::HI, _)));
        assert_matches!(lexer.next(), Some((Token::LPAR, _)));
        assert_matches!(lexer.next(), Some((Token::LIT(0), _)));
        assert_matches!(lexer.next(), Some((Token::RPAR, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn lex_directives() {
        let mut lexer = Lexer::new(".text .data .byte .word .space .even .end .text");

        assert_matches!(lexer.next(), Some((Token::BEGIN_TEXT, _)));
        assert_matches!(lexer.next(), Some((Token::BEGIN_DATA, _)));
        assert_matches!(lexer.next(), Some((Token::BYTE, _)));
        assert_matches!(lexer.next(), Some((Token::WORD, _)));
        assert_matches!(lexer.next(), Some((Token::SPACE, _)));
        assert_matches!(lexer.next(), Some((Token::EVEN, _)));
        assert_matches!(lexer.next(), Some((Token::END, _)));
        assert_matches!(lexer.next(), None);
    }

    #[test]
    fn span() {
        let mut lexer = Lexer::new("    \n MOVI");

        assert_matches!(
            lexer.next(),
            Some((
                _,
                Span {
                    lo: FileLoc { col: 2, line: 2 },
                    hi: FileLoc { col: 6, line: 2 }
                }
            ))
        );
        assert_matches!(lexer.next(), None);
    }
}
