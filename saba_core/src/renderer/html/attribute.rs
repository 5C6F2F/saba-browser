use alloc::string::String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    name: String,
    value: String,
}

impl Attribute {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }

    pub fn add_name_char(&mut self, ch: char) {
        self.name.push(ch);
    }

    pub fn add_value_char(&mut self, ch: char) {
        self.value.push(ch);
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}
