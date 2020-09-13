//! `augdom` provides an "augmented DOM" implementation that can run almost
//! anywhere Rust can. By default the `webdom` feature is enabled and this crate
//! is a wrapper around `web-sys` for creating and manipulating HTML elements.
//! See the [crate::Dom] trait for the provided behavior.
//!
//! The `rsdom` feature enables a DOM emulation layer written in pure Rust which
//! can be used for testing or to render HTML strings.
//!
//! # Known Limitations
//!
//! As of today the `<web_sys::Element as Dom>::*_attribute` methods will panic
//! if called on a text node. This cost seems appropriate today because this is
//! a dependency for other crates which enforce this requirement themselves.
//! `web_sys` enforces this restriction statically.

#![deny(clippy::all, missing_docs)]

static_assertions::assert_cfg!(
    any(feature = "webdom", feature = "rsdom"),
    "At least one DOM implementation's feature must be enabled (`webdom`, `rsdom`)"
);

#[cfg(feature = "webdom")]
pub use {wasm_bindgen::JsCast, web_sys as sys};

#[cfg(feature = "rsdom")]
use {rsdom::VirtNode, std::rc::Rc};

use futures::Stream;
use quick_xml::Writer as XmlWriter;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{prelude::*, Cursor},
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "rsdom")]
pub mod rsdom;
#[cfg(feature = "webdom")]
pub mod webdom;

pub mod event;
pub mod testing;

/// Returns the current document. Panics if called outside a valid context.
pub fn document() -> Document {
    #[cfg(feature = "rsdom")]
    match illicit::get::<Document>() {
        Ok(d) => return d.clone(),
        _e => {
            #[cfg(not(feature = "webdom"))]
            {
                _e.unwrap()
            }
            #[cfg(feature = "webdom")]
            {
                concrete_document()
            }
        }
    }

    #[cfg(all(feature = "webdom", not(feature = "rsdom")))]
    concrete_document()
}

#[cfg(feature = "webdom")]
fn concrete_document() -> Document {
    Document::Concrete(
        sys::window()
            .expect("must run from within a `window`")
            .document()
            .expect("must run from within a `window` with a valid `document`"),
    )
}

/// A value which implements a subset of the web's document object model.
pub trait Dom: Sized {
    /// The type returned by `query_selector_all`.
    type Nodes: IntoIterator<Item = Self>;

    /// The type returned in batches by [`Dom::Observer`].
    type MutationRecord;

    /// The type returned by `observe`.
    type Observer: Stream<Item = Vec<Self::MutationRecord>> + Unpin;

    /// Write this value as XML via the provided writer. Consider using
    /// [Dom::outer_html] or [Dom::pretty_outer_html] unless you need the
    /// performance.
    fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>);

    /// Returns a string of serialized XML without newlines or indentation.
    fn outer_html(&self) -> String {
        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        {
            let mut writer = XmlWriter::new(&mut buf);
            self.write_xml(&mut writer);
        }
        String::from_utf8(buf.into_inner()).unwrap()
    }

    /// Returns a string of "prettified" serialized XML with the provided
    /// indentation.
    fn pretty_outer_html(&self, indent: usize) -> String {
        let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        {
            let mut writer = XmlWriter::new_with_indent(&mut buf, b' ', indent);
            self.write_xml(&mut writer);
        }
        String::from_utf8(buf.into_inner()).unwrap()
    }

    /// Get an attribute from this DOM node.
    fn get_attribute(&self, name: &str) -> Option<String>;

    /// Set an attribute on this DOM node.
    fn set_attribute(&self, name: &str, value: &str);

    /// Ensure the provided attribute has been removed from this DOM node.
    fn remove_attribute(&self, name: &str);

    /// Returns the next child of this node's parent after this node itself.
    fn next_sibling(&self) -> Option<Self>;

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self>;

    /// Adds a new child to the end of this node's children.
    fn append_child(&self, child: &Self);

    /// Replaces the provided child of this node with a new one.
    fn replace_child(&self, new_child: &Self, existing: &Self);

    /// Removes the provided child from this node.
    fn remove_child(&self, to_remove: &Self) -> Option<Self>;

    /// Represents the "rendered" text content of a node and its descendants. It
    /// approximates the text the user would get if they highlighted the
    /// contents of the element with the cursor and then copied it to the
    /// clipboard.
    fn get_inner_text(&self) -> String;

    /// Synchronously invokes the affected EventListeners in the appropriate
    /// order. The normal event processing rules (including the capturing
    /// and optional bubbling phase) also apply to events dispatched
    /// manually with `dispatchEvent()`.
    fn dispatch<E: event::Event>(&self, event: E);

    /// Returns the first descendant of `self` which matches the specified
    /// [selectors].
    ///
    /// [selectors]: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors
    fn query_selector(&self, selectors: &str) -> Option<Self>;

    /// Returns a static (not live) Vec of descendents of `self` which match the
    /// specified [selectors].
    ///
    /// [selectors]: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors
    fn query_selector_all(&self, selectors: &str) -> Self::Nodes;

    /// Return a stream of mutations related to the subtree under this node.
    fn observe_mutations(&self) -> Self::Observer;
}

