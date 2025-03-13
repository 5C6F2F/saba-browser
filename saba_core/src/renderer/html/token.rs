use crate::renderer::html::attribute::Attribute;
use alloc::{string::String, vec::Vec};

pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    re_consume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            re_consume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    fn re_consume_input(&mut self) -> char {
        self.re_consume = false;
        self.input[self.pos - 1]
    }

    fn consume_next_input(&mut self) -> char {
        let ch = self.input[self.pos];
        self.pos += 1;
        ch
    }

    fn is_eof(&self) -> bool {
        self.pos > self.input.len()
    }

    fn create_start_tag(&mut self) {
        self.latest_token = Some(HtmlToken::StartTag {
            tag: String::new(),
            self_closing: false,
            attributes: Vec::new(),
        });
    }

    fn create_end_tag(&mut self) {
        self.latest_token = Some(HtmlToken::EndTag { tag: String::new() })
    }

    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let token = self.latest_token.clone();
        self.latest_token = None;

        token
    }

    fn append_tag_name(&mut self, ch: char) {
        match &mut self.latest_token {
            Some(HtmlToken::StartTag {
                tag,
                self_closing: _,
                attributes: _,
            })
            | Some(HtmlToken::EndTag { tag }) => tag.push(ch),
            _ => panic!("`latest_token` should be either StartTag or EndTag"),
        }
    }

    fn start_new_attribute(&mut self) {
        match &mut self.latest_token {
            Some(HtmlToken::StartTag {
                tag: _,
                self_closing: _,
                attributes,
            }) => attributes.push(Attribute::new()),
            _ => panic!("`latest_token` should be either StartTag"),
        }
    }

    fn append_attribute_name(&mut self, ch: char) {
        match &mut self.latest_token {
            Some(HtmlToken::StartTag {
                tag: _,
                self_closing: _,
                attributes,
            }) => {
                let length = attributes.len();
                attributes[length - 1].add_name_char(ch)
            }
            _ => panic!("`latest_token` should be either StartTag"),
        }
    }

    fn append_attribute_value(&mut self, ch: char) {
        match &mut self.latest_token {
            Some(HtmlToken::StartTag {
                tag: _,
                self_closing: _,
                attributes,
            }) => {
                let length = attributes.len();
                attributes[length - 1].add_value_char(ch)
            }
            _ => panic!("`latest_token` should be either StartTag"),
        }
    }

    fn set_self_closing_flag(&mut self) {
        match &mut self.latest_token {
            Some(HtmlToken::StartTag {
                tag: _,
                self_closing,
                attributes: _,
            }) => *self_closing = true,
            _ => panic!("`latest_toke` should be either StartTag"),
        }
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        loop {
            let ch = match self.re_consume {
                true => self.re_consume_input(),
                false => self.consume_next_input(),
            };

            match self.state {
                State::Data => {
                    if ch == '<' {
                        self.state = State::TagOpen;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(ch));
                }
                State::TagOpen => {
                    if ch == '/' {
                        self.state = State::EndTagOpen;
                        continue;
                    }

                    if ch.is_ascii_alphabetic() {
                        self.re_consume = true;
                        self.state = State::TagName;
                        self.create_start_tag();
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.re_consume = true;
                    self.state = State::Data;
                }
                State::EndTagOpen => {
                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    if ch.is_ascii_alphabetic() {
                        self.re_consume = true;
                        self.state = State::TagName;
                        self.create_end_tag();
                        continue;
                    }
                }
                State::TagName => {
                    if ch == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if ch == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if ch == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if ch.is_ascii_uppercase() {
                        self.append_tag_name(ch.to_ascii_lowercase());
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_tag_name(ch);
                }
                State::BeforeAttributeName => {
                    if ch == '/' || ch == '>' || self.is_eof() {
                        self.re_consume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    self.re_consume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                State::AttributeName => {
                    if ch == ' ' || ch == '/' || ch == '>' || self.is_eof() {
                        self.re_consume = true;
                        self.state = State::AfterAttributeName;
                        continue;
                    }

                    if ch == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if ch.is_ascii_uppercase() {
                        self.append_attribute_name(ch.to_ascii_lowercase());
                        continue;
                    }

                    self.append_attribute_name(ch);
                }
                State::AfterAttributeName => {
                    if ch == ' ' {
                        continue;
                    }

                    if ch == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if ch == '=' {
                        self.state = State::BeforeAttributeValue;
                        continue;
                    }

                    if ch == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.re_consume = true;
                    self.state = State::AttributeName;
                    self.start_new_attribute();
                }
                State::BeforeAttributeValue => {
                    if ch == ' ' {
                        continue;
                    }

                    if ch == '"' {
                        self.state = State::AttributeValueDoubleQuoted;
                        continue;
                    }

                    if ch == '\'' {
                        self.state = State::AttributeValueSingleQuoted;
                        continue;
                    }

                    self.re_consume = true;
                    self.state = State::AttributeValueUnquoted;
                }
                State::AttributeValueDoubleQuoted => {
                    if ch == '"' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute_value(ch);
                }
                State::AttributeValueSingleQuoted => {
                    if ch == '\'' {
                        self.state = State::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute_value(ch);
                }
                State::AttributeValueUnquoted => {
                    if ch == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if ch == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_attribute_value(ch);
                }
                State::AfterAttributeValueQuoted => {
                    if ch == ' ' {
                        self.state = State::BeforeAttributeName;
                        continue;
                    }

                    if ch == '/' {
                        self.state = State::SelfClosingStartTag;
                        continue;
                    }

                    if ch == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.re_consume = true;
                    self.state = State::BeforeAttributeValue;
                }
                State::SelfClosingStartTag => {
                    if ch == '>' {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                }
                State::ScriptData => {
                    if ch == '<' {
                        self.state = State::ScriptDataLessThanSign;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(ch));
                }
                State::ScriptDataLessThanSign => {
                    if ch == '/' {
                        self.buf = String::new();
                        self.state = State::ScriptDataEndTagOpen;
                        continue;
                    }

                    self.re_consume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }
                State::ScriptDataEndTagOpen => {
                    if ch.is_ascii_alphabetic() {
                        self.re_consume = true;
                        self.state = State::ScriptDataEndTagName;
                        self.create_end_tag();
                        continue;
                    }

                    self.re_consume = true;
                    self.state = State::ScriptData;
                    return Some(HtmlToken::Char('<'));
                }
                State::ScriptDataEndTagName => {
                    if ch == '>' {
                        self.state = State::Data;
                        return self.take_latest_token();
                    }

                    if ch.is_ascii_alphabetic() {
                        self.buf.push(ch);
                        self.append_tag_name(ch.to_ascii_lowercase());
                        continue;
                    }

                    self.state = State::TemporaryBuffer;
                    self.buf = String::from("</") + &self.buf;
                    self.buf.push(ch);
                    continue;
                }
                State::TemporaryBuffer => {
                    self.re_consume = true;
                    if self.buf.is_empty() {
                        self.state = State::ScriptData;
                        continue;
                    }

                    let ch = self
                        .buf
                        .chars()
                        .nth(0)
                        .expect("buffer should have at least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(ch));
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HtmlToken {
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    EndTag {
        tag: String,
    },
    Char(char),
    Eof,
}

pub enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    ScriptData,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    TemporaryBuffer,
}

#[cfg(test)]
mod tests {
    use super::{HtmlToken, HtmlTokenizer};
    use crate::renderer::html::attribute::Attribute;
    use alloc::{
        string::{String, ToString},
        vec,
        vec::Vec,
    };

    #[test]
    fn test_empty() {
        let html = String::new();
        let mut tokenizer = HtmlTokenizer::new(html);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn test_start_and_end_tag() {
        let html = "<body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::EndTag {
                tag: "body".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_attributes() {
        let html = "<p class=\"A\" id='B' foo=bar></p>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let mut attr1 = Attribute::new();
        attr1.add_name_char('c');
        attr1.add_name_char('l');
        attr1.add_name_char('a');
        attr1.add_name_char('s');
        attr1.add_name_char('s');
        attr1.add_value_char('A');

        let mut attr2 = Attribute::new();
        attr2.add_name_char('i');
        attr2.add_name_char('d');
        attr2.add_value_char('B');

        let mut attr3 = Attribute::new();
        attr3.add_name_char('f');
        attr3.add_name_char('o');
        attr3.add_name_char('o');
        attr3.add_value_char('b');
        attr3.add_value_char('a');
        attr3.add_value_char('r');

        let expected = [
            HtmlToken::StartTag {
                tag: "p".to_string(),
                self_closing: false,
                attributes: vec![attr1, attr2, attr3],
            },
            HtmlToken::EndTag {
                tag: "p".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let html = "<img />".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [HtmlToken::StartTag {
            tag: "img".to_string(),
            self_closing: true,
            attributes: Vec::new(),
        }];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_script_tag() {
        let html = "<script>js code;</script>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "script".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::Char('j'),
            HtmlToken::Char('s'),
            HtmlToken::Char(' '),
            HtmlToken::Char('c'),
            HtmlToken::Char('o'),
            HtmlToken::Char('d'),
            HtmlToken::Char('e'),
            HtmlToken::Char(';'),
            HtmlToken::EndTag {
                tag: "script".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }
}
