#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Token {
    INVALID(String),
    IGNORE,
    AND,
    OR,
    XOR,
    NOT,
    ADD,
    SUB,
    SHA,
    SHL,
    CMPLT,
    CMPLE,
    CMPEQ,
    CMPLTU,
    CMPLEU,
    ADDI,
    LD,
    ST,
    LDB,
    STB,
    JALR,
    BZ,
    BNZ,
    MOVI,
    MOVHI,
    IN,
    OUT,
    REG(u8),
    LIT(u16),
    COMMA,
    LPAR,
    RPAR,
    COLON,
    IDENT(String),
    LO,
    HI,
    BEGIN_TEXT,
    BEGIN_DATA,
    END,
    BYTE,
    WORD,
    SPACE,
    EVEN,
    EQUALS,
    SET,
}

pub fn parse_reg(tok: &str) -> Token {
    Token::REG(tok.trim_start_matches("R").parse().unwrap())
}

pub fn parse_int_lit(tok: &str) -> Token {
    Token::LIT(tok.parse::<i16>().unwrap() as u16)
}

pub fn parse_hex_lit(tok: &str) -> Token {
    Token::LIT(
        u16::from_str_radix(tok.trim_start_matches("0x").trim_start_matches("0X"), 16).unwrap(),
    )
}
