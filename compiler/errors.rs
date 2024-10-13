#[derive(Debug)]
pub struct TokenError;

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenError")
    }
}

impl std::error::Error for TokenError {}

#[derive(Debug)]
pub struct ParserError;

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParserError")
    }
}

impl std::error::Error for ParserError {}

#[derive(Debug)]
pub struct BackEndError;

impl std::fmt::Display for BackEndError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BackEndError")
    }
}

impl std::error::Error for BackEndError {}

#[derive(Debug)]
pub struct ImpossibleError;

impl std::fmt::Display for ImpossibleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ImpossibleError")
    }
}

impl std::error::Error for ImpossibleError {}
