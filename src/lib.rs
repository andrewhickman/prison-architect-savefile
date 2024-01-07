mod format;
mod parse;

pub use crate::parse::ParseError;

use std::{fmt, io, path::Path, vec};

use indexmap::IndexMap;

/// Reads a prison architect savefile from the file system.
pub fn read(path: impl AsRef<Path>) -> io::Result<Node> {
    Node::read(path)
}

/// A single element of a savefile.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Node {
    properties: IndexMap<String, Vec<String>>,
    children: IndexMap<String, Vec<Node>>,
}

impl Node {
    /// Create a new, empty, node.
    pub fn new() -> Self {
        Default::default()
    }

    /// Parses a node from a string.
    pub fn parse(input: impl AsRef<str>) -> Result<Self, ParseError> {
        input.as_ref().parse()
    }

    /// Reads a savefile from the file system.
    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let input = fs_err::read_to_string(path)?;
        Node::parse(input).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
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
    pub fn property(&self, key: impl AsRef<str>) -> Option<&str> {
        self.properties
            .get(key.as_ref())
            .and_then(|value| value.first())
            .map(|value| value.as_str())
    }

    /// Returns true if this node contains at least one property with the given key.
    pub fn has_property(&mut self, key: impl AsRef<str>) -> bool {
        self.properties
            .get(key.as_ref())
            .map_or(false, |values| !values.is_empty())
    }

    /// Sets the property value for a given key. If a value is already set, it is cleared.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let values = self.properties.entry(key.into()).or_default();
        values.clear();
        values.push(value.into());
    }

    /// Sets an additional property value for a given key. Existing values are kept.
    pub fn add_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties
            .entry(key.into())
            .or_default()
            .push(value.into())
    }

    /// Removes all property values for the given key and returns them.
    pub fn clear_property(&mut self, key: impl AsRef<str>) -> impl Iterator<Item = String> {
        match self.properties.remove(key.as_ref()) {
            Some(properties) => properties.into_iter(),
            None => vec::IntoIter::default(),
        }
    }

    /// Removes all property values for this node and returns them.
    pub fn clear_properties(&mut self) -> impl Iterator<Item = (String, String)> + '_ {
        self.properties.drain(..).flat_map(|(key, properties)| {
            properties
                .into_iter()
                .map(move |property| (key.clone(), property))
        })
    }

    /// Adds the given key-value properties to this node.
    pub fn extend_properties<K, V>(&mut self, properties: impl IntoIterator<Item = (K, V)>)
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (key, value) in properties {
            self.properties
                .entry(key.into())
                .or_default()
                .push(value.into())
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
    pub fn has_child(&mut self, key: impl AsRef<str>) -> bool {
        self.children
            .get(key.as_ref())
            .map_or(false, |values| !values.is_empty())
    }

    /// Sets the child value for a given key. If a child is already set, it is cleared.
    pub fn set_child(&mut self, key: impl Into<String>, child: Node) {
        let values = self.children.entry(key.into()).or_default();
        values.clear();
        values.push(child);
    }

    /// Sets an additional child value for a given key. Existing children are kept.
    pub fn add_child(&mut self, key: impl Into<String>, child: Node) {
        self.children.entry(key.into()).or_default().push(child)
    }

    /// Removes all children for the given key and returns them.
    pub fn clear_child(&mut self, key: impl AsRef<str>) -> impl Iterator<Item = Node> {
        match self.children.remove(key.as_ref()) {
            Some(children) => children.into_iter(),
            None => vec::IntoIter::default(),
        }
    }

    /// Removes all children for this node and returns them.
    pub fn clear_children(&mut self) -> impl Iterator<Item = (String, Node)> + '_ {
        self.children
            .drain(..)
            .flat_map(|(key, children)| children.into_iter().map(move |child| (key.clone(), child)))
    }

    /// Adds the given children to this node.
    pub fn extend_children<K>(&mut self, properties: impl IntoIterator<Item = (K, Node)>)
    where
        K: Into<String>,
    {
        for (key, child) in properties {
            self.children.entry(key.into()).or_default().push(child)
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
