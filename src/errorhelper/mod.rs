use crate::token::Token;

#[derive(Debug, Default)]
pub struct ErrorHelper {
    pub source: String,
}

impl ErrorHelper {
    pub fn new(source: &String) -> Self {
        Self {
            source: source.clone(),
        }
    }

    pub fn show_error(&self, token: &Token) -> String {
        let line: String =
            self.source.split("\n").collect::<Vec<&str>>()[token.lineno - 1].to_string();
        format!("{} |{}", token.lineno, line)
    }
}
