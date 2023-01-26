#[derive(Debug , Clone , PartialEq)]
pub enum Token{
   Illegal,
   Eof,
   Ident(String),
   Plus,
   Minus,
   Eq
}
