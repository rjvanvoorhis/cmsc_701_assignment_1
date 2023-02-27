#[derive(Debug)]
pub struct Record {
    pub header: String,
    pub sequence: String,
}

impl Record {
    pub fn new() -> Self {
        Self {
            header: String::new(),
            sequence: String::new(),
        }
    }

    pub fn header(&self) -> &str {
        self.header.as_ref()
    }

    pub fn sequence(&self) -> &str {
        self.sequence.as_ref()
    }

    pub fn push_sequence_part(&mut self, part: &str) {
        self.sequence.push_str(part)
    }

    pub fn set_header(&mut self, header: String) {
        self.header = header;
    }

    pub fn clear(&mut self) {
        self.header.clear();
        self.sequence.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.header.is_empty() && self.sequence.is_empty()
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}
