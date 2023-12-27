mod format;
mod parse;

use std::{
    collections::{hash_map, HashMap},
    fmt, io,
    path::Path,
    vec,
};

use crate::parse::ParseError;

/// Reads a prison architect savefile from the file system.
pub fn read(path: impl AsRef<Path>) -> io::Result<Node> {
    Node::read(path)
}

/// A single element of a savefile.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Node {
    properties: HashMap<String, Vec<String>>,
    children: HashMap<String, Vec<Node>>,
}

impl Node {
    /// Create a new, empty, node.
    pub fn new() -> Self {
        Default::default()
    }

    /// Parses a node from a string.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        input.parse()
    }

    /// Reads a savefile from the file system.
    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let input = fs_err::read_to_string(path)?;
        Node::parse(&input).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    /// Writes a savefile to the file system.
    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let output = self.to_string();
        fs_err::write(path, output)
    }

    /// Gets an iterator over all key-value properties on this node.
    pub fn properties(&self) -> impl Iterator<Item = (&str, &str)> {
        self.properties
            .iter()
            .flat_map(|(key, values)| values.iter().map(|value| (key.as_str(), value.as_str())))
    }

    /// Gets the property value for a given key, if it exists. If multiple values are set, the first is returned.
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .get(key)
            .and_then(|value| value.first())
            .map(|value| value.as_str())
    }

    /// Returns true if this node contains at least one property with the given key.
    pub fn has_property(&mut self, key: &str) -> bool {
        self.properties
            .get(key)
            .map_or(false, |values| !values.is_empty())
    }

    /// Sets the property value for a given key. If a value is already set, it is cleared.
    pub fn set_property(&mut self, key: &str, value: &str) {
        let values = self.properties.entry(key.to_owned()).or_default();
        values.clear();
        values.push(value.to_owned());
    }

    /// Sets an additional property value for a given key. Existing values are kept.
    pub fn add_property(&mut self, key: &str, value: &str) {
        self.properties
            .entry(key.to_owned())
            .or_default()
            .push(value.to_owned())
    }

    /// Removes all property values for the given key and returns them.
    pub fn clear_property(&mut self, key: &str) -> impl Iterator<Item = String> {
        match self.properties.entry(key.to_owned()) {
            hash_map::Entry::Occupied(entry) => entry.remove().into_iter(),
            hash_map::Entry::Vacant(_) => vec::IntoIter::default(),
        }
    }

    /// Removes all property values for this node and returns them.
    pub fn clear_properties(&mut self) -> impl Iterator<Item = (String, String)> + '_ {
        self.properties.drain().flat_map(|(key, properties)| {
            properties
                .into_iter()
                .map(move |property| (key.clone(), property))
        })
    }

    /// Adds the given key-value properties to this node.
    pub fn extend_properties(&mut self, properties: impl IntoIterator<Item = (String, String)>) {
        for (key, value) in properties {
            self.properties.entry(key).or_default().push(value)
        }
    }

    /// Gets an iterator over all children of this node.
    pub fn children(&self) -> impl Iterator<Item = (&str, &Node)> {
        self.children
            .iter()
            .flat_map(|(key, children)| children.iter().map(|child| (key.as_str(), child)))
    }

    /// Gets a mutable iterator over all children of this node.
    pub fn children_mut(&mut self) -> impl Iterator<Item = (&str, &mut Node)> {
        self.children
            .iter_mut()
            .flat_map(|(key, children)| children.iter_mut().map(|child| (key.as_str(), child)))
    }

    /// Gets the child for a given key, if it exists. If multiple children exist, the first is returned.
    pub fn child(&self, key: &str) -> Option<&Node> {
        self.children.get(key).and_then(|child| child.first())
    }

    /// Gets a mutable reference to the child for a given key, if it exists. If multiple children exist, the first is returned.
    pub fn child_mut(&mut self, key: &str) -> Option<&mut Node> {
        self.children
            .get_mut(key)
            .and_then(|child| child.first_mut())
    }

    /// Returns true if this node contains at least child with the given key.
    pub fn has_child(&mut self, key: &str) -> bool {
        self.children
            .get(key)
            .map_or(false, |values| !values.is_empty())
    }

    /// Sets the child value for a given key. If a child is already set, it is cleared.
    pub fn set_child(&mut self, key: &str, child: Node) {
        let values = self.children.entry(key.to_owned()).or_default();
        values.clear();
        values.push(child);
    }

    /// Sets an additional child value for a given key. Existing children are kept.
    pub fn add_child(&mut self, key: &str, child: Node) {
        self.children.entry(key.to_owned()).or_default().push(child)
    }

    /// Removes all children for the given key and returns them.
    pub fn clear_child(&mut self, key: &str) -> impl Iterator<Item = Node> {
        match self.children.entry(key.to_owned()) {
            hash_map::Entry::Occupied(entry) => entry.remove().into_iter(),
            hash_map::Entry::Vacant(_) => vec::IntoIter::default(),
        }
    }

    /// Removes all children for this node and returns them.
    pub fn clear_children(&mut self) -> impl Iterator<Item = (String, Node)> + '_ {
        self.children
            .drain()
            .flat_map(|(key, children)| children.into_iter().map(move |child| (key.clone(), child)))
    }

    /// Adds the given children to this node.
    pub fn extend_children(&mut self, properties: impl IntoIterator<Item = (String, Node)>) {
        for (key, child) in properties {
            self.children.entry(key).or_default().push(child)
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.properties())
            .entries(self.children())
            .finish()
    }
}
