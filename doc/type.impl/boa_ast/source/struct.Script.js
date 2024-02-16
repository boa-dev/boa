(function() {var type_impls = {
"boa_ast":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#23-47\">source</a><a href=\"#impl-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#26-28\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_ast/source/struct.Script.html#tymethod.new\" class=\"fn\">new</a>(statements: <a class=\"struct\" href=\"boa_ast/statement_list/struct.StatementList.html\" title=\"struct boa_ast::statement_list::StatementList\">StatementList</a>) -&gt; Self</h4></section></summary><div class=\"docblock\"><p>Creates a new <code>ScriptNode</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.statements\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#32-34\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_ast/source/struct.Script.html#tymethod.statements\" class=\"fn\">statements</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"boa_ast/statement_list/struct.StatementList.html\" title=\"struct boa_ast::statement_list::StatementList\">StatementList</a></h4></section></summary><div class=\"docblock\"><p>Gets the list of statements of this <code>ScriptNode</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.statements_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#37-39\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_ast/source/struct.Script.html#tymethod.statements_mut\" class=\"fn\">statements_mut</a>(&amp;mut self) -&gt; &amp;mut <a class=\"struct\" href=\"boa_ast/statement_list/struct.StatementList.html\" title=\"struct boa_ast::statement_list::StatementList\">StatementList</a></h4></section></summary><div class=\"docblock\"><p>Gets a mutable reference to the list of statements of this <code>ScriptNode</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.strict\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#44-46\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_ast/source/struct.Script.html#tymethod.strict\" class=\"fn\">strict</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Gets the strict mode.</p>\n</div></details></div></details>",0,"boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Serialize-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#16\">source</a><a href=\"#impl-Serialize-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.serialize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#16\">source</a><a href=\"#method.serialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serialize.html#tymethod.serialize\" class=\"fn\">serialize</a>&lt;__S&gt;(&amp;self, __serializer: __S) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;__S::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serializer.html#associatedtype.Ok\" title=\"type serde::ser::Serializer::Ok\">Ok</a>, __S::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serializer.html#associatedtype.Error\" title=\"type serde::ser::Serializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __S: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>,</div></h4></section></summary><div class='docblock'>Serialize this value into the given Serde serializer. <a href=\"https://docs.rs/serde/1.0.196/serde/ser/trait.Serialize.html#tymethod.serialize\">Read more</a></div></details></div></details>","Serialize","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#impl-Debug-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Arbitrary%3C'arbitrary%3E-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#17\">source</a><a href=\"#impl-Arbitrary%3C'arbitrary%3E-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'arbitrary&gt; Arbitrary&lt;'arbitrary&gt; for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.arbitrary\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#17\">source</a><a href=\"#method.arbitrary\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">arbitrary</a>(u: &amp;mut Unstructured&lt;'arbitrary&gt;) -&gt; Result&lt;Self&gt;</h4></section></summary><div class='docblock'>Generate an arbitrary value of <code>Self</code> from the given unstructured data. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.arbitrary_take_rest\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#17\">source</a><a href=\"#method.arbitrary_take_rest\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">arbitrary_take_rest</a>(u: Unstructured&lt;'arbitrary&gt;) -&gt; Result&lt;Self&gt;</h4></section></summary><div class='docblock'>Generate an arbitrary value of <code>Self</code> from the entirety of the given\nunstructured data. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.size_hint\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#17\">source</a><a href=\"#method.size_hint\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">size_hint</a>(depth: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.usize.html\">usize</a>) -&gt; (<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.usize.html\">usize</a>, <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.usize.html\">usize</a>&gt;)</h4></section></summary><div class='docblock'>Get a size hint for how many bytes out of an <code>Unstructured</code> this type\nneeds to construct itself. <a>Read more</a></div></details></div></details>","Arbitrary<'arbitrary>","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-VisitWith-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#49-63\">source</a><a href=\"#impl-VisitWith-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_ast/visitor/trait.VisitWith.html\" title=\"trait boa_ast::visitor::VisitWith\">VisitWith</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.visit_with\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#50-55\">source</a><a href=\"#method.visit_with\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_ast/visitor/trait.VisitWith.html#tymethod.visit_with\" class=\"fn\">visit_with</a>&lt;'a, V&gt;(&amp;'a self, visitor: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.reference.html\">&amp;mut V</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/control_flow/enum.ControlFlow.html\" title=\"enum core::ops::control_flow::ControlFlow\">ControlFlow</a>&lt;V::<a class=\"associatedtype\" href=\"boa_ast/visitor/trait.Visitor.html#associatedtype.BreakTy\" title=\"type boa_ast::visitor::Visitor::BreakTy\">BreakTy</a>&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"boa_ast/visitor/trait.Visitor.html\" title=\"trait boa_ast::visitor::Visitor\">Visitor</a>&lt;'a&gt;,</div></h4></section></summary><div class='docblock'>Visit this node with the provided visitor.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.visit_with_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#57-62\">source</a><a href=\"#method.visit_with_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_ast/visitor/trait.VisitWith.html#tymethod.visit_with_mut\" class=\"fn\">visit_with_mut</a>&lt;'a, V&gt;(\n    &amp;'a mut self,\n    visitor: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.reference.html\">&amp;mut V</a>\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/ops/control_flow/enum.ControlFlow.html\" title=\"enum core::ops::control_flow::ControlFlow\">ControlFlow</a>&lt;V::<a class=\"associatedtype\" href=\"boa_ast/visitor/trait.VisitorMut.html#associatedtype.BreakTy\" title=\"type boa_ast::visitor::VisitorMut::BreakTy\">BreakTy</a>&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"boa_ast/visitor/trait.VisitorMut.html\" title=\"trait boa_ast::visitor::VisitorMut\">VisitorMut</a>&lt;'a&gt;,</div></h4></section></summary><div class='docblock'>Visit this node with the provided visitor mutably, allowing the visitor to modify private\nfields.</div></details></div></details>","VisitWith","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#impl-Clone-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.76.0/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deserialize%3C'de%3E-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#16\">source</a><a href=\"#impl-Deserialize%3C'de%3E-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.196/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.deserialize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#16\">source</a><a href=\"#method.deserialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.196/serde/de/trait.Deserialize.html#tymethod.deserialize\" class=\"fn\">deserialize</a>&lt;__D&gt;(__deserializer: __D) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;Self, __D::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.196/serde/de/trait.Deserializer.html#associatedtype.Error\" title=\"type serde::de::Deserializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __D: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.196/serde/de/trait.Deserializer.html\" title=\"trait serde::de::Deserializer\">Deserializer</a>&lt;'de&gt;,</div></h4></section></summary><div class='docblock'>Deserialize this value from the given Serde deserializer. <a href=\"https://docs.rs/serde/1.0.196/serde/de/trait.Deserialize.html#tymethod.deserialize\">Read more</a></div></details></div></details>","Deserialize<'de>","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#impl-PartialEq-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>self</code> and <code>other</code> values to be equal, and is used\nby <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/core/cmp.rs.html#242\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>!=</code>. The default implementation is almost always\nsufficient, and should not be overridden without very good reason.</div></details></div></details>","PartialEq","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ToIndentedString-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#65-69\">source</a><a href=\"#impl-ToIndentedString-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_interner/trait.ToIndentedString.html\" title=\"trait boa_interner::ToIndentedString\">ToIndentedString</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_indented_string\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#66-68\">source</a><a href=\"#method.to_indented_string\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_interner/trait.ToIndentedString.html#tymethod.to_indented_string\" class=\"fn\">to_indented_string</a>(&amp;self, interner: &amp;<a class=\"struct\" href=\"boa_interner/struct.Interner.html\" title=\"struct boa_interner::Interner\">Interner</a>, indentation: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.usize.html\">usize</a>) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a></h4></section></summary><div class='docblock'>Converts the element to a string using an interner, with the given indentation.</div></details></div></details>","ToIndentedString","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<section id=\"impl-StructuralPartialEq-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#impl-StructuralPartialEq-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section>","StructuralPartialEq","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Script\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#impl-Default-for-Script\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_ast/source.rs.html#18\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; <a class=\"struct\" href=\"boa_ast/source/struct.Script.html\" title=\"struct boa_ast::source::Script\">Script</a></h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/1.76.0/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","boa_ast::function::class::StaticBlockBody","boa_ast::function::FunctionBody"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()