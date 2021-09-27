use std::{iter::Peekable, str::Chars};
use uuid::Uuid;

use crate::Version;

pub struct ParsedJson {
    template: String,
    key: String,
    pub version: Option<Version>,
}
impl ParsedJson {
    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }

    pub fn get_version(&self) -> &Version {
        self.version.as_ref().unwrap()
    }

    pub fn get_version_mut(&mut self) -> &mut Version {
        self.version.as_mut().unwrap()
    }

    pub fn emb_string(&self) -> String {
        self.template.replace(&self.key, &self.get_version().to_string())
    }
}

pub fn parse_json(json: impl Into<String>) -> Result<ParsedJson, String> {
    Ok(Parser::parse(json))
}

pub struct Parser<'a> {
    json: Peekable<Chars<'a>>,
    parsed_json: ParsedJson,
    nested: usize,
}
impl<'a> Parser<'a> {
    pub fn parse(json: impl Into<String>) -> ParsedJson {
        let json: String = json.into();
        let mut parser = Parser::new(json.chars());
        let template = parser.content();
        parser.parsed_json.template = template;
        parser.parsed_json
    }

    pub fn new(json: Chars<'a>) -> Self {
        Self {
            json: json.peekable(),
            parsed_json: ParsedJson {
                template: String::new(),
                key: Uuid::new_v4().to_string(),
                version: None,
            },
            nested: 0,
        }
    }

    fn content(&mut self) -> String {
        let mut content = String::new();
        while let Some(c) = self.json.next() {
            content.push(c);
            content += match c {
                '{' => self.object(),
                '[' => self.array(),
                '"' => self.string(),
                '0'..='9' | 'a'..='z' | 'E' | '.' | '-' | '+' => self.num_like(),
                ' ' => continue,
                _ => panic!("unexpected char: {}", c),
            }.as_str();
            break;
        }
        content
    }

    fn object(&mut self) -> String {
        let mut object = String::new();
        self.nested += 1;
        if let Some(&c) = self.json.peek() {
            if c == '}' {
                self.json.next();
                object.push(c);
                self.nested -= 1;
                return object;
            }
        } else {
            panic!("unexpected end of json");
        }
        while let Some(&c) = self.json.peek() {
            if is_whitespace(c) {
                self.json.next();
                object.push(c);
                continue;
            }
            break;
        }
        object += self.object_body().as_str();
        loop {
            if let Some(&c) = self.json.peek() {
                match c {
                    '}' => {
                        self.json.next();
                        object.push(c);
                        break;
                    },
                    ',' => {
                        self.json.next();
                        object.push(c);
                        object += self.object_body().as_str();
                    },
                    x if is_whitespace(x) => {
                        self.json.next();
                        object.push(c);
                        continue;
                    },
                    _ => panic!("unexpected char: {}", c),
                }
            } else {
                panic!("unexpected end of json");
            }
        }
        self.nested -= 1;
        object
    }

    fn object_body(&mut self) -> String {
        let mut object_body = String::new();
        let mut key = String::new();

        while let Some(c) = self.json.next() {
            object_body.push(c);
            if is_whitespace(c) {
                continue;
            }
            if c == '"' {
                key = self.string();
                object_body += key.as_str();
                key = key.trim_end_matches("\"").to_string();
                break;
            }
            panic!("unexpected char: {}", c);
        }
        while let Some(c) = self.json.next() {
            object_body.push(c);
            if is_whitespace(c) {
                continue;
            }
            if c == ':' {
                break;
            }
            panic!("unexpected char: {}", c);
        }
        while let Some(&c) = self.json.peek() {
            if is_whitespace(c) {
                self.json.next();
                object_body.push(c);
                continue;
            } else {
                break;
            }
        }
        let value = self.content();
        if self.nested == 1 && key == "version" {
            if let Ok(version) = Version::from_str(value.trim_matches('\"')) {
                if self.parsed_json.version.is_some() {
                    panic!("duplicate version");
                }
                self.parsed_json.version = Some(version);
                let key_with_quote = format!("\"{}\"", self.parsed_json.key);
                object_body.push_str(&key_with_quote);
            } else {
                panic!("invalid version: {}", value);
            }
        } else {
            object_body += value.as_str();
        }
        object_body
    }

