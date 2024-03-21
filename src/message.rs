use std::fmt::Display;

#[derive(Debug)]
pub struct Message {
    pub from: String,
    pub text: String,
}

impl Message {
    pub fn new(from: String, text: String) -> Self {
        Self { from, text }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.from, self.text)
    }
}
