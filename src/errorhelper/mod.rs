use crate::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParserError {
    pub msg: String,
    pub token: Option<Token>,
    pub tokentype: Option<TokenType>,
}

impl ParserError {
    pub fn new(msg: &str, token: Option<&Token>, tokentype: Option<&TokenType>) -> Self {
        Self {
            msg: msg.to_string(),
            token: token.cloned(),
            tokentype: tokentype.copied(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ErrorHelper {
    pub source: String,
}

impl ErrorHelper {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_owned(),
        }
    }

    pub fn show_error(&self, token: &Token) -> String {
        let line: String =
            self.source.split('\n').collect::<Vec<&str>>()[token.lineno - 1].to_string();
        format!("{} |{}", token.lineno, line)
    }
}
