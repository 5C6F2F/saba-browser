use super::{
    attribute::Attribute,
    token::{HtmlToken, HtmlTokenizer},
};
use crate::renderer::dom::node::{Element, ElementKind, Node, NodeKind, Window};
use alloc::{rc::Rc, string::String, vec::Vec};
use core::{cell::RefCell, str::FromStr};

pub struct HtmlParser {
    window: Rc<RefCell<Window>>,
    mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    tokenizer: HtmlTokenizer,
}

impl HtmlParser {
    pub fn new(tokenizer: HtmlTokenizer) -> Self {
        Self {
            window: Rc::new(RefCell::new(Window::new())),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            tokenizer,
        }
    }

    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut token = self.tokenizer.next();

        while let Some(token_ref) = &token {
            match self.mode {
                InsertionMode::Initial => {
                    if let HtmlToken::Char(_) = token_ref {
                        token = self.tokenizer.next();
                        continue;
                    }

                    self.mode = InsertionMode::BeforeHtml;
                    continue;
                }
                InsertionMode::BeforeHtml => {
                    match token_ref {
                        HtmlToken::StartTag {
                            tag,
                            self_closing: _,
                            attributes,
                        } => {
                            if tag == "html" {
                                self.insert_element(tag, attributes.clone());
                                self.mode = InsertionMode::BeforeHead;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            if ch == ' ' || ch == '\n' {
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.insert_element("html", Vec::new());
                    self.mode = InsertionMode::BeforeHead;
                    continue;
                }
                InsertionMode::BeforeHead => {
                    match token_ref {
                        HtmlToken::StartTag {
                            tag,
                            self_closing: _,
                            attributes,
                        } => {
                            if tag == "head" {
                                self.insert_element(tag, attributes.clone());
                                self.mode = InsertionMode::InHead;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            if ch == ' ' || ch == '\n' {
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    // ' ', '\n', headタグ, EOF以外の場合headタグを追加してInHeadへ移行
                    self.insert_element("head", Vec::new());
                    self.mode = InsertionMode::InHead;
                    continue;
                }
                InsertionMode::InHead => {
                    match token_ref {
                        HtmlToken::StartTag {
                            tag,
                            self_closing: _,
                            attributes,
                        } => {
                            if tag == "style" || tag == "script" {
                                self.insert_element(tag, attributes.clone());
                                self.original_insertion_mode = self.mode;
                                self.mode = InsertionMode::Text;
                                token = self.tokenizer.next();
                                continue;
                            }

                            if tag == "body" {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }

                            if ElementKind::from_str(tag).is_ok() {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                continue;
                            }
                        }
                        HtmlToken::EndTag { tag } => {
                            if tag == "head" {
                                self.pop_until(ElementKind::Head);
                                self.mode = InsertionMode::AfterHead;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            if ch == ' ' || ch == '\n' {
                                self.insert_char(ch);
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                    }

                    // サポート外のタグを無視
                    token = self.tokenizer.next();
                    continue;
                }
                InsertionMode::AfterHead => {
                    match token_ref {
                        HtmlToken::StartTag {
                            tag,
                            self_closing: _,
                            attributes,
                        } => {
                            if tag == "body" {
                                self.insert_element(tag, attributes.clone());
                                self.mode = InsertionMode::InBody;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            if ch == ' ' || ch == '\n' {
                                self.insert_char(ch);
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.insert_element("body", Vec::new());
                    self.mode = InsertionMode::InBody;
                    continue;
                }
                InsertionMode::InBody => {
                    match token_ref {
                        HtmlToken::StartTag {
                            tag,
                            self_closing: _,
                            attributes,
                        } => match tag.as_str() {
                            "p" => {
                                self.insert_element(tag, attributes.clone());
                                token = self.tokenizer.next();
                                continue;
                            }
                            "h1" | "h2" | "h3" => {
                                self.insert_element(tag, attributes.clone());
                                token = self.tokenizer.next();
                                continue;
                            }
                            "a" => {
                                self.insert_element(tag, attributes.clone());
                                token = self.tokenizer.next();
                            }
                            _ => {
                                token = self.tokenizer.next();
                            }
                        },
                        HtmlToken::EndTag { tag } => {
                            match tag.as_str() {
                                "body" => {
                                    self.mode = InsertionMode::AfterBody;
                                    token = self.tokenizer.next();

                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // パース失敗のためトークンを無視
                                        continue;
                                    }

                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html));
                                    } else {
                                        token = self.tokenizer.next();
                                    }
                                    continue;
                                }
                                "p" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    self.pop_until(element_kind);
                                    token = self.tokenizer.next();
                                    continue;
                                }
                                "h1" | "h2" | "h3" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    self.pop_until(element_kind);
                                    token = self.tokenizer.next();
                                    continue;
                                }
                                "a" => {
                                    let element_kind = ElementKind::from_str(tag)
                                        .expect("failed to convert string to ElementKind");
                                    self.pop_until(element_kind);
                                    token = self.tokenizer.next();
                                    continue;
                                }
                                _ => {
                                    token = self.tokenizer.next();
                                }
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            self.insert_char(ch);
                            token = self.tokenizer.next();
                            continue;
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                    }
                }
                InsertionMode::Text => {
                    match token_ref {
                        HtmlToken::EndTag { tag } => {
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.mode = self.original_insertion_mode;
                                token = self.tokenizer.next();
                                continue;
                            }

                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.mode = self.original_insertion_mode;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        &HtmlToken::Char(ch) => {
                            self.insert_char(ch);
                            token = self.tokenizer.next();
                            continue;
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = self.original_insertion_mode;
                }
                InsertionMode::AfterBody => {
                    match token_ref {
                        HtmlToken::EndTag { tag } => {
                            if tag == "html" {
                                self.mode = InsertionMode::AfterAfterBody;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        HtmlToken::Char(_) => {
                            token = self.tokenizer.next();
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                }
                InsertionMode::AfterAfterBody => {
                    match token_ref {
                        HtmlToken::Char(_) => {
                            token = self.tokenizer.next();
                            continue;
                        }
                        HtmlToken::Eof => {
                            return self.window.clone();
                        }
                        _ => {}
                    }

                    self.mode = InsertionMode::InBody;
                }
            }
        }

        self.window.clone()
    }

    fn create_element(&self, tag: &str, attributes: Vec<Attribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    fn insert_element(&mut self, tag: &str, attributes: Vec<Attribute>) {
        let new_child = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        let current_node = match self.stack_of_open_elements.last() {
            Some(node) => node.clone(),
            None => self.window.borrow().document(),
        };

        // let first_child = current_node.borrow().first_child();
        // if let Some(mut last_sibling) = first_child {
        //     // next_siblingを辿り、最後のsiblingに移動
        //     loop {
        //         let next_sibling = match last_sibling.borrow().next_sibling() {
        //             Some(next_sibling) => next_sibling,
        //             None => break,
        //         };
        //         last_sibling = next_sibling;
        //     }

        //     last_sibling
        //         .borrow_mut()
        //         .set_next_sibling(Some(new_child.clone()));

        //     new_child.borrow_mut().set_previous_sibling(Rc::downgrade(
        //         &current_node
        //             .borrow()
        //             .last_child()
        //             .upgrade()
        //             .expect("failed to get a last child"),
        //     ));
        // } else {
        //     current_node
        //         .borrow_mut()
        //         .set_first_child(Some(new_child.clone()));
        // }

        // siblingを辿らなくてもlast_childの次に追加すれば良いのでは。
        let last_child_option = current_node.borrow().last_child().upgrade();
        if let Some(last_child) = last_child_option {
            last_child
                .borrow_mut()
                .set_next_sibling(Some(new_child.clone()));
            new_child
                .borrow_mut()
                .set_previous_sibling(Rc::downgrade(&last_child));
        } else {
            current_node
                .borrow_mut()
                .set_first_child(Some(new_child.clone()));
        }

        current_node
            .borrow_mut()
            .set_last_child(Rc::downgrade(&new_child));

        new_child
            .borrow_mut()
            .set_parent(Rc::downgrade(&current_node));

        self.stack_of_open_elements.push(new_child);
    }

    fn pop_current_node(&mut self, element_kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(node) => node,
            None => return false,
        };

        if current.borrow().element_kind() == Some(element_kind) {
            self.stack_of_open_elements.pop();
            return true;
        }

        false
    }

    fn pop_until(&mut self, element_kind: ElementKind) {
        assert!(
            self.contain_in_stack(element_kind),
            "stack doesn't have an element {:?}",
            element_kind
        );

        loop {
            let current_node = match self.stack_of_open_elements.pop() {
                Some(node) => node,
                None => return,
            };

            if current_node.borrow().element_kind() == Some(element_kind) {
                return;
            }
        }
    }

    fn contain_in_stack(&self, element_kind: ElementKind) -> bool {
        self.stack_of_open_elements
            .iter()
            .any(|elem| elem.borrow().element_kind() == Some(element_kind))
    }

    fn insert_char(&mut self, ch: char) {
        // ルートノードの場合何もしない
        let current_node = match self.stack_of_open_elements.last() {
            Some(node) => node,
            None => return,
        };

        if let NodeKind::Text(s) = &mut current_node.borrow_mut().kind {
            s.push(ch);
            return;
        }

        if ch == '\n' || ch == ' ' {
            return;
        }

        let new_child = Rc::new(RefCell::new(self.create_text_node_from_char(ch)));

        // なぜfirst_childの後に追加するのだろうか...?
        // let first_child_option = current_node.borrow().first_child();
        // if let Some(first_child) = first_child_option {
        //     first_child
        //         .borrow_mut()
        //         .set_next_sibling(Some(new_child.clone()));
        //     new_child
        //         .borrow_mut()
        //         .set_previous_sibling(Rc::downgrade(&first_child));
        // } else {
        //     current_node
        //         .borrow_mut()
        //         .set_first_child(Some(new_child.clone()));
        // }

        let last_child_option = current_node.borrow().last_child().upgrade();
        if let Some(last_child) = last_child_option {
            last_child
                .borrow_mut()
                .set_next_sibling(Some(new_child.clone()));
            new_child
                .borrow_mut()
                .set_previous_sibling(Rc::downgrade(&last_child));
        } else {
            current_node
                .borrow_mut()
                .set_first_child(Some(new_child.clone()));
        }

        current_node
            .borrow_mut()
            .set_last_child(Rc::downgrade(&new_child));
        new_child
            .borrow_mut()
            .set_parent(Rc::downgrade(current_node));

        self.stack_of_open_elements.push(new_child);
    }

    fn create_text_node_from_char(&self, ch: char) -> Node {
        let mut s = String::new();
        s.push(ch);
        Node::new(NodeKind::Text(s))
    }
}

#[derive(Clone, Copy)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

#[cfg(test)]
mod tests {
    use super::HtmlParser;
    use crate::renderer::{
        dom::node::{Element, Node, NodeKind},
        html::{attribute::Attribute, token::HtmlTokenizer},
    };
    use alloc::{
        rc::Rc,
        string::{String, ToString},
        vec,
        vec::Vec,
    };
    use core::cell::RefCell;

    #[test]
    fn test_empty() {
        let html = String::new();
        let tokenizer = HtmlTokenizer::new(html);
        let window = HtmlParser::new(tokenizer).construct_tree();
        let document = window.borrow().document();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));

        assert_eq!(expected, document);
    }

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let tokenizer = HtmlTokenizer::new(html);
        let window = HtmlParser::new(tokenizer).construct_tree();
        let document = window.borrow().document();

        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );

        let html = document.borrow().first_child().unwrap();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );

        let head = html.borrow().first_child().unwrap();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "head",
                Vec::new()
            ))))),
            head
        );

        let body = head.borrow().next_sibling().unwrap();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );
    }

    // ギブ。以降コピペ

    #[test]
    fn test_text() {
        let html = "<html><head></head><body>text</body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );

        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );

        let body = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );

        let text = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }

    #[test]
    fn test_multiple_nodes() {
        let html = "<html><head></head><body><p><a foo=bar>text</a></p></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();

        let body = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );

        let p = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of body");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "p",
                Vec::new()
            ))))),
            p
        );

        let mut attr = Attribute::new();
        attr.add_name_char('f');
        attr.add_name_char('o');
        attr.add_name_char('o');
        attr.add_value_char('b');
        attr.add_value_char('a');
        attr.add_value_char('r');
        let a = p
            .borrow()
            .first_child()
            .expect("failed to get a first child of p");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "a",
                vec![attr]
            ))))),
            a
        );

        let text = a
            .borrow()
            .first_child()
            .expect("failed to get a first child of a");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }
}
