use super::{
    dom::node::Window,
    html::{parser::HtmlParser, token::HtmlTokenizer},
};
use crate::{browser::Browser, http::HttpResponse, utils::convert_dom_to_string};
use alloc::{
    rc::{Rc, Weak},
    string::String,
};
use core::cell::RefCell;

pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
}

impl Page {
    pub fn new(browser: Weak<RefCell<Browser>>) -> Self {
        Self {
            browser,
            frame: None,
        }
    }

    pub fn receive_response(&mut self, response: HttpResponse) -> String {
        self.create_frame(response.body());

        if let Some(frame) = &self.frame {
            let dom = frame.borrow().document().clone();
            let debug = convert_dom_to_string(&Some(dom));
            return debug;
        }

        String::new()
    }

    fn create_frame(&mut self, html: String) {
        let tokenizer = HtmlTokenizer::new(html);
        let frame = HtmlParser::new(tokenizer).construct_tree();
        self.frame = Some(frame);
    }
}