/// The current document.
#[derive(Clone)]
pub enum Document {
    /// A handle to a concrete document.
    #[cfg(feature = "webdom")]
    Concrete(sys::Document),

    /// A virtual environment's document.
    #[cfg(feature = "rsdom")]
    Virtual {
        /// The virtual document's head.
        head: Rc<VirtNode>,
        /// The virtual document's body.
        body: Rc<VirtNode>,
    },
}

impl Document {
    /// Create a new document for virtual rendering, inside or outside of the
    /// browser.
    #[cfg(feature = "rsdom")]
    pub fn new_virtual() -> Self {
        Document::Virtual {
            head: VirtNode::create_element("head"),
            body: VirtNode::create_element("body"),
        }
    }

    /// Return this document's head.
    pub fn head(&self) -> Node {
        match self {
            #[cfg(feature = "webdom")]
            Document::Concrete(d) => {
                let head = d.head().unwrap();
                let node: &sys::Node = head.as_ref();
                Node::Concrete(node.clone())
            }
            #[cfg(feature = "rsdom")]
            Document::Virtual { head, .. } => Node::Virtual(head.clone()),
        }
    }

    /// Return this document's body.
    pub fn body(&self) -> Node {
        match self {
            #[cfg(feature = "webdom")]
            Document::Concrete(d) => {
                let body = d.body().unwrap();
                let node: &sys::Node = body.as_ref();
                Node::Concrete(node.clone())
            }
            #[cfg(feature = "rsdom")]
            Document::Virtual { body, .. } => Node::Virtual(body.clone()),
        }
    }

    /// Create a new element in this document.
    pub fn create_element(&self, ty: &str) -> Node {
        match self {
            #[cfg(feature = "webdom")]
            Document::Concrete(d) => {
                let elem = d.create_element(ty).unwrap();
                let node: &sys::Node = elem.as_ref();
                Node::Concrete(node.clone())
            }
            #[cfg(feature = "rsdom")]
            Document::Virtual { .. } => Node::Virtual(VirtNode::create_element(ty)),
        }
    }

    /// Create a new text node in this document.
    pub fn create_text_node(&self, contents: &str) -> Node {
        match self {
            #[cfg(feature = "webdom")]
            Document::Concrete(d) => {
                let text = d.create_text_node(contents);
                let node: &sys::Node = text.as_ref();
                Node::Concrete(node.clone())
            }
            #[cfg(feature = "rsdom")]
            Document::Virtual { .. } => Node::Virtual(VirtNode::create_text_node(contents)),
        }
    }
}

impl Debug for Document {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        <Node as Debug>::fmt(&self.head(), f)?;
        <Node as Debug>::fmt(&self.body(), f)
    }
}

