use std::fmt::{self, Write};

use crate::Node;

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Writer { f, depth: 0 }.write_node(self)
    }
}

struct Writer<F> {
    f: F,
    depth: u32,
}

impl<F> Writer<F>
where
    F: Write,
{
    fn write_node(&mut self, node: &Node) -> fmt::Result {
        let mut first = true;
        for (key, value) in node.properties() {
            if !first {
                self.write_newline()?;
            }
            first = false;

            self.write_property(key, value)?;
        }

        for (key, value) in node.children() {
            if !first {
                self.write_newline()?;
            }
            first = false;

            self.f.write_str("BEGIN ")?;
            self.write_str(key)?;

            if value.children.is_empty() {
                self.f.write_char(' ')?;
                for (key, value) in value.properties() {
                    self.write_property(key, value)?;
                    self.f.write_char(' ')?;
                }
                self.f.write_char(' ')?;
            } else {
                self.depth += 1;
                self.write_newline()?;

                self.write_node(value)?;

                self.depth -= 1;
                self.write_newline()?;
            }

            self.f.write_str("END")?;
        }

        Ok(())
    }

    fn write_property(&mut self, key: &str, value: &str) -> fmt::Result {
        self.write_str(key)?;
        self.f.write_char(' ')?;
        self.write_str(value)?;
        Ok(())
    }

    fn write_str(&mut self, value: &str) -> fmt::Result {
        let mut special_chars = value.match_indices(&[' ', '"', '\n']).peekable();
        if special_chars.peek().is_some() {
            self.f.write_char('"')?;
            let mut pos = 0;
            for (start, ch) in special_chars {
                self.f.write_str(&value[pos..start])?;
                match ch {
                    " " => self.f.write_char(' ')?,
                    "\"" => self.f.write_str("\\\"")?,
                    "\n" => self.f.write_str("\\n")?,
                    _ => unreachable!(),
                }
                pos = start + ch.len();
            }
            self.f.write_str(&value[pos..])?;
            self.f.write_char('"')?;
            Ok(())
        } else {
            self.f.write_str(value)
        }
    }

    fn write_newline(&mut self) -> fmt::Result {
        self.f.write_char('\n')?;
        for _ in 0..self.depth {
            self.f.write_str("    ")?;
        }
        Ok(())
    }
}

#[test]
fn test() {
    use std::collections::HashMap;

    let node = Node {
        properties: HashMap::from_iter([("foo".to_owned(), vec!["hello, world!".to_owned()])]),
        children: HashMap::from_iter([(
            "bar".to_owned(),
            vec![Node {
                properties: HashMap::from_iter([(
                    "baz".to_owned(),
                    vec!["one\n\"two\"".to_owned()],
                )]),
                children: HashMap::default(),
            }],
        )]),
    };

    assert_eq!(
        node.to_string(),
        "foo \"hello, world!\"\nBEGIN bar baz \"one\\n\\\"two\\\"\"  END"
    )
}
