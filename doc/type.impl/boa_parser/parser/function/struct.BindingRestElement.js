(function() {var type_impls = {
"boa_parser":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-BindingRestElement\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#232-244\">source</a><a href=\"#impl-BindingRestElement\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#234-243\">source</a><h4 class=\"code-header\">fn <a href=\"boa_parser/parser/function/struct.BindingRestElement.html#tymethod.new\" class=\"fn\">new</a>&lt;Y, A&gt;(allow_yield: Y, allow_await: A) -&gt; Self<span class=\"where fmt-newline\">where\n    Y: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"boa_parser/parser/struct.AllowYield.html\" title=\"struct boa_parser::parser::AllowYield\">AllowYield</a>&gt;,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"boa_parser/parser/struct.AllowAwait.html\" title=\"struct boa_parser::parser::AllowAwait\">AllowAwait</a>&gt;,</span></h4></section></summary><div class=\"docblock\"><p>Creates a new <code>BindingRestElement</code> parser.</p>\n</div></details></div></details>",0,"boa_parser::parser::function::FunctionRestParameter"],["<section id=\"impl-Copy-for-BindingRestElement\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#226\">source</a><a href=\"#impl-Copy-for-BindingRestElement\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a></h3></section>","Copy","boa_parser::parser::function::FunctionRestParameter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-BindingRestElement\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#226\">source</a><a href=\"#impl-Clone-for-BindingRestElement\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#226\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a></h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.75.0/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.75.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","boa_parser::parser::function::FunctionRestParameter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TokenParser%3CR%3E-for-BindingRestElement\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#246-311\">source</a><a href=\"#impl-TokenParser%3CR%3E-for-BindingRestElement\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"trait\" href=\"boa_parser/parser/trait.TokenParser.html\" title=\"trait boa_parser::parser::TokenParser\">TokenParser</a>&lt;R&gt; for <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a><span class=\"where fmt-newline\">where\n    R: <a class=\"trait\" href=\"boa_parser/source/trait.ReadChar.html\" title=\"trait boa_parser::source::ReadChar\">ReadChar</a>,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"boa_parser/parser/trait.TokenParser.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = FormalParameter</h4></section></summary><div class='docblock'>Output type for the parser.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.parse\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#252-310\">source</a><a href=\"#method.parse\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_parser/parser/trait.TokenParser.html#tymethod.parse\" class=\"fn\">parse</a>(\n    self,\n    cursor: &amp;mut <a class=\"struct\" href=\"boa_parser/parser/cursor/struct.Cursor.html\" title=\"struct boa_parser::parser::cursor::Cursor\">Cursor</a>&lt;R&gt;,\n    interner: &amp;mut Interner\n) -&gt; <a class=\"type\" href=\"boa_parser/error/type.ParseResult.html\" title=\"type boa_parser::error::ParseResult\">ParseResult</a>&lt;Self::<a class=\"associatedtype\" href=\"boa_parser/parser/trait.TokenParser.html#associatedtype.Output\" title=\"type boa_parser::parser::TokenParser::Output\">Output</a>&gt;</h4></section></summary><div class='docblock'>Parses the token stream using the current parser. <a href=\"boa_parser/parser/trait.TokenParser.html#tymethod.parse\">Read more</a></div></details></div></details>","TokenParser<R>","boa_parser::parser::function::FunctionRestParameter"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-BindingRestElement\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#226\">source</a><a href=\"#impl-Debug-for-BindingRestElement\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"boa_parser/parser/function/struct.BindingRestElement.html\" title=\"struct boa_parser::parser::function::BindingRestElement\">BindingRestElement</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/function/mod.rs.html#226\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","boa_parser::parser::function::FunctionRestParameter"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()