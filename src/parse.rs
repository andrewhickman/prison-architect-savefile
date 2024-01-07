use core::fmt;
use std::{error::Error, str::FromStr};

use indexmap::IndexMap;

use crate::Node;

/// An error when parsing an invalid savefile.
#[derive(Debug)]
pub struct ParseError {
    message: String,
    line: u32,
    col: u32,
}

impl FromStr for Node {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Parser { input, pos: 0 }.parse_node(false)
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_node(&mut self, child: bool) -> Result<Node, ParseError> {
        let mut node = Node {
            properties: IndexMap::new(),
            children: IndexMap::new(),
        };

        loop {
            self.skip_whitespace();
            if self.eat("BEGIN") {
                self.skip_whitespace();
                let key = self.parse_string()?;
                node.children
                    .entry(key.clone())
                    .or_default()
                    .push(self.parse_node(true)?);
                if !self.eat("END") {
                    return Err(self.error(format!("unterminated object '{}'", key)));
                }
            } else if self.eof() || (child && self.remainder().starts_with("END")) {
                return Ok(node);
            } else {
                let (key, value) = self.parse_attribute()?;
                node.properties.entry(key).or_default().push(value);
            }
        }
    }

    fn parse_attribute(&mut self) -> Result<(String, String), ParseError> {
        let key = self.parse_string()?;
        self.skip_whitespace();
        let value = self.parse_string()?;
        Ok((key, value))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        if self.remainder().starts_with('"') {
            self.parse_quoted_string()
        } else if self.eof() {
            Err(self.error("unexpected eof"))
        } else {
            self.parse_plain_string()
        }
    }

    fn parse_quoted_string(&mut self) -> Result<String, ParseError> {
        assert!(self.eat("\""));
        let mut result = String::new();

        let remainder = self.remainder();
        let mut pos = 0;
        for (start, ch) in self.remainder().match_indices(['\"', '\\']) {
            match ch {
                "\"" => {
                    result.push_str(&remainder[pos..start]);
                    self.pos += start + 1;
                    return Ok(result);
                }
                "\\" => {
                    result.push_str(&remainder[pos..start]);
                    match remainder.as_bytes().get(start + 1) {
                        Some(b'n') => {
                            result.push('\n');
                            pos = start + 2;
                        }
                        Some(b'"') => {
                            result.push('"');
                            pos = start + 2;
                        }
                        Some(_) => {
                            pos = start + 1;
                        }
                        None => return Err(self.error("incomplete escape")),
                    }
                }
                _ => unreachable!(),
            }
        }

        Err(self.error("unterminated string"))
    }

    fn parse_plain_string(&mut self) -> Result<String, ParseError> {
        match self.remainder().find(|ch: char| ch.is_ascii_whitespace()) {
            Some(end) => {
                let token = self.remainder()[..end].to_owned();
                self.pos += end;
                Ok(token)
            }
            None => Err(self.error("expected string")),
        }
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        let line = self.input[..self.pos]
            .chars()
            .filter(|&ch| ch == '\n')
            .count();
        let col = self.input[..self.pos]
            .lines()
            .last()
            .map(|line| line.len())
            .unwrap_or(0);

        ParseError {
            line: line as u32,
            col: col as u32,
            message: message.into(),
        }
    }

    fn eat(&mut self, token: &str) -> bool {
        if self.remainder().starts_with(token) {
            self.pos += token.len();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn remainder(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn eof(&self) -> bool {
        self.pos == self.input.len()
    }

    fn bump(&mut self) {
        let ch = self.peek().expect("no char");
        self.pos += ch.len_utf8();
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line + 1, self.col + 1, self.message)
    }
}

impl Error for ParseError {}
