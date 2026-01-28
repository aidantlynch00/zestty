use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionHistory {
    stack: Vec<String>,
    head: Option<usize>,
}

impl SessionHistory {
    #[tracing::instrument(skip_all)]
    pub fn prev(&mut self, session: String) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index > 0 => Some(index - 1),
            Some(_index) => None,
            None if length > 0 => {
                self.stack.push(session);
                Some(length - 1) // precomputed
            },
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    #[tracing::instrument(skip_all)]
    pub fn next(&mut self) -> Option<&str> {
        let length = self.stack.len();
        let new_index = match self.head {
            Some(index) if index < length - 1 => Some(index + 1),
            Some(_index) => None,
            None => None,
        }?;

        self.head = Some(new_index);
        Some(&self.stack[new_index])
    }

    #[tracing::instrument(skip_all)]
    pub fn truncate(&mut self) {
        if let Some(index) = self.head {
            while self.stack.len() > index {
                self.stack.pop();
            }

            self.head = None;
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn push(&mut self, session: String) {
        self.stack.push(session);
    }
}
