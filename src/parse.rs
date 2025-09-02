use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(char),
    UnescapedChar(char),
    UnexpectedEnd,
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidEscape(ch) => write!(f, "Invalid escape sequence '\\{ch}'"),
            ParseError::UnescapedChar(ch) => write!(f, "Unescaped '{ch}' character"),
            ParseError::UnexpectedEnd => {
                write!(f, "Unexpected end of message, ensure references are closed")
            }
        }
    }
}

pub enum Token {
    Text(String),
    Resource(String),
}

struct EntryParser {
    state: ParseState,
    last_state: Option<ParseState>,
}

enum ParseState {
    Text(String),
    InPath(String),
    Escape,
}

pub fn parse_message(msg: &str) -> Result<Vec<Token>, ParseError> {
    let parser = EntryParser::new();
    parser.parse(msg)
}

impl EntryParser {
    pub fn new() -> Self {
        Self {
            state: ParseState::Text(String::new()),
            last_state: None,
        }
    }

    pub fn parse(mut self, msg: &str) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();

        for ch in msg.chars() {
            if let Some(token) = self.process_char(ch)? {
                tokens.push(token);
            }
        }

        if let ParseState::Text(val) = self.state {
            if !val.is_empty() {
                tokens.push(Token::Text(val))
            }
        } else {
            return Err(ParseError::UnexpectedEnd);
        }

        Ok(tokens)
    }

    fn process_char(&mut self, ch: char) -> Result<Option<Token>, ParseError> {
        match &mut self.state {
            ParseState::Text(s) => match ch {
                '[' => {
                    let token = (!s.is_empty()).then_some(Token::Text(s.clone()));
                    self.state = ParseState::InPath(String::new());
                    return Ok(token);
                }
                '\\' => {
                    let last_state = std::mem::replace(&mut self.state, ParseState::Escape);
                    self.last_state = Some(last_state);
                }
                ']' => return Err(ParseError::UnescapedChar(ch)),
                _ => s.push(ch),
            },
            ParseState::InPath(s) => match ch {
                ']' => {
                    let token = Token::Resource(s.clone());
                    self.state = ParseState::Text(String::new());
                    return Ok(Some(token));
                }
                '\\' => {
                    let last_state = std::mem::replace(&mut self.state, ParseState::Escape);
                    self.last_state = Some(last_state);
                }
                '[' => return Err(ParseError::UnescapedChar(ch)),
                _ => s.push(ch),
            },
            ParseState::Escape => match ch {
                '\\' | '[' | ']' | crate::ENTRY_DELIMITER_CHAR => {
                    let mut last_state = self
                        .last_state
                        .take()
                        .unwrap_or_else(|| ParseState::Text(String::new()));
                    match &mut last_state {
                        ParseState::Text(s) => s.push(ch),
                        ParseState::InPath(s) => s.push(ch),
                        ParseState::Escape => panic!("Can't escape from an escape state"),
                    }

                    self.state = last_state;
                }
                _ => return Err(ParseError::InvalidEscape(ch)),
            },
        }

        Ok(None)
    }
}
