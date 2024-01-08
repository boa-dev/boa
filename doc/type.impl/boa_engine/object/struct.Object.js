(function() {var type_impls = {
"boa_engine":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Object%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#242-337\">source</a><a href=\"#impl-Object%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.shape\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#245-247\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_engine/object/struct.Object.html#tymethod.shape\" class=\"fn\">shape</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"boa_engine/object/shape/struct.Shape.html\" title=\"struct boa_engine::object::shape::Shape\">Shape</a></h4></section></summary><div class=\"docblock\"><p>Returns the shape of the object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.data\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#252-254\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_engine/object/struct.Object.html#tymethod.data\" class=\"fn\">data</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.reference.html\">&amp;T</a></h4></section></summary><div class=\"docblock\"><p>Returns the data of the object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.prototype\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#259-261\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.prototype\" class=\"fn\">prototype</a>(&amp;self) -&gt; <a class=\"type\" href=\"boa_engine/object/type.JsPrototype.html\" title=\"type boa_engine::object::JsPrototype\">JsPrototype</a></h4></section></summary><div class=\"docblock\"><p>Gets the prototype instance of this object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_prototype\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#269-279\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.set_prototype\" class=\"fn\">set_prototype</a>&lt;O: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"type\" href=\"boa_engine/object/type.JsPrototype.html\" title=\"type boa_engine::object::JsPrototype\">JsPrototype</a>&gt;&gt;(&amp;mut self, prototype: O) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Sets the prototype instance of the object.</p>\n<p><a href=\"https://tc39.es/ecma262/#sec-invariants-of-the-essential-internal-methods\">More information</a></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.properties\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#284-286\">source</a><h4 class=\"code-header\">pub const fn <a href=\"boa_engine/object/struct.Object.html#tymethod.properties\" class=\"fn\">properties</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"boa_engine/object/property_map/struct.PropertyMap.html\" title=\"struct boa_engine::object::property_map::PropertyMap\">PropertyMap</a></h4></section></summary><div class=\"docblock\"><p>Returns the properties of the object.</p>\n</div></details><section id=\"method.properties_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#289-291\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.properties_mut\" class=\"fn\">properties_mut</a>(&amp;mut self) -&gt; &amp;mut <a class=\"struct\" href=\"boa_engine/object/property_map/struct.PropertyMap.html\" title=\"struct boa_engine::object::property_map::PropertyMap\">PropertyMap</a></h4></section><details class=\"toggle method-toggle\" open><summary><section id=\"method.insert\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#297-303\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.insert\" class=\"fn\">insert</a>&lt;K, P&gt;(&amp;mut self, key: K, property: P) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a><span class=\"where fmt-newline\">where\n    K: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"enum\" href=\"boa_engine/property/enum.PropertyKey.html\" title=\"enum boa_engine::property::PropertyKey\">PropertyKey</a>&gt;,\n    P: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"boa_engine/property/struct.PropertyDescriptor.html\" title=\"struct boa_engine::property::PropertyDescriptor\">PropertyDescriptor</a>&gt;,</span></h4></section></summary><div class=\"docblock\"><p>Inserts a field in the object <code>properties</code> without checking if it’s writable.</p>\n<p>If a field was already in the object with the same name, then <code>true</code> is returned\notherwise, <code>false</code> is returned.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.remove\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#309-311\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.remove\" class=\"fn\">remove</a>(&amp;mut self, key: &amp;<a class=\"enum\" href=\"boa_engine/property/enum.PropertyKey.html\" title=\"enum boa_engine::property::PropertyKey\">PropertyKey</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Helper function for property removal without checking if it’s configurable.</p>\n<p>Returns <code>true</code> if the property was removed, <code>false</code> otherwise.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.append_private_element\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#314-336\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.append_private_element\" class=\"fn\">append_private_element</a>(\n    &amp;mut self,\n    name: <a class=\"struct\" href=\"boa_engine/object/struct.PrivateName.html\" title=\"struct boa_engine::object::PrivateName\">PrivateName</a>,\n    element: <a class=\"enum\" href=\"boa_engine/object/enum.PrivateElement.html\" title=\"enum boa_engine::object::PrivateElement\">PrivateElement</a>\n)</h4></section></summary><div class=\"docblock\"><p>Append a private element to an object.</p>\n</div></details></div></details>",0,"boa_engine::object::jsobject::ErasedObject"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Object%3Cdyn+NativeObject%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#339-478\">source</a><a href=\"#impl-Object%3Cdyn+NativeObject%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;dyn <a class=\"trait\" href=\"boa_engine/object/trait.NativeObject.html\" title=\"trait boa_engine::object::NativeObject\">NativeObject</a>&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.is\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#342-344\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is\" class=\"fn\">is</a>&lt;T: <a class=\"trait\" href=\"boa_engine/object/trait.NativeObject.html\" title=\"trait boa_engine::object::NativeObject\">NativeObject</a>&gt;(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Return <code>true</code> if it is a native object and the native type is <code>T</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.downcast_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#349-351\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.downcast_ref\" class=\"fn\">downcast_ref</a>&lt;T: <a class=\"trait\" href=\"boa_engine/object/trait.NativeObject.html\" title=\"trait boa_engine::object::NativeObject\">NativeObject</a>&gt;(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.reference.html\">&amp;T</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Downcast a reference to the object,\nif the object is type native object type <code>T</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.downcast_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#355-357\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.downcast_mut\" class=\"fn\">downcast_mut</a>&lt;T: <a class=\"trait\" href=\"boa_engine/object/trait.NativeObject.html\" title=\"trait boa_engine::object::NativeObject\">NativeObject</a>&gt;(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.reference.html\">&amp;mut T</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Downcast a mutable reference to the object,\nif the object is type native object type <code>T</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_buffer\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#362-368\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.as_buffer\" class=\"fn\">as_buffer</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"boa_engine/builtins/array_buffer/enum.BufferRef.html\" title=\"enum boa_engine::builtins::array_buffer::BufferRef\">BufferRef</a>&lt;'_&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Gets the buffer data if the object is an <code>ArrayBuffer</code> or a <code>SharedArrayBuffer</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_buffer_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#372-384\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.as_buffer_mut\" class=\"fn\">as_buffer_mut</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.75.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"boa_engine/builtins/array_buffer/enum.BufferRefMut.html\" title=\"enum boa_engine::builtins::array_buffer::BufferRefMut\">BufferRefMut</a>&lt;'_&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Gets the mutable buffer data if the object is an <code>ArrayBuffer</code> or a <code>SharedArrayBuffer</code>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_arguments\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#387-389\">source</a><h4 class=\"code-header\">pub(crate) fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_arguments\" class=\"fn\">is_arguments</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if this object is an <code>Arguments</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_uint8_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#394-400\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_uint8_array\" class=\"fn\">is_typed_uint8_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Uint8Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_int8_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#405-411\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_int8_array\" class=\"fn\">is_typed_int8_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Int8Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_uint16_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#416-422\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_uint16_array\" class=\"fn\">is_typed_uint16_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Uint16Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_int16_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#427-433\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_int16_array\" class=\"fn\">is_typed_int16_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Int16Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_uint32_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#438-444\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_uint32_array\" class=\"fn\">is_typed_uint32_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Uint32Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_int32_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#449-455\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_int32_array\" class=\"fn\">is_typed_int32_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Int32Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_float32_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#460-466\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_float32_array\" class=\"fn\">is_typed_float32_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Float32Array</code> object.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_typed_float64_array\" class=\"method\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#471-477\">source</a><h4 class=\"code-header\">pub fn <a href=\"boa_engine/object/struct.Object.html#tymethod.is_typed_float64_array\" class=\"fn\">is_typed_float64_array</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.75.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if it a <code>Float64Array</code> object.</p>\n</div></details></div></details>",0,"boa_engine::object::jsobject::ErasedObject"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Finalize-for-Object%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#impl-Finalize-for-Object%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"boa_engine/trait.Finalize.html\" title=\"trait boa_engine::Finalize\">Finalize</a> for <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.finalize\" class=\"method trait-impl\"><a href=\"#method.finalize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Finalize.html#method.finalize\" class=\"fn\">finalize</a>(&amp;self)</h4></section></summary><div class='docblock'>Cleanup logic for a type.</div></details></div></details>","Finalize","boa_engine::object::jsobject::ErasedObject"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Object%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#impl-Debug-for-Object%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/1.75.0/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.75.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","boa_engine::object::jsobject::ErasedObject"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Trace-for-Object%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#impl-Trace-for-Object%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"boa_engine/trait.Trace.html\" title=\"trait boa_engine::Trace\">Trace</a> for <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;T&gt;<span class=\"where fmt-newline\">where\n    T: <a class=\"trait\" href=\"boa_engine/trait.Trace.html\" title=\"trait boa_engine::Trace\">Trace</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</span></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#method.trace\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace\" class=\"fn\">trace</a>(&amp;self, tracer: &amp;mut Tracer)</h4></section></summary><div class='docblock'>Marks all contained <code>Gc</code>s. <a href=\"boa_engine/trait.Trace.html#tymethod.trace\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.trace_non_roots\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#method.trace_non_roots\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\" class=\"fn\">trace_non_roots</a>(&amp;self)</h4></section></summary><div class='docblock'>Trace handles located in GC heap, and mark them as non root. <a href=\"boa_engine/trait.Trace.html#tymethod.trace_non_roots\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.run_finalizer\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#181\">source</a><a href=\"#method.run_finalizer\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"boa_engine/trait.Trace.html#tymethod.run_finalizer\" class=\"fn\">run_finalizer</a>(&amp;self)</h4></section></summary><div class='docblock'>Runs <a href=\"boa_engine/trait.Finalize.html#method.finalize\" title=\"method boa_engine::Finalize::finalize\"><code>Finalize::finalize</code></a> on this object and all\ncontained subobjects.</div></details></div></details>","Trace","boa_engine::object::jsobject::ErasedObject"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Object%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#195-204\">source</a><a href=\"#impl-Default-for-Object%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.75.0/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"boa_engine/object/struct.Object.html\" title=\"struct boa_engine::object::Object\">Object</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/boa_engine/object/mod.rs.html#196-203\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.75.0/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; Self</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/1.75.0/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","boa_engine::object::jsobject::ErasedObject"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()