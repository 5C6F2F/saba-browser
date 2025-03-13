use crate::renderer::dom::node::Node;
use alloc::{
    format,
    rc::Rc,
    string::{String, ToString},
};
use core::cell::RefCell;

pub fn convert_dom_to_string(root: &Option<Rc<RefCell<Node>>>) -> String {
    let mut result = "\n".to_string();
    convert_dom_to_string_internal(root, 0, &mut result);
    result
}

fn convert_dom_to_string_internal(
    node: &Option<Rc<RefCell<Node>>>,
    depth: usize,
    result: &mut String,
) {
    match node {
        Some(node) => {
            result.push_str(&"  ".repeat(depth));
            result.push_str(&format!("{:?}", node.borrow().kind()));
            result.push('\n');
            convert_dom_to_string_internal(&node.borrow().first_child(), depth + 1, result);
            convert_dom_to_string_internal(&node.borrow().next_sibling(), depth, result);
        }
        None => {}
    }
}
