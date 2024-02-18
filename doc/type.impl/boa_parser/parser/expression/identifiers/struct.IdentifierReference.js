(function() {var type_impls = {
"boa_parser":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-IdentifierReference\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#30-43\">source</a><a href=\"#impl-IdentifierReference\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#33-42\">source</a><h4 class=\"code-header\">pub(in <a class=\"mod\" href=\"boa_parser/parser/index.html\" title=\"mod boa_parser::parser\">parser</a>) fn <a href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html#tymethod.new\" class=\"fn\">new</a>&lt;Y, A&gt;(\n    allow_yield: Y,\n    allow_await: A\n) -&gt; Self<div class=\"where\">where\n    Y: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"boa_parser/parser/struct.AllowYield.html\" title=\"struct boa_parser::parser::AllowYield\">AllowYield</a>&gt;,\n    A: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"boa_parser/parser/struct.AllowAwait.html\" title=\"struct boa_parser::parser::AllowAwait\">AllowAwait</a>&gt;,</div></h4></section></summary><div class=\"docblock\"><p>Creates a new <code>IdentifierReference</code> parser.</p>\n</div></details></div></details>",0,"boa_parser::parser::expression::identifiers::LabelIdentifier"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-IdentifierReference\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#24\">source</a><a href=\"#impl-Debug-for-IdentifierReference\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#24\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","boa_parser::parser::expression::identifiers::LabelIdentifier"],["<section id=\"impl-Copy-for-IdentifierReference\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#24\">source</a><a href=\"#impl-Copy-for-IdentifierReference\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a></h3></section>","Copy","boa_parser::parser::expression::identifiers::LabelIdentifier"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-IdentifierReference\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#24\">source</a><a href=\"#impl-Clone-for-IdentifierReference\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#24\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a></h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","boa_parser::parser::expression::identifiers::LabelIdentifier"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TokenParser%3CR%3E-for-IdentifierReference\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#45-70\">source</a><a href=\"#impl-TokenParser%3CR%3E-for-IdentifierReference\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"trait\" href=\"boa_parser/parser/trait.TokenParser.html\" title=\"trait boa_parser::parser::TokenParser\">TokenParser</a>&lt;R&gt; for <a class=\"struct\" href=\"boa_parser/parser/expression/identifiers/struct.IdentifierReference.html\" title=\"struct boa_parser::parser::expression::identifiers::IdentifierReference\">IdentifierReference</a><div class=\"where\">where\n    R: <a class=\"trait\" href=\"boa_parser/source/trait.ReadChar.html\" title=\"trait boa_parser::source::ReadChar\">ReadChar</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"boa_parser/parser/trait.TokenParser.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = <a class=\"struct\" href=\"boa_ast/expression/identifier/struct.Identifier.html\" title=\"struct boa_ast::expression::identifier::Identifier\">Identifier</a></h4></section></summary><div class='docblock'>Output type for the parser.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.parse\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_parser/parser/expression/identifiers.rs.html#51-69\">source</a><a href=\"#method.parse\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_parser/parser/trait.TokenParser.html#tymethod.parse\" class=\"fn\">parse</a>(\n    self,\n    cursor: &amp;mut <a class=\"struct\" href=\"boa_parser/parser/cursor/struct.Cursor.html\" title=\"struct boa_parser::parser::cursor::Cursor\">Cursor</a>&lt;R&gt;,\n    interner: &amp;mut <a class=\"struct\" href=\"boa_interner/struct.Interner.html\" title=\"struct boa_interner::Interner\">Interner</a>\n) -&gt; <a class=\"type\" href=\"boa_parser/error/type.ParseResult.html\" title=\"type boa_parser::error::ParseResult\">ParseResult</a>&lt;Self::<a class=\"associatedtype\" href=\"boa_parser/parser/trait.TokenParser.html#associatedtype.Output\" title=\"type boa_parser::parser::TokenParser::Output\">Output</a>&gt;</h4></section></summary><div class='docblock'>Parses the token stream using the current parser. <a href=\"boa_parser/parser/trait.TokenParser.html#tymethod.parse\">Read more</a></div></details></div></details>","TokenParser<R>","boa_parser::parser::expression::identifiers::LabelIdentifier"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()