#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    // Keywords (Myanglish)
    Kain,   // kain (int)
    Sar,    // sar (string)
    Sit,    // sit (bool)
    DaTha,  // da_tha (float)
    Hman,   // hman (true)
    Hmar,   // hmar (false)
    Bhala,  // bhala (nil/null)
    Hlyin,  // hlyin (if)
    Mo,     // mo (else)
    Pat,    // pat (while)
    Htae,   // htae (for-in)
    Loke,   // loke (function)
    Pyan,   // pyan (return)
    Pya,    // pya (print)
    Phat,   // phat (read)
    Su,     // su (array)
    Yu,     // yu (import)
    Twe,    // twe (hashmap)
    Pone,   // pone (struct)
    Nee,    // nee (method)
    Myat,   // myat (interface)
    Amhar,  // amhar (error type)

    // Identifiers and Literals
    Identifier(String),
    Number(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // Symbols and Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Assign,        // =
    Equals,        // ==
    NotEquals,     // !=
    GreaterThan,   // >
    LessThan,      // <
    GreaterEquals, // >=
    LessEquals,    // <=
    Dot,           // .

    // Punctuation
    LParen,        // (
    RParen,        // )
    LBrace,        // {
    RBrace,        // }
    LBracket,      // [
    RBracket,      // ]
    Comma,         // ,
    Semicolon,     // ;
    Colon,         // :
    Arrow,         // ->

    Comment(String), // // comment text

    Eof,           // End of File
    Illegal,       // Unrecognized Character
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}