    fn array(&mut self) -> String {
        let mut array = String::new();
        self.nested += 1;
        loop {
            if let Some(&c) = self.json.peek() {
                match c {
                    ']' => {
                        self.json.next();
                        array.push(']');
                        break;
                    },
                    ',' => {
                        self.json.next();
                        array.push(c);
                    },
                    x if is_whitespace(x) => {
                        self.json.next();
                        array.push(x);
                        continue;
                    },
                    _ => (),
                }
            } else {
                panic!("unexpected end of json");
            }
            array += self.content().as_str();
        }
        self.nested -= 1;
        array
    }

    fn string(&mut self) -> String {
        let mut string = String::new();
        let mut is_escaped = false;
        while let Some(c) = self.json.next() {
            string.push(c);
            match c {
                '"' if !is_escaped => break,
                '\\' if !is_escaped => is_escaped = true,
                _ if is_escaped => is_escaped = false,
                _ => continue,
            }
        }
        string
    }

    fn num_like(&mut self) -> String {
        let mut num_like = String::new();
        while let Some(&c) = self.json.peek() {
            match c {
                '0'..='9' | 'a'..='z' | 'E' | '.' | '-' | '+' => {
                    self.json.next();
                    num_like.push(c);
                    continue
                },
                ']' | ',' | '}' => break,
                x if is_whitespace(x) => break,
                _ => panic!("unexpected char: {}", c),
            }
        }
        num_like
    }
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_simple() {
        let mut parser = Parser::new(r#""hello""#.chars());
        assert_eq!(r#""hello""#.to_string(), parser.content());
    }

    #[test]
    fn string_include_double_quote() {
        let mut parser = Parser::new(r#""hello\"""#.chars());
        assert_eq!(r#""hello\"""#.to_string(), parser.content());
    }

    #[test]
    fn string_include_backslash() {
        let mut parser = Parser::new(r#""hello\\""#.chars());
        assert_eq!(r#""hello\\""#.to_string(), parser.content());
    }

    #[test]
    fn string_include_backslash_and_double_quote() {
        let mut parser = Parser::new(r#""hello\\\"""#.chars());
        assert_eq!(r#""hello\\\"""#.to_string(), parser.content());
    }

    #[test]
    fn num_like_simple() {
        let mut parser = Parser::new(r#"123"#.chars());
        assert_eq!(r#"123"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_simple_with_dot() {
        let mut parser = Parser::new(r#"123.456"#.chars());
        assert_eq!(r#"123.456"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_simple_with_dot_and_e() {
        let mut parser = Parser::new(r#"123.456e7"#.chars());
        assert_eq!(r#"123.456e7"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_simple_with_dot_and_e_and_minus() {
        let mut parser = Parser::new(r#"123.456e-7"#.chars());
        assert_eq!(r#"123.456e-7"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_simple_with_dot_and_e_and_minus_and_plus() {
        let mut parser = Parser::new(r#"123.456e-7+"#.chars());
        assert_eq!(r#"123.456e-7+"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_true() {
        let mut parser = Parser::new(r#"true"#.chars());
        assert_eq!(r#"true"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_false() {
        let mut parser = Parser::new(r#"false"#.chars());
        assert_eq!(r#"false"#.to_string(), parser.content());
    }

    #[test]
    fn num_like_null() {
        let mut parser = Parser::new(r#"null"#.chars());
        assert_eq!(r#"null"#.to_string(), parser.content());
    }

    #[test]
    #[should_panic(expected = "unexpected char: \"")]
    fn num_like_fail_with_double_quote() {
        let mut parser = Parser::new(r#"tr"ue"#.chars());
        parser.content();
    }

    #[test]
    #[should_panic(expected = "unexpected char: [")]
    fn num_like_fail_with_left_bracket() {
        let mut parser = Parser::new(r#"tr[ue"#.chars());
        parser.content();
    }

    #[test]
    fn array_simple() {
        let mut parser = Parser::new(r#"[1,"x",null]"#.chars());
        assert_eq!(r#"[1,"x",null]"#.to_string(), parser.content());
    }

    #[test]
    fn array_with_space() {
        let mut parser = Parser::new(r#"[1, "x", null]"#.chars());
        assert_eq!(r#"[1, "x", null]"#.to_string(), parser.content());
    }

    #[test]
    fn array_nested() {
        let mut parser = Parser::new(r#"[1,[2,3],null]"#.chars());
        assert_eq!(r#"[1,[2,3],null]"#.to_string(), parser.content());
    }

    #[test]
    fn object_simple() {
        let mut parser = Parser::new(r#"{"a":1}"#.chars());
        assert_eq!(r#"{"a":1}"#.to_string(), parser.content());
    }

    #[test]
    fn object_simple_with_space() {
        let mut parser = Parser::new(r#"{"a": 1}"#.chars());
        assert_eq!(r#"{"a": 1}"#.to_string(), parser.content());
    }

    #[test]
    fn object_simple_with_many_space() {
        let mut parser = Parser::new(r#"{
    "a" : 1 }"#.chars());
        assert_eq!(r#"{
    "a" : 1 }"#.to_string(), parser.content());
    }

    #[test]
    fn object_nested() {
        let mut parser = Parser::new(r#"{"a":{"b":1}}"#.chars());
        assert_eq!(r#"{"a":{"b":1}}"#.to_string(), parser.content());
    }

    #[test]
    fn object_nested_with_space() {
        let mut parser = Parser::new(r#"{
    "a" : {
        "b" : 1
    }
}"#.chars());
        assert_eq!(r#"{
    "a" : {
        "b" : 1
    }
}"#.to_string(), parser.content());
    }

    #[test]
    fn object_with_comma() {
        let mut parser = Parser::new(r#"{"a":1, "b":2}"#.chars());
        assert_eq!(r#"{"a":1, "b":2}"#.to_string(), parser.content());
    }

    #[test]
    fn object_include_version() {
        let parsed_json = parse_json(r#"{"a":1,"version":"0.1.0"}"#).unwrap();
        assert_eq!(parsed_json.version.unwrap().to_string(), "0.1.0".to_string());
        assert_eq!(parsed_json.template, format!("{{\"a\":1,\"version\":\"{}\"}}", parsed_json.key));
    }

    #[test]
    #[should_panic(expected = "invalid version: \"1.0\"")]
    #[allow(unused_must_use)]
    fn object_include_invalid_version() {
        parse_json(r#"{"a":1,"version":"1.0"}"#);
    }

    #[test]
    fn object_include_behind_version() {
        let parsed_json = parse_json(r#"{"a":{"b":["x",null,1]},"version":"0.1.0"}"#).unwrap();
        assert_eq!(parsed_json.version.unwrap().to_string(), "0.1.0".to_string());
        assert_eq!(parsed_json.template, format!("{{\"a\":{{\"b\":[\"x\",null,1]}},\"version\":\"{}\"}}", parsed_json.key));
    }

    #[test]
    fn object_include_version_with_space() {
        let parsed_json = parse_json(r#" {  "a" : 1  ,   "version"  :  "0.1.0"  }"#).unwrap();
        assert_eq!(parsed_json.version.unwrap().to_string(), "0.1.0".to_string());
        assert_eq!(parsed_json.template, format!(" {{  \"a\" : 1  ,   \"version\"  :  \"{}\"  }}", parsed_json.key));
    }

    #[test]
    fn object_include_nested_version() {
        let parsed_json = parse_json(r#"{"a":1,"b":{"version":"0.1.0"}}"#).unwrap();
        assert!(parsed_json.version.is_none());
        assert_eq!(parsed_json.template, "{\"a\":1,\"b\":{\"version\":\"0.1.0\"}}".to_string());
    }

    #[test]
    fn object_include_version_and_nested_version() {
        let parsed_json = parse_json(r#"{"a":1,"version":"0.1.0","b":{"version":"0.2.0"}}"#).unwrap();
        assert_eq!(parsed_json.version.unwrap().to_string(), "0.1.0".to_string());
        assert_eq!(parsed_json.template, format!("{{\"a\":1,\"version\":\"{}\",\"b\":{{\"version\":\"0.2.0\"}}}}", parsed_json.key));
    }
}
