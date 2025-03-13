use crate::renderer::page::Page;
use alloc::{rc::Rc, vec::Vec};
use core::cell::RefCell;

pub struct Browser {
    active_page_index: usize,
    pages: Vec<Rc<RefCell<Page>>>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let browser = Rc::new(RefCell::new(Self {
            active_page_index: 0,
            pages: Vec::new(),
        }));

        let page = Page::new(Rc::downgrade(&browser));
        browser.borrow_mut().pages.push(Rc::new(RefCell::new(page)));

        browser
    }

    pub fn current_page(&self) -> Rc<RefCell<Page>> {
        self.pages[self.active_page_index].clone()
    }
}
