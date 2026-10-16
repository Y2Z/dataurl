//! Minimal JSON reader for the Web Platform Tests fixtures, kept dependency-free on purpose.
//! It panics on malformed input, which is fine for test data we control.

#![allow(dead_code)]

pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn as_array(&self) -> &[Json] {
        match self {
            Json::Array(items) => items,
            _ => panic!("expected a JSON array"),
        }
    }

    pub fn as_str(&self) -> &str {
        self.as_opt_str().expect("expected a JSON string")
    }

    pub fn as_opt_str(&self) -> Option<&str> {
        match self {
            Json::String(s) => Some(s),
            Json::Null => None,
            _ => panic!("expected a JSON string or null"),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.as_array()
            .iter()
            .map(|item| match item {
                Json::Number(n) if (0.0..=255.0).contains(n) => *n as u8,
                _ => panic!("expected an array of bytes"),
            })
            .collect()
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Json::Null)
    }

    pub fn get_opt(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => panic!("expected a JSON object"),
        }
    }

    pub fn get(&self, key: &str) -> &Json {
        self.get_opt(key)
            .unwrap_or_else(|| panic!("missing key {key:?}"))
    }
}

pub fn parse(input: &str) -> Json {
    let mut parser = Parser {
        chars: input.chars().collect(),
        pos: 0,
    };
    let value = parser.value();
    parser.skip_whitespace();
    assert_eq!(
        parser.pos,
        parser.chars.len(),
        "trailing data after JSON value"
    );
    value
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn next(&mut self) -> char {
        let c = *self.chars.get(self.pos).expect("unexpected end of JSON");
        self.pos += 1;
        c
    }

    fn expect_literal(&mut self, literal: &str) {
        for expected in literal.chars() {
            assert_eq!(self.next(), expected, "invalid JSON literal");
        }
    }

    fn value(&mut self) -> Json {
        self.skip_whitespace();
        match self.chars[self.pos] {
            'n' => {
                self.expect_literal("null");
                Json::Null
            }
            't' => {
                self.expect_literal("true");
                Json::Bool(true)
            }
            'f' => {
                self.expect_literal("false");
                Json::Bool(false)
            }
            '"' => Json::String(self.string()),
            '[' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_whitespace();
                    if self.chars[self.pos] == ']' {
                        self.pos += 1;
                        break;
                    }
                    items.push(self.value());
                    self.skip_whitespace();
                    if self.next() == ']' {
                        break;
                    }
                }
                Json::Array(items)
            }
            '{' => {
                self.pos += 1;
                let mut members = Vec::new();
                loop {
                    self.skip_whitespace();
                    if self.chars[self.pos] == '}' {
                        self.pos += 1;
                        break;
                    }
                    let key = self.string();
                    self.skip_whitespace();
                    assert_eq!(self.next(), ':');
                    members.push((key, self.value()));
                    self.skip_whitespace();
                    if self.next() == '}' {
                        break;
                    }
                }
                Json::Object(members)
            }
            _ => {
                let start = self.pos;
                while self.pos < self.chars.len()
                    && matches!(
                        self.chars[self.pos],
                        '-' | '+' | '.' | 'e' | 'E' | '0'..='9'
                    )
                {
                    self.pos += 1;
                }
                let number: String = self.chars[start..self.pos].iter().collect();
                Json::Number(number.parse().expect("invalid JSON number"))
            }
        }
    }

    fn hex4(&mut self) -> u32 {
        let digits: String = (0..4).map(|_| self.next()).collect();
        u32::from_str_radix(&digits, 16).expect("invalid \\u escape")
    }

    fn string(&mut self) -> String {
        assert_eq!(self.next(), '"');
        let mut s = String::new();
        loop {
            match self.next() {
                '"' => break,
                '\\' => match self.next() {
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'r' => s.push('\r'),
                    'b' => s.push('\u{8}'),
                    'f' => s.push('\u{c}'),
                    'u' => {
                        let mut code = self.hex4();
                        if (0xD800..0xDC00).contains(&code) {
                            // Surrogate pair
                            assert_eq!(self.next(), '\\');
                            assert_eq!(self.next(), 'u');
                            let low = self.hex4();
                            code = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                        }
                        s.push(char::from_u32(code).expect("invalid code point"));
                    }
                    c => s.push(c), // \" \\ \/
                },
                c => s.push(c),
            }
        }
        s
    }
}
