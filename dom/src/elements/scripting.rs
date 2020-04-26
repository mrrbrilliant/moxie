//! Scripting

use crate::{
    interfaces::node::{sealed::Memoized, Node},
    memo_node::MemoNode,
    prelude::*,
};
use augdom::event;

element! {
    /// Use the [HTML `<canvas>` element][mdn] with either the [canvas scripting API][api] or the
    /// [WebGL API][gl] to draw graphics and animations.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas
    /// [api]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
    /// [gl]: https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API
    canvas -> Canvas
}

element! {
    /// The [HTML `<noscript>` element][mdn] defines a section of HTML to be inserted if a script
    /// type on the page is unsupported or if scripting is currently turned off in the browser.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript
    noscript -> NoScript
}

element! {
    /// The [HTML `<script>` element][mdn] is used to embed or reference executable code; this is
    /// typically used to embed or refer to JavaScript code.
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script
    script -> Script
}
