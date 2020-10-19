(function() {var implementors = {};
implementors["arrayvec"] = [{"text":"impl&lt;A:&nbsp;Array&lt;Item = u8&gt;&gt; Borrow&lt;str&gt; for ArrayString&lt;A&gt;","synthetic":false,"types":[]},{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for ArrayVec&lt;A&gt;","synthetic":false,"types":[]}];
implementors["boa"] = [{"text":"impl Borrow&lt;str&gt; for RcString","synthetic":false,"types":[]}];
implementors["gc"] = [{"text":"impl&lt;T:&nbsp;Trace + ?Sized&gt; Borrow&lt;T&gt; for Gc&lt;T&gt;","synthetic":false,"types":[]}];
implementors["num_format"] = [{"text":"impl Borrow&lt;str&gt; for Buffer","synthetic":false,"types":[]}];
implementors["tinyvec"] = [{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for ArrayVec&lt;A&gt;","synthetic":false,"types":[]},{"text":"impl&lt;A:&nbsp;Array&gt; Borrow&lt;[&lt;A as Array&gt;::Item]&gt; for TinyVec&lt;A&gt;","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()