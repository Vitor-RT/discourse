#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Delete,
    Insert,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub start: Position,
    pub finish: Option<Position>,
    pub operation: Operation,
    pub text: Option<String>,
}

pub struct ContentBuffer {
    pub lines: Vec<String>,
}

impl ContentBuffer {
    pub fn new(initial_content: &str) -> Self {
        let lines = initial_content.split('\n').map(String::from).collect();
        Self { lines }
    }

    pub fn apply_transform(&mut self, transform: Transform) {
        let start_row = transform.start.row;
        let start_col = transform.start.col;

        match transform.operation {
            Operation::Delete => {
                let finish_row = transform.finish.as_ref().unwrap().row;
                let finish_col = transform.finish.as_ref().unwrap().col;

                let mut first_line = self.lines[start_row].clone();
                first_line.truncate(start_col);

                if finish_row == start_row {
                    let rest = &self.lines[start_row][finish_col..];
                    first_line.push_str(rest);
                    self.lines[start_row] = first_line;
                    return;
                }

                self.lines[start_row] = first_line;

                let mut removed_lines = Vec::new();
                for _ in 0..(finish_row - start_row) {
                    removed_lines.push(self.lines.remove(start_row + 1));
                }

                let append_str = if finish_row < self.lines.len() {
                    let chars: Vec<char> = self.lines[finish_row].chars().collect();
                    if finish_col > 0 && finish_col - 1 < chars.len() {
                        chars[finish_col - 1..].iter().collect::<String>()
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };
                self.lines[start_row].push_str(&append_str);
            }
            Operation::Insert => {
                let text = transform.text.unwrap();
                self.lines[start_row].insert_str(start_col, &text);

                let split: Vec<String> = self.lines[start_row]
                    .split('\n')
                    .map(|s| s.to_string())
                    .collect();
                if split.len() > 1 {
                    self.lines[start_row] = split[0].clone();
                    let mut i = 1;

                    if split.len() > 2 {
                        for line in &split[1..split.len() - 1] {
                            self.lines.insert(start_row + i, line.clone());
                            i += 1;
                        }
                    }

                    if self.lines.len() <= start_row + i {
                        self.lines.insert(start_row + i, String::new());
                    }

                    let new_val = format!("{}{}", split.last().unwrap(), self.lines[start_row + i]);
                    self.lines[start_row + i] = new_val;
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deletion_across_lines() {
        let mut c = ContentBuffer::new("a\nbc\nc");
        c.apply_transform(Transform {
            start: Position { row: 0, col: 0 },
            finish: Some(Position { row: 1, col: 1 }),
            operation: Operation::Delete,
            text: None,
        });
        assert_eq!(c.to_string(), "c\nc");
    }

    #[test]
    fn test_deletion_inside_lines() {
        let mut c = ContentBuffer::new("hello world");
        c.apply_transform(Transform {
            start: Position { row: 0, col: 1 },
            finish: Some(Position { row: 0, col: 4 }),
            operation: Operation::Delete,
            text: None,
        });
        assert_eq!(c.to_string(), "ho world");
    }

    #[test]
    fn test_inserts_inside_lines() {
        let mut c = ContentBuffer::new("hello!");
        c.apply_transform(Transform {
            start: Position { row: 0, col: 5 },
            finish: None,
            operation: Operation::Insert,
            text: Some(" world".to_string()),
        });
        assert_eq!(c.to_string(), "hello world!");
    }

    #[test]
    fn test_multiline_inserts() {
        let mut c = ContentBuffer::new("hello!");
        c.apply_transform(Transform {
            start: Position { row: 0, col: 5 },
            finish: None,
            operation: Operation::Insert,
            text: Some("\nworld".to_string()),
        });
        assert_eq!(c.to_string(), "hello\nworld!");
    }

    #[test]
    fn test_multiline_inserts_row2() {
        let mut c = ContentBuffer::new("line0\nline1\nline2");
        c.apply_transform(Transform {
            start: Position { row: 2, col: 0 },
            finish: None,
            operation: Operation::Insert,
            text: Some("foo\nbar\nbaz".to_string()),
        });
        assert_eq!(c.to_string(), "line0\nline1\nfoo\nbar\nbazline2");
    }
}