/// A `Node` in the augmented DOM.
#[derive(Clone)]
pub enum Node {
    /// A handle to a concrete DOM node running in the browser.
    #[cfg(feature = "webdom")]
    Concrete(sys::Node),

    /// A handle to a "virtual" DOM node, emulating the web in memory. While
    /// this implementation lacks many features, it can run on any target
    /// that Rust supports.
    #[cfg(feature = "rsdom")]
    Virtual(Rc<VirtNode>),
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let s = if f.alternate() { self.pretty_outer_html(4) } else { self.outer_html() };
        f.write_str(&s)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(&self.pretty_outer_html(2))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            #[cfg(feature = "webdom")]
            (Node::Concrete(s), Node::Concrete(o)) => s.is_same_node(Some(o)),

            #[cfg(feature = "rsdom")]
            (Node::Virtual(s), Node::Virtual(o)) => Rc::ptr_eq(s, o),

            #[cfg(all(feature = "webdom", feature = "rsdom"))]
            _ => unreachable!("if moxie-dom is comparing two different types of nodes...uh-oh."),
        }
    }
}

impl Dom for Node {
    type MutationRecord = MutationRecord;
    type Nodes = Vec<Self>;
    type Observer = MutationObserver;

    fn write_xml<W: Write>(&self, writer: &mut XmlWriter<W>) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                n.write_xml(writer);
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.write_xml(writer);
            }
        }
    }

    fn first_child(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::first_child(n).map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.first_child().map(Node::Virtual),
        }
    }

    fn append_child(&self, child: &Self) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::append_child(n, child.expect_concrete());
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.append_child(child.expect_virtual());
            }
        }
    }

    fn next_sibling(&self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::next_sibling(n).map(Node::Concrete),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.next_sibling().map(Node::Virtual),
        }
    }

    fn remove_child(&self, to_remove: &Self) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::remove_child(n, to_remove.expect_concrete()).map(Node::Concrete)
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.remove_child(to_remove.expect_virtual()).map(Node::Virtual),
        }
    }

    fn replace_child(&self, new_child: &Self, existing: &Self) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => {
                <sys::Node as Dom>::replace_child(
                    n,
                    new_child.expect_concrete(),
                    existing.expect_concrete(),
                );
            }

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.replace_child(new_child.expect_virtual(), existing.expect_virtual());
            }
        }
    }

    fn get_attribute(&self, name: &str) -> Option<String> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::get_attribute(n, name),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => <Rc<VirtNode> as Dom>::get_attribute(n, name),
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::set_attribute(n, name, value),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.set_attribute(name, value),
        }
    }

    fn remove_attribute(&self, name: &str) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::remove_attribute(n, name),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.remove_attribute(name),
        }
    }

    fn get_inner_text(&self) -> String {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::get_inner_text(n),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => <Rc<VirtNode> as Dom>::get_inner_text(n),
        }
    }

    fn dispatch<E: event::Event>(&self, event: E) {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => <sys::Node as Dom>::dispatch(n, event),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => <Rc<VirtNode> as Dom>::dispatch(n, event),
        }
    }

    fn query_selector(&self, selectors: &str) -> Option<Self> {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => n.query_selector(selectors).map(Node::Concrete),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => n.query_selector(selectors).map(Node::Virtual),
        }
    }

    fn query_selector_all(&self, selectors: &str) -> Self::Nodes {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => n.query_selector_all(selectors).map(Node::Concrete).collect(),
            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => {
                n.query_selector_all(selectors).into_iter().map(Node::Virtual).collect()
            }
        }
    }

    fn observe_mutations(&self) -> Self::Observer {
        match self {
            #[cfg(feature = "webdom")]
            Node::Concrete(n) => MutationObserver::Concrete(n.observe_mutations()),

            #[cfg(feature = "rsdom")]
            Node::Virtual(n) => MutationObserver::Virtual(n.observe_mutations()),
        }
    }
}

