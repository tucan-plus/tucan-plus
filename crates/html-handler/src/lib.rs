use std::fmt::Debug;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::ops::Deref;

use data_encoding::BASE64URL_NOPAD;
use ego_tree::iter::Edge;
use ego_tree::{NodeMut, NodeRef, Tree};
use html5ever::serialize::{Serialize, SerializeOpts, Serializer, TraversalScope, serialize};
use scraper::node::{Attrs, Element, ProcessingInstruction, Text};
use scraper::{Html, StrTendril};
use scraper::{Node, node::Doctype};
use sha3::{Digest, Sha3_256};

// Copied from https://github.com/rust-scraper/scraper licensed under ISC License
/*
Copyright © 2016, June McEnroe <june@causal.agency>
Copyright © 2017, Vivek Kushwaha <yoursvivek@gmail.com>
Copyright © 2024-2025, rust-scraper Contributors

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#[derive(Clone, PartialEq, Eq)]
pub enum MyNode {
    /// The document root.
    Document,

    /// The fragment root.
    Fragment,

    /// A doctype.
    Doctype(Doctype),

    /// Text.
    Text(Text),

    /// An element.
    Element(Element),

    /// A processing instruction.
    ProcessingInstruction(ProcessingInstruction),
}

impl MyNode {
    /// Returns true if node is the document root.
    pub const fn is_document(&self) -> bool {
        matches!(*self, Self::Document)
    }

    /// Returns true if node is the fragment root.
    pub const fn is_fragment(&self) -> bool {
        matches!(*self, Self::Fragment)
    }

    /// Returns true if node is a doctype.
    pub const fn is_doctype(&self) -> bool {
        matches!(*self, Self::Doctype(_))
    }

    /// Returns true if node is text.
    pub const fn is_text(&self) -> bool {
        matches!(*self, Self::Text(_))
    }

    /// Returns true if node is an element.
    pub const fn is_element(&self) -> bool {
        matches!(*self, Self::Element(_))
    }

    /// Returns self as a doctype.
    pub const fn as_doctype(&self) -> Option<&Doctype> {
        match *self {
            Self::Doctype(ref d) => Some(d),
            _ => None,
        }
    }

    /// Returns self as text.
    pub const fn as_text(&self) -> Option<&Text> {
        match *self {
            Self::Text(ref t) => Some(t),
            _ => None,
        }
    }

    /// Returns self as an element.
    pub const fn as_element(&self) -> Option<&Element> {
        match *self {
            Self::Element(ref e) => Some(e),
            _ => None,
        }
    }

    /// Returns self as an element.
    pub const fn as_processing_instruction(&self) -> Option<&ProcessingInstruction> {
        match *self {
            Self::ProcessingInstruction(ref pi) => Some(pi),
            _ => None,
        }
    }
}

// Always use one line.
impl core::fmt::Debug for MyNode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match *self {
            Self::Document => write!(f, "Document"),
            Self::Fragment => write!(f, "Fragment"),
            Self::Doctype(ref d) => write!(f, "Doctype({d:?})"),
            Self::Text(ref t) => write!(f, "Text({t:?})"),
            Self::Element(ref e) => write!(f, "Element({e:?})"),
            Self::ProcessingInstruction(ref pi) => write!(f, "ProcessingInstruction({pi:?})"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MyElementRef<'a> {
    node: NodeRef<'a, MyNode>,
}

impl<'a> MyElementRef<'a> {
    /// Wraps a `NodeRef` only if it references a `Node::Element`.
    #[must_use]
    pub fn wrap(node: NodeRef<'a, MyNode>) -> Option<Self> {
        if node.value().is_element() {
            Some(MyElementRef { node })
        } else {
            None
        }
    }

    /// Returns the `Element` referenced by `self`.
    #[must_use]
    pub fn value(&self) -> &'a Element {
        self.node.value().as_element().unwrap()
    }

    fn serialize(&self, traversal_scope: TraversalScope) -> String {
        let opts = SerializeOpts {
            scripting_enabled: false, // It's not clear what this does.
            traversal_scope,
            create_missing_parent: false,
        };
        let mut buf = Vec::new();
        serialize(&mut buf, self, opts).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Returns the HTML of this element.
    #[must_use]
    pub fn html(&self) -> String {
        self.serialize(TraversalScope::IncludeNode)
    }

    /// Returns the inner HTML of this element.
    #[must_use]
    pub fn inner_html(&self) -> String {
        self.serialize(TraversalScope::ChildrenOnly(None))
    }

    /// Returns the value of an attribute.
    #[must_use]
    pub fn attr(&self, attr: &str) -> Option<&'a str> {
        self.value().attr(attr)
    }
}

impl Debug for MyElementRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.value(), f)
    }
}

impl<'a> Deref for MyElementRef<'a> {
    type Target = NodeRef<'a, MyNode>;
    fn deref(&self) -> &NodeRef<'a, MyNode> {
        &self.node
    }
}

impl Serialize for MyElementRef<'_> {
    fn serialize<S: Serializer>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope,
    ) -> Result<(), std::io::Error> {
        myserialize(**self, serializer, &traversal_scope)
    }
}

pub struct MyHtml<'a>(pub &'a Tree<MyNode>);

impl Serialize for MyHtml<'_> {
    fn serialize<S: Serializer>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope,
    ) -> Result<(), std::io::Error> {
        myserialize(self.0.root(), serializer, &traversal_scope)
    }
}

/// Serialize an HTML node using html5ever serializer.
pub(crate) fn myserialize<S: Serializer>(
    self_node: NodeRef<MyNode>,
    serializer: &mut S,
    traversal_scope: &TraversalScope,
) -> Result<(), std::io::Error> {
    for edge in self_node.traverse() {
        match edge {
            Edge::Open(node) => {
                if node == self_node && *traversal_scope == TraversalScope::ChildrenOnly(None) {
                    continue;
                }

                match *node.value() {
                    MyNode::Doctype(ref doctype) => {
                        serializer.write_doctype(doctype.name())?;
                    }
                    MyNode::Text(ref text) => {
                        serializer.write_text(text)?;
                    }
                    MyNode::Element(ref elem) => {
                        let attrs = elem.attrs.iter().map(|(k, v)| (k, &v[..]));
                        serializer.start_elem(elem.name.clone(), attrs)?;
                    }
                    _ => (),
                }
            }

            Edge::Close(node) => {
                if node == self_node && *traversal_scope == TraversalScope::ChildrenOnly(None) {
                    continue;
                }

                if let Some(elem) = node.value().as_element() {
                    serializer.end_elem(elem.name.clone())?;
                }
            }
        }
    }

    Ok(())
}

pub fn parse_document(content: &str) -> Tree<MyNode> {
    let html = Html::parse_document(content);
    let mut tree = Tree::new(MyNode::Document);
    assert!(html.tree.root().value().is_document());
    convert_children_inner(tree.root_mut(), html.tree.root());
    tree
}

#[must_use]
pub fn html(tree: &Tree<MyNode>) -> String {
    let opts = SerializeOpts {
        scripting_enabled: false, // It's not clear what this does.
        traversal_scope: html5ever::serialize::TraversalScope::IncludeNode,
        create_missing_parent: false,
    };
    let mut buf = Vec::new();
    serialize(&mut buf, &MyHtml(tree), opts).unwrap();
    String::from_utf8(buf).unwrap()
}

fn convert_children_inner(mut new_parent: NodeMut<'_, MyNode>, old_node: NodeRef<'_, Node>) {
    old_node.children().for_each(|old_child| {
        let new_child = match old_child.value() {
            Node::Document => MyNode::Document,
            Node::Fragment => MyNode::Fragment,
            Node::Doctype(doctype) => MyNode::Doctype(doctype.clone()),
            Node::Text(text) if !text.trim().is_empty() => MyNode::Text(Text {
                text: StrTendril::from_slice(text.trim()),
            }),
            Node::Element(element) => MyNode::Element(element.clone()),
            Node::ProcessingInstruction(processing_instruction) => {
                MyNode::ProcessingInstruction(processing_instruction.clone())
            }
            _ => return,
        };
        let new_child = new_parent.append(new_child);
        convert_children_inner(new_child, old_child);
    });
}

pub struct Root<'a> {
    node: NodeRef<'a, MyNode>,
}

pub struct BeforeDoctype;

pub struct AfterDoctype;

pub struct InRoot<'a, OuterState> {
    node: NodeRef<'a, MyNode>,
    current_child: Option<NodeRef<'a, MyNode>>,
    outer_state: PhantomData<OuterState>,
}

pub struct Open<'a, OuterState> {
    element: NodeRef<'a, MyNode>,
    attrs: Peekable<Attrs<'a>>,
    outer_state: PhantomData<OuterState>,
}

pub struct InElement<'a, OuterState> {
    element: NodeRef<'a, MyNode>,
    current_child: Option<NodeRef<'a, MyNode>>,
    outer_state: PhantomData<OuterState>,
}

impl<'a> Root<'a> {
    #[must_use]
    pub fn new(node: NodeRef<'a, MyNode>) -> Self {
        assert_eq!(
            *node.value(),
            MyNode::Document,
            "expected document but got {:?}",
            node.value()
        );
        Self { node }
    }

    #[must_use]
    pub fn document_start(self) -> InRoot<'a, Self> {
        InRoot {
            node: self.node,
            current_child: self.node.children().next(),
            outer_state: PhantomData,
        }
    }
}

impl<'a> InRoot<'a, Root<'a>> {
    #[must_use]
    pub const fn peek(&self) -> Option<&NodeRef<'a, MyNode>> {
        self.current_child.as_ref()
    }

    #[track_caller]
    #[must_use]
    pub fn doctype(self) -> Self {
        let child_node = self.current_child.expect("expected child but none left");
        let Some(_child_element) = child_node.value().as_doctype() else {
            panic!("expected doctype but got {:?}", child_node.value())
        };
        InRoot {
            node: self.node,
            current_child: child_node.next_sibling(),
            outer_state: self.outer_state,
        }
    }

    #[track_caller]
    pub fn end_document(self) {
        assert_eq!(
            self.current_child, None,
            "Expected no remaining children but got {:?}",
            self.current_child
        );
    }
}

impl<'a, OuterState> InRoot<'a, OuterState> {
    #[track_caller]
    #[must_use]
    pub const fn skip_whitespace(self) -> Self {
        self
    }

    #[must_use]
    pub fn next_child_tag_open_start(self, name: &str) -> Open<'a, Self> {
        let child_node = self.current_child.expect("expected child but none left");
        let Some(child_element) = child_node.value().as_element() else {
            panic!("expected element but got {:?}", child_node.value())
        };
        assert_eq!(
            child_element.name(),
            name,
            "{}",
            MyElementRef::wrap(child_node).unwrap().html()
        );
        Open {
            element: child_node,
            attrs: child_element.attrs().peekable(),
            outer_state: PhantomData,
        }
    }
}

impl<'a, OuterState> Open<'a, OuterState> {
    #[track_caller]
    #[must_use]
    pub fn attribute(mut self, name: &str, value: &str) -> Self {
        if name == "xss" {
            while let Some((attr_key, _attr_value)) = self.attrs.peek() {
                if *attr_key == value {
                    break;
                }
                self.attrs.next().unwrap();
            }
            return self;
        }
        assert_eq!(
            self.attrs.next().expect("expected attribute but none left"),
            (name, value),
            "{}",
            MyElementRef::wrap(self.element).unwrap().html()
        );
        self
    }

    #[track_caller]
    #[must_use]
    pub fn attribute_value(mut self, expected_name: &str) -> (Self, String) {
        let (name, value) = self.attrs.next().expect("expected attribute but none left");
        assert_eq!(
            name,
            expected_name,
            "{}",
            MyElementRef::wrap(self.element).unwrap().html()
        );
        (self, value.to_owned())
    }

    #[track_caller]
    #[must_use]
    pub fn tag_open_end(mut self) -> InElement<'a, OuterState> {
        let Some(_child_element) = self.element.value().as_element() else {
            panic!("expected element but got {:?}", self.element.value())
        };
        let attr = self.attrs.next();
        assert_eq!(
            attr,
            None,
            "expected no remaining attributes but got {attr:?} in {}",
            MyElementRef::wrap(self.element).unwrap().html()
        );
        InElement {
            element: self.element,
            current_child: self.element.children().next(),
            outer_state: self.outer_state,
        }
    }
}

impl<'a, OuterState> InElement<'a, OuterState> {
    #[must_use]
    pub const fn peek(&self) -> Option<&NodeRef<'a, MyNode>> {
        self.current_child.as_ref()
    }

    #[must_use]
    #[track_caller]
    pub fn next_any_child(mut self) -> (Self, NodeRef<'a, MyNode>) {
        let current_child = self.current_child.expect("expected child but none left");
        self.current_child = current_child.next_sibling();
        (self, current_child)
    }

    #[track_caller]
    #[must_use]
    pub const fn skip_whitespace(self) -> Self {
        self
    }

    #[track_caller]
    #[must_use]
    pub fn text(mut self) -> (Self, String) {
        let child_node = self
            .current_child
            .expect("expected child with text but got no children. maybe there is a closing tag?");
        let Some(child_element) = child_node.value().as_text() else {
            panic!("expected text but got {:?}", child_node.value())
        };
        self.current_child = child_node.next_sibling();
        (self, child_element.to_string())
    }

    #[track_caller]
    #[must_use]
    pub fn skip_text(mut self, text: &str) -> Self {
        let child_node = self
            .current_child
            .expect("expected child with text but got no children. maybe there is a closing tag?");
        let Some(child_element) = child_node.value().as_text() else {
            panic!("expected text but got {:?}", child_node.value())
        };
        match BASE64URL_NOPAD.decode(text.as_bytes()) {
            Ok(value) if value.len() == 32 => {
                let actual_hash = BASE64URL_NOPAD.encode(&Sha3_256::digest(&**child_element));
                assert_eq!(actual_hash, text);
            }
            _ => {
                assert_eq!(
                    &**child_element,
                    text,
                    "{}",
                    BASE64URL_NOPAD.encode(&Sha3_256::digest(&**child_element))
                );
            }
        }
        self.current_child = child_node.next_sibling();
        self
    }

    #[track_caller]
    #[must_use]
    pub const fn skip_any_comment(self) -> Self {
        self
    }

    #[track_caller]
    #[must_use]
    pub fn next_child_tag_open_start(self, name: &str) -> Open<'a, Self> {
        let Some(_child_element) = self.element.value().as_element() else {
            panic!("expected element but got {:?}", self.element.value())
        };
        let Some(child_node) = self.current_child else {
            panic!(
                "expected one more child in {}",
                MyElementRef::wrap(self.element).unwrap().html()
            )
        };
        let Some(child_element) = child_node.value().as_element() else {
            panic!("expected element but got {:?}", child_node.value())
        };
        assert_eq!(
            child_element.name(),
            name,
            "{}",
            MyElementRef::wrap(child_node).unwrap().html()
        );
        Open {
            element: child_node,
            attrs: child_element.attrs().peekable(),
            outer_state: PhantomData,
        }
    }
}

impl<'a, OuterState> InElement<'a, InElement<'a, OuterState>> {
    #[track_caller]
    #[must_use]
    pub fn close_element(self, name: &str) -> InElement<'a, OuterState> {
        assert_eq!(
            self.current_child.map(|child| child.value()),
            None,
            "expected there to be no more children"
        );
        let Some(element) = self.element.value().as_element() else {
            panic!("expected element but got {:?}", self.element.value())
        };
        assert_eq!(
            element.name(),
            name,
            "Expected tag to be {name} but got {} for {}",
            element.name(),
            MyElementRef::wrap(self.element).unwrap().html()
        );
        InElement {
            element: self.element.parent().unwrap(),
            current_child: self.element.next_sibling(),
            outer_state: PhantomData,
        }
    }
}

impl<'a, OuterState> InElement<'a, InRoot<'a, OuterState>> {
    #[track_caller]
    #[must_use]
    pub fn close_element(self, name: &str) -> InRoot<'a, OuterState> {
        assert_eq!(
            self.current_child
                .map(|child| MyElementRef::wrap(child).unwrap().html()),
            None,
            "expected there to be no more children"
        );
        let Some(element) = self.element.value().as_element() else {
            panic!("expected element but got {:?}", self.element.value())
        };
        assert_eq!(
            element.name(),
            name,
            "Expected tag to be {name} but got {} for {}",
            element.name(),
            MyElementRef::wrap(self.element).unwrap().html()
        );
        InRoot {
            node: self.element.parent().unwrap(),
            current_child: self.element.next_sibling(),
            outer_state: PhantomData,
        }
    }
}
