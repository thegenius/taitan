use crate::middleware::jwt::TokenParser;

pub trait SharedState {
    fn get_jwt_parser(&self) -> &TokenParser;
}

pub struct AppState {
    jwt_parser: TokenParser,
}

impl SharedState for AppState {
    fn get_jwt_parser(&self) -> &TokenParser {
        return &self.jwt_parser;
    }
}