/// Wraps streams of mutation events from a given DOM backend.
pub enum MutationObserver {
    /// Results from a MutationObserver.
    #[cfg(feature = "webdom")]
    Concrete(webdom::Mutations),

    /// A stream of mutations from the virtual backend.
    #[cfg(feature = "rsdom")]
    Virtual(futures::channel::mpsc::UnboundedReceiver<Vec<rsdom::Mutation>>),
}

impl Stream for MutationObserver {
    type Item = Vec<MutationRecord>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            #[cfg(feature = "webdom")]
            MutationObserver::Concrete(mutations) => {
                futures::pin_mut!(mutations);
                let next = futures::ready!(mutations.poll_next(cx));
                let batch = next.map(|n| n.into_iter().map(MutationRecord::Concrete).collect());
                Poll::Ready(batch)
            }

            #[cfg(feature = "rsdom")]
            MutationObserver::Virtual(mutations) => {
                futures::pin_mut!(mutations);
                let next = futures::ready!(mutations.poll_next(cx));
                let batch = next.map(|n| n.into_iter().map(MutationRecord::Virtual).collect());
                Poll::Ready(batch)
            }
        }
    }
}

/// Wraps individual mutation records from a given DOM backend.
pub enum MutationRecord {
    /// A mutation record from the web backend.
    #[cfg(feature = "webdom")]
    Concrete(sys::MutationRecord),

    /// A mutation record from the virtual backend.
    #[cfg(feature = "rsdom")]
    Virtual(rsdom::Mutation),
}

#[cfg(test)]
mod tests {
    use super::{
        testing::{Query, TargetExt},
        *,
    };
    use std::mem::forget as cleanup_with_test;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    fn example_dom() -> Node {
        let document = document();
        let div = document.create_element("div");

        let label = document.create_element("label");
        label.set_attribute("for", "username");
        label.append_child(&document.create_text_node("Username"));
        div.append_child(&label);

        let input = document.create_element("input");
        input.set_attribute("id", "username");
        div.append_child(&input);

        let button = document.create_element("button");
        button.append_child(&document.create_text_node("Print Username"));
        div.append_child(&button);

        let container_for_callback = div.clone();
        let onclick = event::EventHandle::new(&button, move |_: event::Click| {
            // on a click, add this dom node to the parent in a callback
            let (input, document) = (input.clone(), document.clone());
            let div = container_for_callback.clone();
            let cb = move || {
                let printed_name_container = document.create_element("div");
                printed_name_container.set_attribute("data-testid", "printed-username");
                let input_text = document.create_text_node(&input.get_attribute("value").unwrap());
                printed_name_container.append_child(&input_text);
                div.append_child(&printed_name_container);
            };

            // fire the callback on a timer
            let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut()>);
            let empty_args = js_sys::Array::new();
            sys::window()
                .expect("must be able to retrieve window")
                .set_timeout_with_callback_and_timeout_and_arguments(
                    cb.as_ref().unchecked_ref(),
                    500,
                    &empty_args,
                )
                .unwrap();
            cleanup_with_test(cb);
        });
        cleanup_with_test(onclick);

        div
    }

    #[wasm_bindgen_test]
    async fn basic_matchers() {
        let container = example_dom();

        let ada = "Ada Lovelace";
        let input = container.find().by_label_text("Username").one().unwrap();
        input.set_attribute("value", ada);

        container.find().by_text("Print Username").one().unwrap().click();
        let printed = container.find().by_test_id("printed-username").until().one().await.unwrap();

        assert_eq!(printed.get_inner_text(), ada);

        let container_html = container.to_string();
        let expected = "<div>
  <label for=\"username\">Username</label>
  <input id=\"username\" value=\"Ada Lovelace\">
  </input>
  <button>Print Username</button>
  <div data-testid=\"printed-username\">Ada Lovelace</div>
</div>";

        assert_eq!(container_html, expected);
    }
}
