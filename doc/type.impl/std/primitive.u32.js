(function() {var type_impls = {
"boa_engine":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Finalize-for-u32\" class=\"impl\"><a href=\"#impl-Finalize-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/trait.Finalize.html\" title=\"trait boa_engine::Finalize\">Finalize</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.finalize\" class=\"method trait-impl\"><a href=\"#method.finalize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Finalize.html#method.finalize\" class=\"fn\">finalize</a>(&amp;self)</h4></section></summary><div class='docblock'>Cleanup logic for a type.</div></details></div></details>","Finalize","boa_engine::object::shape::slot::SlotIndex"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Trace-for-u32\" class=\"impl\"><a href=\"#impl-Trace-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/trait.Trace.html\" title=\"trait boa_engine::Trace\">Trace</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace\" class=\"method trait-impl\"><a href=\"#method.trace\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace\" class=\"fn\">trace</a>(&amp;self, _tracer: &amp;mut Tracer)</h4></section></summary><div class='docblock'>Marks all contained <code>Gc</code>s. <a href=\"boa_engine/trait.Trace.html#tymethod.trace\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace_non_roots\" class=\"method trait-impl\"><a href=\"#method.trace_non_roots\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\" class=\"fn\">trace_non_roots</a>(&amp;self)</h4></section></summary><div class='docblock'>Trace handles located in GC heap, and mark them as non root. <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.run_finalizer\" class=\"method trait-impl\"><a href=\"#method.run_finalizer\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Trace.html#tymethod.run_finalizer\" class=\"fn\">run_finalizer</a>(&amp;self)</h4></section></summary><div class='docblock'>Runs <a href=\"boa_engine/trait.Finalize.html#method.finalize\" title=\"method boa_engine::Finalize::finalize\"><code>Finalize::finalize</code></a> on this object and all\ncontained subobjects.</div></details></div></details>","Trace","boa_engine::object::shape::slot::SlotIndex"],["<section id=\"impl-Readable-for-u32\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/vm/code_block.rs.html#41\">source</a><a href=\"#impl-Readable-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/vm/code_block/trait.Readable.html\" title=\"trait boa_engine::vm::code_block::Readable\">Readable</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section>","Readable","boa_engine::object::shape::slot::SlotIndex"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Element-for-u32\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#impl-Element-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/builtins/typed_array/element/trait.Element.html\" title=\"trait boa_engine::builtins::typed_array::element::Element\">Element</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Atomic\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Atomic\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#associatedtype.Atomic\" class=\"associatedtype\">Atomic</a> = AtomicU32</h4></section></summary><div class='docblock'>The atomic type used for shared array buffers.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_js_value\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.from_js_value\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.from_js_value\" class=\"fn\">from_js_value</a>(value: &amp;<a class=\"enum\" href=\"boa_engine/value/enum.JsValue.html\" title=\"enum boa_engine::value::JsValue\">JsValue</a>, context: &amp;mut <a class=\"struct\" href=\"boa_engine/context/struct.Context.html\" title=\"struct boa_engine::context::Context\">Context</a>) -&gt; <a class=\"type\" href=\"boa_engine/type.JsResult.html\" title=\"type boa_engine::JsResult\">JsResult</a>&lt;Self&gt;</h4></section></summary><div class='docblock'>Converts a <code>JsValue</code> into the native element <code>Self</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.from_plain\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.from_plain\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.from_plain\" class=\"fn\">from_plain</a>(plain: &lt;Self::<a class=\"associatedtype\" href=\"boa_engine/builtins/typed_array/element/trait.Element.html#associatedtype.Atomic\" title=\"type boa_engine::builtins::typed_array::element::Element::Atomic\">Atomic</a> as <a class=\"trait\" href=\"boa_engine/builtins/typed_array/element/atomic/trait.Atomic.html\" title=\"trait boa_engine::builtins::typed_array::element::atomic::Atomic\">Atomic</a>&gt;::<a class=\"associatedtype\" href=\"boa_engine/builtins/typed_array/element/atomic/trait.Atomic.html#associatedtype.Plain\" title=\"type boa_engine::builtins::typed_array::element::atomic::Atomic::Plain\">Plain</a>) -&gt; Self</h4></section></summary><div class='docblock'>Converts from the plain type of an atomic to <code>Self</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_plain\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.to_plain\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.to_plain\" class=\"fn\">to_plain</a>(self) -&gt; &lt;Self::<a class=\"associatedtype\" href=\"boa_engine/builtins/typed_array/element/trait.Element.html#associatedtype.Atomic\" title=\"type boa_engine::builtins::typed_array::element::Element::Atomic\">Atomic</a> as <a class=\"trait\" href=\"boa_engine/builtins/typed_array/element/atomic/trait.Atomic.html\" title=\"trait boa_engine::builtins::typed_array::element::atomic::Atomic\">Atomic</a>&gt;::<a class=\"associatedtype\" href=\"boa_engine/builtins/typed_array/element/atomic/trait.Atomic.html#associatedtype.Plain\" title=\"type boa_engine::builtins::typed_array::element::atomic::Atomic::Plain\">Plain</a></h4></section></summary><div class='docblock'>Converts from <code>Self</code> to the plain type of an atomic.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_big_endian\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.to_big_endian\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.to_big_endian\" class=\"fn\">to_big_endian</a>(self) -&gt; Self</h4></section></summary><div class='docblock'>Gets the big endian representation of <code>Self</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.to_little_endian\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.to_little_endian\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.to_little_endian\" class=\"fn\">to_little_endian</a>(self) -&gt; Self</h4></section></summary><div class='docblock'>Gets the little endian representation of <code>Self</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.read\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.read\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.read\" class=\"fn\">read</a>(buffer: <a class=\"enum\" href=\"boa_engine/builtins/array_buffer/utils/enum.SliceRef.html\" title=\"enum boa_engine::builtins::array_buffer::utils::SliceRef\">SliceRef</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"boa_engine/builtins/typed_array/element/enum.ElementRef.html\" title=\"enum boa_engine::builtins::typed_array::element::ElementRef\">ElementRef</a>&lt;'_, Self&gt;</h4></section></summary><div class='docblock'>Reads <code>Self</code> from the <code>buffer</code>. <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.read\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.read_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/builtins/typed_array/element/mod.rs.html#318\">source</a><a href=\"#method.read_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.read_mut\" class=\"fn\">read_mut</a>(buffer: <a class=\"enum\" href=\"boa_engine/builtins/array_buffer/utils/enum.SliceRefMut.html\" title=\"enum boa_engine::builtins::array_buffer::utils::SliceRefMut\">SliceRefMut</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"boa_engine/builtins/typed_array/element/enum.ElementRefMut.html\" title=\"enum boa_engine::builtins::typed_array::element::ElementRefMut\">ElementRefMut</a>&lt;'_, Self&gt;</h4></section></summary><div class='docblock'>Writes the bytes of this element into <code>buffer</code>. <a href=\"boa_engine/builtins/typed_array/element/trait.Element.html#tymethod.read_mut\">Read more</a></div></details></div></details>","Element","boa_engine::object::shape::slot::SlotIndex"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TryFromJs-for-u32\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/value/conversions/try_from_js.rs.html#172-185\">source</a><a href=\"#impl-TryFromJs-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/value/conversions/try_from_js/trait.TryFromJs.html\" title=\"trait boa_engine::value::conversions::try_from_js::TryFromJs\">TryFromJs</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.try_from_js\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/value/conversions/try_from_js.rs.html#173-184\">source</a><a href=\"#method.try_from_js\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/value/conversions/try_from_js/trait.TryFromJs.html#tymethod.try_from_js\" class=\"fn\">try_from_js</a>(value: &amp;<a class=\"enum\" href=\"boa_engine/value/enum.JsValue.html\" title=\"enum boa_engine::value::JsValue\">JsValue</a>, _context: &amp;mut <a class=\"struct\" href=\"boa_engine/context/struct.Context.html\" title=\"struct boa_engine::context::Context\">Context</a>) -&gt; <a class=\"type\" href=\"boa_engine/type.JsResult.html\" title=\"type boa_engine::JsResult\">JsResult</a>&lt;Self&gt;</h4></section></summary><div class='docblock'>This function tries to convert a JavaScript value into <code>Self</code>.</div></details></div></details>","TryFromJs","boa_engine::object::shape::slot::SlotIndex"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-BytecodeConversion-for-u32\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/vm/opcode/mod.rs.html#282-291\">source</a><a href=\"#impl-BytecodeConversion-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/vm/opcode/trait.BytecodeConversion.html\" title=\"trait boa_engine::vm::opcode::BytecodeConversion\">BytecodeConversion</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section></summary><div class=\"impl-items\"><section id=\"method.to_bytecode\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/vm/opcode/mod.rs.html#283-285\">source</a><a href=\"#method.to_bytecode\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/vm/opcode/trait.BytecodeConversion.html#tymethod.to_bytecode\" class=\"fn\">to_bytecode</a>(&amp;self, bytes: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.75.0/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u8.html\">u8</a>&gt;)</h4></section><section id=\"method.from_bytecode\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/vm/opcode/mod.rs.html#286-290\">source</a><a href=\"#method.from_bytecode\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/vm/opcode/trait.BytecodeConversion.html#tymethod.from_bytecode\" class=\"fn\">from_bytecode</a>(\n    bytes: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u8.html\">u8</a>],\n    pc: &amp;mut <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.usize.html\">usize</a>,\n    _varying_kind: <a class=\"enum\" href=\"boa_engine/vm/opcode/enum.VaryingOperandKind.html\" title=\"enum boa_engine::vm::opcode::VaryingOperandKind\">VaryingOperandKind</a>\n) -&gt; Self</h4></section></div></details>","BytecodeConversion","boa_engine::object::shape::slot::SlotIndex"],["<section id=\"impl-JsData-for-u32\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/datatypes.rs.html#59-93\">source</a><a href=\"#impl-JsData-for-u32\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"boa_engine/object/datatypes/trait.JsData.html\" title=\"trait boa_engine::object::datatypes::JsData\">JsData</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.u32.html\">u32</a></h3></section>","JsData","boa_engine::object::shape::slot::SlotIndex"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()