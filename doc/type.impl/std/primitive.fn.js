(function() {var type_impls = {
"boa_engine":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Finalize-for-fn(A,+B,+C)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Finalize-for-fn(A,+B,+C)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A, B, C&gt; <a class=\"trait\" href=\"boa_engine/trait.Finalize.html\" title=\"trait boa_engine::Finalize\">Finalize</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A, _: B, _: C) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.finalize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#49\">source</a><a href=\"#method.finalize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Finalize.html#method.finalize\" class=\"fn\">finalize</a>(&amp;self)</h4></section></summary><div class='docblock'>Cleanup logic for a type.</div></details></div></details>","Finalize","boa_engine::native_function::NativeFunctionPointer"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Trace-for-fn(A,+B,+C)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Trace-for-fn(A,+B,+C)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A, B, C&gt; <a class=\"trait\" href=\"boa_engine/trait.Trace.html\" title=\"trait boa_engine::Trace\">Trace</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A, _: B, _: C) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace\" class=\"fn\">trace</a>(&amp;self, _tracer: &amp;mut <a class=\"struct\" href=\"boa_gc/trace/struct.Tracer.html\" title=\"struct boa_gc::trace::Tracer\">Tracer</a>)</h4></section></summary><div class='docblock'>Marks all contained <code>Gc</code>s. <a href=\"boa_engine/trait.Trace.html#tymethod.trace\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace_non_roots\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace_non_roots\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\" class=\"fn\">trace_non_roots</a>(&amp;self)</h4></section></summary><div class='docblock'>Trace handles located in GC heap, and mark them as non root. <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.run_finalizer\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.run_finalizer\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Trace.html#tymethod.run_finalizer\" class=\"fn\">run_finalizer</a>(&amp;self)</h4></section></summary><div class='docblock'>Runs <a href=\"boa_engine/trait.Finalize.html#method.finalize\" title=\"method boa_engine::Finalize::finalize\"><code>Finalize::finalize</code></a> on this object and all\ncontained subobjects.</div></details></div></details>","Trace","boa_engine::native_function::NativeFunctionPointer"],["<section id=\"impl-JsData-for-fn(A,+B,+C)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/datatypes.rs.html#151-165\">source</a><a href=\"#impl-JsData-for-fn(A,+B,+C)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A, B, C&gt; <a class=\"trait\" href=\"boa_engine/object/datatypes/trait.JsData.html\" title=\"trait boa_engine::object::datatypes::JsData\">JsData</a> for <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A, _: B, _: C) -&gt; Ret</h3></section>","JsData","boa_engine::native_function::NativeFunctionPointer"]],
"boa_gc":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Finalize-for-unsafe+fn(A,+B)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Finalize-for-unsafe+fn(A,+B)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A, B&gt; <a class=\"trait\" href=\"boa_gc/trace/trait.Finalize.html\" title=\"trait boa_gc::trace::Finalize\">Finalize</a> for unsafe <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A, _: B) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.finalize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#49\">source</a><a href=\"#method.finalize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_gc/trace/trait.Finalize.html#method.finalize\" class=\"fn\">finalize</a>(&amp;self)</h4></section></summary><div class='docblock'>Cleanup logic for a type.</div></details></div></details>","Finalize","boa_gc::internals::vtable::TraceFn"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Finalize-for-unsafe+fn(A)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Finalize-for-unsafe+fn(A)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A&gt; <a class=\"trait\" href=\"boa_gc/trace/trait.Finalize.html\" title=\"trait boa_gc::trace::Finalize\">Finalize</a> for unsafe <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.finalize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#49\">source</a><a href=\"#method.finalize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_gc/trace/trait.Finalize.html#method.finalize\" class=\"fn\">finalize</a>(&amp;self)</h4></section></summary><div class='docblock'>Cleanup logic for a type.</div></details></div></details>","Finalize","boa_gc::internals::vtable::TraceNonRootsFn","boa_gc::internals::vtable::RunFinalizerFn","boa_gc::internals::vtable::DropFn"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Trace-for-unsafe+fn(A)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Trace-for-unsafe+fn(A)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A&gt; <a class=\"trait\" href=\"boa_gc/trace/trait.Trace.html\" title=\"trait boa_gc::trace::Trace\">Trace</a> for unsafe <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace\" class=\"fn\">trace</a>(&amp;self, _tracer: &amp;mut Tracer)</h4></section></summary><div class='docblock'>Marks all contained <code>Gc</code>s. <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace_non_roots\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace_non_roots\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace_non_roots\" class=\"fn\">trace_non_roots</a>(&amp;self)</h4></section></summary><div class='docblock'>Trace handles located in GC heap, and mark them as non root. <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace_non_roots\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.run_finalizer\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.run_finalizer\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.run_finalizer\" class=\"fn\">run_finalizer</a>(&amp;self)</h4></section></summary><div class='docblock'>Runs <a href=\"boa_gc/trace/trait.Finalize.html#method.finalize\" title=\"method boa_gc::trace::Finalize::finalize\"><code>Finalize::finalize</code></a> on this object and all\ncontained subobjects.</div></details></div></details>","Trace","boa_gc::internals::vtable::TraceNonRootsFn","boa_gc::internals::vtable::RunFinalizerFn","boa_gc::internals::vtable::DropFn"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Trace-for-unsafe+fn(A,+B)+-%3E+Ret\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#impl-Trace-for-unsafe+fn(A,+B)+-%3E+Ret\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;Ret, A, B&gt; <a class=\"trait\" href=\"boa_gc/trace/trait.Trace.html\" title=\"trait boa_gc::trace::Trace\">Trace</a> for unsafe <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.fn.html\">fn</a>(_: A, _: B) -&gt; Ret</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace\" class=\"fn\">trace</a>(&amp;self, _tracer: &amp;mut Tracer)</h4></section></summary><div class='docblock'>Marks all contained <code>Gc</code>s. <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace_non_roots\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.trace_non_roots\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace_non_roots\" class=\"fn\">trace_non_roots</a>(&amp;self)</h4></section></summary><div class='docblock'>Trace handles located in GC heap, and mark them as non root. <a href=\"boa_gc/trace/trait.Trace.html#tymethod.trace_non_roots\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.run_finalizer\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_gc/trace.rs.html#279-293\">source</a><a href=\"#method.run_finalizer\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_gc/trace/trait.Trace.html#tymethod.run_finalizer\" class=\"fn\">run_finalizer</a>(&amp;self)</h4></section></summary><div class='docblock'>Runs <a href=\"boa_gc/trace/trait.Finalize.html#method.finalize\" title=\"method boa_gc::trace::Finalize::finalize\"><code>Finalize::finalize</code></a> on this object and all\ncontained subobjects.</div></details></div></details>","Trace","boa_gc::internals::vtable::TraceFn"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()