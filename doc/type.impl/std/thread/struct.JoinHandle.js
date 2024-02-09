(function() {var type_impls = {
"boa_tester":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-JoinHandle%3CT%3E\" class=\"impl\"><a class=\"src rightside\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1592\">source</a><a href=\"#impl-JoinHandle%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html\" title=\"struct std::thread::JoinHandle\">JoinHandle</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.thread\" class=\"method\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1611\">source</a></span><h4 class=\"code-header\">pub fn <a href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html#tymethod.thread\" class=\"fn\">thread</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.Thread.html\" title=\"struct std::thread::Thread\">Thread</a></h4></section></summary><div class=\"docblock\"><p>Extracts a handle to the underlying thread.</p>\n<h5 id=\"examples\"><a href=\"#examples\">Examples</a></h5>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>std::thread;\n\n<span class=\"kw\">let </span>builder = thread::Builder::new();\n\n<span class=\"kw\">let </span>join_handle: thread::JoinHandle&lt;<span class=\"kw\">_</span>&gt; = builder.spawn(|| {\n    <span class=\"comment\">// some work here\n</span>}).unwrap();\n\n<span class=\"kw\">let </span>thread = join_handle.thread();\n<span class=\"macro\">println!</span>(<span class=\"string\">\"thread id: {:?}\"</span>, thread.id());</code></pre></div>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.join\" class=\"method\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1649\">source</a></span><h4 class=\"code-header\">pub fn <a href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html#tymethod.join\" class=\"fn\">join</a>(self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;T, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/alloc/boxed/struct.Box.html\" title=\"struct alloc::boxed::Box\">Box</a>&lt;dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/any/trait.Any.html\" title=\"trait core::any::Any\">Any</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Waits for the associated thread to finish.</p>\n<p>This function will return immediately if the associated thread has already finished.</p>\n<p>In terms of <a href=\"https://doc.rust-lang.org/1.76.0/core/sync/atomic/index.html\" title=\"mod core::sync::atomic\">atomic memory orderings</a>,  the completion of the associated\nthread synchronizes with this function returning. In other words, all\noperations performed by that thread <a href=\"https://doc.rust-lang.org/nomicon/atomics.html#data-accesses\">happen\nbefore</a> all\noperations that happen after <code>join</code> returns.</p>\n<p>If the associated thread panics, <a href=\"https://doc.rust-lang.org/1.76.0/core/result/enum.Result.html#variant.Err\" title=\"variant core::result::Result::Err\"><code>Err</code></a> is returned with the parameter given\nto <a href=\"https://doc.rust-lang.org/1.76.0/std/macro.panic.html\" title=\"macro std::panic\"><code>panic!</code></a>.</p>\n<h5 id=\"panics\"><a href=\"#panics\">Panics</a></h5>\n<p>This function may panic on some platforms if a thread attempts to join\nitself or otherwise may create a deadlock with joining threads.</p>\n<h5 id=\"examples-1\"><a href=\"#examples-1\">Examples</a></h5>\n<div class=\"example-wrap\"><pre class=\"rust rust-example-rendered\"><code><span class=\"kw\">use </span>std::thread;\n\n<span class=\"kw\">let </span>builder = thread::Builder::new();\n\n<span class=\"kw\">let </span>join_handle: thread::JoinHandle&lt;<span class=\"kw\">_</span>&gt; = builder.spawn(|| {\n    <span class=\"comment\">// some work here\n</span>}).unwrap();\njoin_handle.join().expect(<span class=\"string\">\"Couldn't join on the associated thread\"</span>);</code></pre></div>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_finished\" class=\"method\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.61.0\">1.61.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1664\">source</a></span><h4 class=\"code-header\">pub fn <a href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html#tymethod.is_finished\" class=\"fn\">is_finished</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.bool.html\">bool</a></h4></section></summary><div class=\"docblock\"><p>Checks if the associated thread has finished running its main function.</p>\n<p><code>is_finished</code> supports implementing a non-blocking join operation, by checking\n<code>is_finished</code>, and calling <code>join</code> if it returns <code>true</code>. This function does not block. To\nblock while waiting on the thread to finish, use <a href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html#method.join\" title=\"method std::thread::JoinHandle::join\"><code>join</code></a>.</p>\n<p>This might return <code>true</code> for a brief moment after the thread’s main\nfunction has returned, but before the thread itself has stopped running.\nHowever, once this returns <code>true</code>, <a href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html#method.join\" title=\"method std::thread::JoinHandle::join\"><code>join</code></a> can be expected\nto return quickly, without blocking for any significant amount of time.</p>\n</div></details></div></details>",0,"boa_tester::exec::js262::WorkerHandle"],["<section id=\"impl-Sync-for-JoinHandle%3CT%3E\" class=\"impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.29.0\">1.29.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1590\">source</a></span><a href=\"#impl-Sync-for-JoinHandle%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html\" title=\"struct std::thread::JoinHandle\">JoinHandle</a>&lt;T&gt;</h3></section>","Sync","boa_tester::exec::js262::WorkerHandle"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-JoinHandleExt-for-JoinHandle%3CT%3E\" class=\"impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.9.0\">1.9.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/os/unix/thread.rs.html#33\">source</a></span><a href=\"#impl-JoinHandleExt-for-JoinHandle%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/std/os/unix/thread/trait.JoinHandleExt.html\" title=\"trait std::os::unix::thread::JoinHandleExt\">JoinHandleExt</a> for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html\" title=\"struct std::thread::JoinHandle\">JoinHandle</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_pthread_t\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"https://doc.rust-lang.org/1.76.0/src/std/os/unix/thread.rs.html#34\">source</a><a href=\"#method.as_pthread_t\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/std/os/unix/thread/trait.JoinHandleExt.html#tymethod.as_pthread_t\" class=\"fn\">as_pthread_t</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Extracts the raw pthread_t without taking ownership</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.into_pthread_t\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"https://doc.rust-lang.org/1.76.0/src/std/os/unix/thread.rs.html#38\">source</a><a href=\"#method.into_pthread_t\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/std/os/unix/thread/trait.JoinHandleExt.html#tymethod.into_pthread_t\" class=\"fn\">into_pthread_t</a>(self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.u64.html\">u64</a></h4></section></summary><div class='docblock'>Consumes the thread, returning the raw pthread_t <a href=\"https://doc.rust-lang.org/1.76.0/std/os/unix/thread/trait.JoinHandleExt.html#tymethod.into_pthread_t\">Read more</a></div></details></div></details>","JoinHandleExt","boa_tester::exec::js262::WorkerHandle"],["<section id=\"impl-Send-for-JoinHandle%3CT%3E\" class=\"impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.29.0\">1.29.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1588\">source</a></span><a href=\"#impl-Send-for-JoinHandle%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html\" title=\"struct std::thread::JoinHandle\">JoinHandle</a>&lt;T&gt;</h3></section>","Send","boa_tester::exec::js262::WorkerHandle"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-JoinHandle%3CT%3E\" class=\"impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.16.0\">1.16.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1682\">source</a></span><a href=\"#impl-Debug-for-JoinHandle%3CT%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/std/thread/struct.JoinHandle.html\" title=\"struct std::thread::JoinHandle\">JoinHandle</a>&lt;T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"https://doc.rust-lang.org/1.76.0/src/std/thread/mod.rs.html#1683\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.76.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.76.0/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.76.0/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/1.76.0/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","boa_tester::exec::js262::WorkerHandle"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()