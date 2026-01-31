use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionHistory {
    stack: Vec<String>,
    head: Option<usize>,
}

impl SessionHistory {
    pub fn prev(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index > 0 => Some(index - 1),
            Some(_index) => None,
            None if length > 0 => panic!("no session head in a non-empty stack"),
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    pub fn next(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index < length - 1 => Some(index + 1),
            Some(_index) => None,
            None if length > 0 => panic!("no session head in a non-empty stack"),
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    fn truncate(&mut self) {
        if let Some(index) = self.head {
            while self.stack.len() > index {
                self.stack.pop();
            }

            self.head = None;
        }
    }

    pub fn add_session(&mut self, session: String) {
        let length = self.stack.len();
        match self.head {
            // move the head if the next session in the stack is the one we
            // wanted to add
            Some(index) if index < length - 1 && self.stack[index + 1] == session => {
                self.head = Some(index + 1);
            },
            _ => {
                self.truncate();
                self.stack.push(session);
            }
        }
    }
}
