window.BENCHMARK_DATA = {
  "lastUpdate": 1641925100267,
  "repoUrl": "https://github.com/boa-dev/boa",
  "entries": {
    "Boa Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jase.williams@gmail.com",
            "name": "Jason Williams",
            "username": "jasonwilliams"
          },
          "committer": {
            "email": "jase.williams@gmail.com",
            "name": "Jason Williams",
            "username": "jasonwilliams"
          },
          "distinct": false,
          "id": "13b29ecd682f323a1221d227428f61f0985e36cf",
          "message": "auto generate release notes based on labels (#1756)\n\nThis should help with the process of making the changelogs on releases.\r\nhttps://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes",
          "timestamp": "2021-12-22T03:44:12Z",
          "tree_id": "29b98305f878614e9e841023f80d18a8e59bea57",
          "url": "https://github.com/boa-dev/boa/commit/13b29ecd682f323a1221d227428f61f0985e36cf"
        },
        "date": 1640146523986,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 344.35,
            "range": "± 0.060",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2025,
            "range": "± 0.003",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.12,
            "range": "± 0.010",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2572,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 871.69,
            "range": "± 1.130",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 9.4953,
            "range": "± 0.035",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 12.546,
            "range": "± 0.055",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.8046,
            "range": "± 0.004",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 674.58,
            "range": "± 0.900",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 622.27,
            "range": "± 3.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 302.19,
            "range": "± 0.140",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 374.79,
            "range": "± 1.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.6486,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 369.73,
            "range": "± 0.280",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.9584,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.1816,
            "range": "± 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 316.93,
            "range": "± 0.170",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 326.86,
            "range": "± 0.240",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 335.58,
            "range": "± 0.140",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 331.05,
            "range": "± 0.290",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 311.64,
            "range": "± 0.120",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.0563,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.98,
            "range": "± 0.980",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1726,
            "range": "± 0.001",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.1067,
            "range": "± 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.23,
            "range": "± 0.008",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 727.14,
            "range": "± 1.810",
            "unit": "ns/iter"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 11.057,
            "range": "± 0.008",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 31.376,
            "range": "± 0.011",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 27.555,
            "range": "± 0.083",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "distinct": false,
          "id": "baa272c9bbc38ee3d38ce20c890986f739af13e1",
          "message": "Bump webpack-dev-server from 4.6.0 to 4.7.1 (#1759)\n\nBumps [webpack-dev-server](https://github.com/webpack/webpack-dev-server) from 4.6.0 to 4.7.1.\n<details>\n<summary>Release notes</summary>\n<p><em>Sourced from <a href=\"https://github.com/webpack/webpack-dev-server/releases\">webpack-dev-server's releases</a>.</em></p>\n<blockquote>\n<h2>v4.7.1</h2>\n<h3><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.7.0...v4.7.1\">4.7.1</a> (2021-12-22)</h3>\n<h3>Bug Fixes</h3>\n<ul>\n<li>removed <code>url</code> package, fixed compatibility with future webpack defaults (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4132\">#4132</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/4e5d8eae654ef382697722c6406dbc96207594aa\">4e5d8ea</a>)</li>\n</ul>\n<h2>v4.7.0</h2>\n<h2><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.6.0...v4.7.0\">4.7.0</a> (2021-12-21)</h2>\n<h3>Features</h3>\n<ul>\n<li>added the <code>setupMiddlewares</code> option and deprecated <code>onAfterSetupMiddleware</code> and <code>onBeforeSetupMiddleware</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4068\">#4068</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/c13aa560651a3bb4c4a7b1b4363c04383596c7e9\">c13aa56</a>)</li>\n<li>added types (<a href=\"https://github.com/webpack/webpack-dev-server/commit/8f02c3f3d6131fd37f58ef4d5cbe15578c94a6fd\">8f02c3f</a>)</li>\n<li>show deprecation warning for <code>cacert</code> option (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4115\">#4115</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/c73ddfb934ec748e3dd34456d4293b933e9c6c99\">c73ddfb</a>)</li>\n</ul>\n<h3>Bug Fixes</h3>\n<ul>\n<li>add description for <code>watchFiles</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4057\">#4057</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/75f381751e5377ae297c32f9fcdcd096ef28c5c2\">75f3817</a>)</li>\n<li>allow passing options for custom server (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4110\">#4110</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/fc8bed95251f27a24c1441307c44782f3836edd6\">fc8bed9</a>)</li>\n<li>correct schema for <code>ClientLogging</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4084\">#4084</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/9b7ae7b5f4ac4a920b1ae3b47a8eb15d093cb369\">9b7ae7b</a>)</li>\n<li>mark <code>--open-app</code> deprecated in favor of <code>--open-app-name</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4091\">#4091</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/693c28a0499e431b09274b8b7ecce71adb292c8f\">693c28a</a>)</li>\n<li>show deprecation warning for both <code>https</code> and <code>http2</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4069\">#4069</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/d8d5d71c8ca495098e1ee30ebc72ffd657ad5ba0\">d8d5d71</a>)</li>\n<li>update <code>--web-socket-server</code> description (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4098\">#4098</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/65955e96cf7869dd4294699fd2a3878c2179c656\">65955e9</a>)</li>\n<li>update <code>listen</code> and <code>close</code> deprecation warning message (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4097\">#4097</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/b217a191d09a93e8dcc1fff2ee26e97857e096d3\">b217a19</a>)</li>\n<li>update descriptions of <code>https</code> and <code>server</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4094\">#4094</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/f97c9e2df460ef9a84c8ab2016c6bce3c90d93ac\">f97c9e2</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a href=\"https://github.com/webpack/webpack-dev-server/blob/master/CHANGELOG.md\">webpack-dev-server's changelog</a>.</em></p>\n<blockquote>\n<h3><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.7.0...v4.7.1\">4.7.1</a> (2021-12-22)</h3>\n<h3>Bug Fixes</h3>\n<ul>\n<li>removed <code>url</code> package, fixed compatibility with future webpack defaults (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4132\">#4132</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/4e5d8eae654ef382697722c6406dbc96207594aa\">4e5d8ea</a>)</li>\n</ul>\n<h2><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.6.0...v4.7.0\">4.7.0</a> (2021-12-21)</h2>\n<h3>Features</h3>\n<ul>\n<li>added the <code>setupMiddlewares</code> option and deprecated <code>onAfterSetupMiddleware</code> and <code>onBeforeSetupMiddleware</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4068\">#4068</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/c13aa560651a3bb4c4a7b1b4363c04383596c7e9\">c13aa56</a>)</li>\n<li>added types (<a href=\"https://github.com/webpack/webpack-dev-server/commit/8f02c3f3d6131fd37f58ef4d5cbe15578c94a6fd\">8f02c3f</a>)</li>\n<li>show deprecation warning for <code>cacert</code> option (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4115\">#4115</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/c73ddfb934ec748e3dd34456d4293b933e9c6c99\">c73ddfb</a>)</li>\n</ul>\n<h3>Bug Fixes</h3>\n<ul>\n<li>add description for <code>watchFiles</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4057\">#4057</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/75f381751e5377ae297c32f9fcdcd096ef28c5c2\">75f3817</a>)</li>\n<li>allow passing options for custom server (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4110\">#4110</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/fc8bed95251f27a24c1441307c44782f3836edd6\">fc8bed9</a>)</li>\n<li>correct schema for <code>ClientLogging</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4084\">#4084</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/9b7ae7b5f4ac4a920b1ae3b47a8eb15d093cb369\">9b7ae7b</a>)</li>\n<li>mark <code>--open-app</code> deprecated in favor of <code>--open-app-name</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4091\">#4091</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/693c28a0499e431b09274b8b7ecce71adb292c8f\">693c28a</a>)</li>\n<li>show deprecation warning for both <code>https</code> and <code>http2</code> (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4069\">#4069</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/d8d5d71c8ca495098e1ee30ebc72ffd657ad5ba0\">d8d5d71</a>)</li>\n<li>update <code>--web-socket-server</code> description (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4098\">#4098</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/65955e96cf7869dd4294699fd2a3878c2179c656\">65955e9</a>)</li>\n<li>update <code>listen</code> and <code>close</code> deprecation warning message (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4097\">#4097</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/b217a191d09a93e8dcc1fff2ee26e97857e096d3\">b217a19</a>)</li>\n<li>update descriptions of <code>https</code> and <code>server</code> options (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4094\">#4094</a>) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/f97c9e2df460ef9a84c8ab2016c6bce3c90d93ac\">f97c9e2</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/afe49753b9f38679d200e88059bbe9a97e25e368\"><code>afe4975</code></a> chore(release): 4.1.7</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/4e5d8eae654ef382697722c6406dbc96207594aa\"><code>4e5d8ea</code></a> fix: droped <code>url</code> package (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4132\">#4132</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/b0c98f047e41116d947490e3adcdfaccaaf9afb5\"><code>b0c98f0</code></a> chore(release): 4.7.0</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/3138213401301ebf191b3b152a78529f5f5e412b\"><code>3138213</code></a> chore(deps): update (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4127\">#4127</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/8f02c3f3d6131fd37f58ef4d5cbe15578c94a6fd\"><code>8f02c3f</code></a> feat: added types</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/f4fb15f14cd1c2b6bd3a536c4d25b3004f035a90\"><code>f4fb15f</code></a> fix: update description of <code>onAfterSetupMiddleware</code> and `onBeforeSetupMiddlew...</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/37b73d5f7d7e3cff12fed8aedfc981b3fb4d3de7\"><code>37b73d5</code></a> test: add e2e test for <code>WEBPACK_SERVE</code> env variable (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4125\">#4125</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/f5a9d05f3888cd5c0bb9e974d48680710fdda6f7\"><code>f5a9d05</code></a> chore(deps-dev): bump eslint from 8.4.1 to 8.5.0 (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4121\">#4121</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/c9b959fe15e5778a906d957f832a43384cd90b1b\"><code>c9b959f</code></a> chore(deps): bump ws from 8.3.0 to 8.4.0 (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4124\">#4124</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/42208aab74c5b77382b8e8058e657e478ee62174\"><code>42208aa</code></a> chore(deps-dev): bump lint-staged from 12.1.2 to 12.1.3 (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4122\">#4122</a>)</li>\n<li>Additional commits viewable in <a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.6.0...v4.7.1\">compare view</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility score](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=webpack-dev-server&package-manager=npm_and_yarn&previous-version=4.6.0&new-version=4.7.1)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't alter it yourself. You can also trigger a rebase manually by commenting `@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits that have been made to it\n- `@dependabot merge` will merge this PR after your CI passes on it\n- `@dependabot squash and merge` will squash and merge this PR after your CI passes on it\n- `@dependabot cancel merge` will cancel a previously requested merge and block automerging\n- `@dependabot reopen` will reopen this PR if it is closed\n- `@dependabot close` will close this PR and stop Dependabot recreating it. You can achieve the same result by closing it manually\n- `@dependabot ignore this major version` will close this PR and stop Dependabot creating any more for this major version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop Dependabot creating any more for this minor version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop Dependabot creating any more for this dependency (unless you reopen the PR or upgrade to it yourself)\n\n\n</details>",
          "timestamp": "2021-12-23T15:11:48Z",
          "tree_id": "b0a4c4856db1f9c4faf3194476535f534bcabd79",
          "url": "https://github.com/boa-dev/boa/commit/baa272c9bbc38ee3d38ce20c890986f739af13e1"
        },
        "date": 1640274009314,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 344.16,
            "range": "± 0.050",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2468,
            "range": "± 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.269,
            "range": "± 0.008",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2693,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 870.27,
            "range": "± 0.420",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 9.6632,
            "range": "± 0.006",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 12.631,
            "range": "± 0.008",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1845,
            "range": "± 0.002",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 592.69,
            "range": "± 0.790",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 547.7,
            "range": "± 0.850",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 301.98,
            "range": "± 0.400",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 330.75,
            "range": "± 0.220",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.3322,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 324.53,
            "range": "± 0.310",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.5667,
            "range": "± 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.3224,
            "range": "± 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 315.71,
            "range": "± 0.140",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 321.82,
            "range": "± 0.130",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 331.68,
            "range": "± 0.150",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 334.52,
            "range": "± 0.260",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 313.35,
            "range": "± 0.200",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 932.25,
            "range": "± 1.030",
            "unit": "us"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.62,
            "range": "± 1.020",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5853,
            "range": "± 0.001",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.7354,
            "range": "± 0.002",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.175,
            "range": "± 0.006",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 646.95,
            "range": "± 0.210",
            "unit": "ns/iter"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 11.091,
            "range": "± 0.009",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 28.224,
            "range": "± 0.016",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 24.775,
            "range": "± 0.009",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "committer": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "distinct": false,
          "id": "039c46ba7b3d6d672bfe7c6bc395677e1240874b",
          "message": "Removed a bunch of warnings and clippy errors (#1754)\n\nThis Pull Request fixes some warnings and clips errors. It conflicts with the VM/non-VM PR, so should probably go in first, so that this branch gets properly updated and we get the list of real warnings/errors there.",
          "timestamp": "2021-12-23T17:43:15Z",
          "tree_id": "a58638a3e680d9d3775df1ee7a317f4eeeb68ed7",
          "url": "https://github.com/boa-dev/boa/commit/039c46ba7b3d6d672bfe7c6bc395677e1240874b"
        },
        "date": 1640283483296,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 303.96,
            "range": "± 0.120",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7571,
            "range": "± 0.002",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 16.09,
            "range": "± 0.009",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0001,
            "range": "± 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 768.75,
            "range": "± 0.420",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 8.4105,
            "range": "± 0.007",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 11.256,
            "range": "± 0.013",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.7539,
            "range": "± 0.005",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 672.31,
            "range": "± 0.900",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 618.61,
            "range": "± 1.020",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 342.56,
            "range": "± 0.730",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 372.82,
            "range": "± 1.070",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.663,
            "range": "± 0.008",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 371.38,
            "range": "± 0.260",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.9027,
            "range": "± 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.3203,
            "range": "± 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 359.75,
            "range": "± 0.290",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 368.88,
            "range": "± 1.910",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 371.19,
            "range": "± 0.260",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 378.1,
            "range": "± 0.300",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 353.26,
            "range": "± 0.320",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.049,
            "range": "± 0.002",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.58,
            "range": "± 1.330",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6143,
            "range": "± 0.011",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.7383,
            "range": "± 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.149,
            "range": "± 0.009",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 647.98,
            "range": "± 0.380",
            "unit": "ns/iter"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 9.7356,
            "range": "± 0.013",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 27.686,
            "range": "± 0.009",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 24.281,
            "range": "± 0.016",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "committer": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "distinct": false,
          "id": "949e481be88c48b833ef3daaa9995ade691f672c",
          "message": "Fix some broken links in the profiler documentation (#1762)\n\nThe `measureme` repo changed their file names for their READMEs, so the links were broken. This is now fixed.",
          "timestamp": "2021-12-24T13:05:27Z",
          "tree_id": "feb742e56b12ac5871e669d9e08e811b9f8c790b",
          "url": "https://github.com/boa-dev/boa/commit/949e481be88c48b833ef3daaa9995ade691f672c"
        },
        "date": 1640352839530,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 416.72,
            "range": "± 15.910",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.5811,
            "range": "± 0.178",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.159,
            "range": "± 0.722",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.1741,
            "range": "± 0.085",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 993.36,
            "range": "± -978.023",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 11.801,
            "range": "± 0.358",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 16.262,
            "range": "± 0.431",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.1516,
            "range": "± 0.179",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 806.28,
            "range": "± 25.610",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 713.58,
            "range": "± 23.360",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 438.21,
            "range": "± 12.250",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 480.71,
            "range": "± 13.660",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 3.6164,
            "range": "± 0.091",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 476.64,
            "range": "± 11.690",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.6968,
            "range": "± 0.084",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.642,
            "range": "± 0.038",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 469.85,
            "range": "± 10.410",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 475.56,
            "range": "± 12.470",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 493.47,
            "range": "± 13.080",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 493.13,
            "range": "± 11.590",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 471.83,
            "range": "± 10.220",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.3081,
            "range": "± 0.043",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 1.2401,
            "range": "± 0.038",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 6.2477,
            "range": "± 0.289",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.6762,
            "range": "± 0.119",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 17.679,
            "range": "± 0.539",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 817.13,
            "range": "± 32.670",
            "unit": "ns/iter"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 13.036,
            "range": "± 0.504",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 37.76,
            "range": "± 1.274",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 33.18,
            "range": "± 0.648",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "committer": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "distinct": false,
          "id": "0545f004248c26c4afaf1ca5d9dbe787f4064575",
          "message": "Updated test262 suite and dependencies (#1755)\n\nThis Pull Request updates the Test262 test suite with the latest tests, and updates both the JavaScript and Rust dependencies.",
          "timestamp": "2021-12-24T13:14:36Z",
          "tree_id": "06a9f5b506329571fd3a06976f7ed69ada5dad8c",
          "url": "https://github.com/boa-dev/boa/commit/0545f004248c26c4afaf1ca5d9dbe787f4064575"
        },
        "date": 1640353707148,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 421.71,
            "range": "± 6.010",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.5901,
            "range": "± 0.192",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.487,
            "range": "± 0.644",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9769,
            "range": "± 0.080",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 991.5,
            "range": "± -978.116",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 12.228,
            "range": "± 0.384",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 16.288,
            "range": "± 0.312",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.1595,
            "range": "± 0.123",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 734.41,
            "range": "± 18.850",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 691.43,
            "range": "± 24.170",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 430.47,
            "range": "± 12.340",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 478.85,
            "range": "± 13.740",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 3.5538,
            "range": "± 0.089",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 454.79,
            "range": "± 10.890",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.3446,
            "range": "± 0.092",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.6567,
            "range": "± 0.039",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 484.35,
            "range": "± 12.800",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 464.56,
            "range": "± 16.260",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 454.31,
            "range": "± 9.500",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 498.01,
            "range": "± 16.940",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 435.97,
            "range": "± 8.260",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.3158,
            "range": "± 0.041",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 1.2071,
            "range": "± 0.032",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 6.1983,
            "range": "± 0.172",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.7261,
            "range": "± 0.145",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 18.261,
            "range": "± 0.477",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 826.1,
            "range": "± 24.540",
            "unit": "ns/iter"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 12.479,
            "range": "± 0.372",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 36.423,
            "range": "± 1.191",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 31.451,
            "range": "± 0.729",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "936006+jasonwilliams@users.noreply.github.com",
            "name": "Jason Williams",
            "username": "jasonwilliams"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dfb3df5bf2c920262a0250d4b924201e78373541",
          "message": "Start removing non-VM path (#1747)",
          "timestamp": "2021-12-25T18:56:36+01:00",
          "tree_id": "699f9b045c443fc5d27154b330f12abe1a5ef6c6",
          "url": "https://github.com/boa-dev/boa/commit/dfb3df5bf2c920262a0250d4b924201e78373541"
        },
        "date": 1640456211531,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 394.74,
            "range": "± 2.930",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 4.9877,
            "range": "± 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 16.572,
            "range": "± 0.114",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 19.367,
            "range": "± 0.255",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 9.7897,
            "range": "± 0.041",
            "unit": "us"
          },
          {
            "name": "RegExp (Parser)",
            "value": 12.194,
            "range": "± 0.063",
            "unit": "us"
          },
          {
            "name": "Array access (Parser)",
            "value": 14.792,
            "range": "± 0.240",
            "unit": "us"
          },
          {
            "name": "Array creation (Parser)",
            "value": 15.992,
            "range": "± 0.101",
            "unit": "us"
          },
          {
            "name": "Array pop (Parser)",
            "value": 173.55,
            "range": "± 1.100",
            "unit": "us"
          },
          {
            "name": "String copy (Parser)",
            "value": 6.606,
            "range": "± 0.078",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 34.241,
            "range": "± 0.220",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 30.085,
            "range": "± 0.256",
            "unit": "us"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 962.94,
            "range": "± 3.630",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2.8776,
            "range": "± 0.028",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 3.3503,
            "range": "± 0.015",
            "unit": "us"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 1.9611,
            "range": "± 0.028",
            "unit": "us"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1.6459,
            "range": "± 0.030",
            "unit": "us"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 8.3769,
            "range": "± 0.099",
            "unit": "us"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1.4448,
            "range": "± 0.027",
            "unit": "us"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 6.182,
            "range": "± 0.126",
            "unit": "us"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 5.8261,
            "range": "± 0.099",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.8707,
            "range": "± 0.092",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 49.969,
            "range": "± 0.792",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0927,
            "range": "± 0.048",
            "unit": "ms"
          },
          {
            "name": "RegExp (Execution)",
            "value": 13.883,
            "range": "± 0.232",
            "unit": "us"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5393,
            "range": "± 0.023",
            "unit": "ms"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.1732,
            "range": "± 0.093",
            "unit": "us"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1.5995,
            "range": "± 0.021",
            "unit": "ms"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1.5253,
            "range": "± 0.025",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "50dda0ba7e4a12a044731e1f6b0363403620c551",
          "message": "Using upstream benchmark action (#1753)\n\n* Using upstream benchmark action\r\n\r\n* Updated benchmarks action",
          "timestamp": "2021-12-31T14:10:59+01:00",
          "tree_id": "46d1948f911af692d193904bfbff39a922df9712",
          "url": "https://github.com/boa-dev/boa/commit/50dda0ba7e4a12a044731e1f6b0363403620c551"
        },
        "date": 1640957402332,
        "tool": "cargo",
        "benches": [
          {
            "name": "Create Realm",
            "value": 336,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 4778,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Parser)",
            "value": 15686,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 18479,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Parser)",
            "value": 10636,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Parser)",
            "value": 11335,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Parser)",
            "value": 12243,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Parser)",
            "value": 7148,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Parser)",
            "value": 9549,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 9278,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Parser)",
            "value": 11627,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Parser)",
            "value": 13540,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Parser)",
            "value": 15159,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Parser)",
            "value": 159553,
            "range": "± 226",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Parser)",
            "value": 8675,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Parser)",
            "value": 12536,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Parser)",
            "value": 6397,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Parser)",
            "value": 12625,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Parser)",
            "value": 15936,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Parser)",
            "value": 15856,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Parser)",
            "value": 6215,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Parser)",
            "value": 33137,
            "range": "± 68",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Parser)",
            "value": 28777,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 803,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2412,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 2870,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Compiler)",
            "value": 1475,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Compiler)",
            "value": 1567,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Compiler)",
            "value": 1878,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Compiler)",
            "value": 1482,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Compiler)",
            "value": 1484,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Compiler)",
            "value": 1821,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 1821,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1445,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Compiler)",
            "value": 2222,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 7279,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Compiler)",
            "value": 1784,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Compiler)",
            "value": 2519,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1257,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Compiler)",
            "value": 1668,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Compiler)",
            "value": 2008,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Compiler)",
            "value": 2467,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Compiler)",
            "value": 968,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 5460,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 5344,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5240,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Execution)",
            "value": 45937,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2851316,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Execution)",
            "value": 6423,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Execution)",
            "value": 6600,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Execution)",
            "value": 7130,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Execution)",
            "value": 10082,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Execution)",
            "value": 10104,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Execution)",
            "value": 13199,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Execution)",
            "value": 13226,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Execution)",
            "value": 10749,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Execution)",
            "value": 3186127,
            "range": "± 5885",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1344167,
            "range": "± 5352",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Execution)",
            "value": 6418,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Execution)",
            "value": 7679,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Execution)",
            "value": 5594,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Execution)",
            "value": 5449,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Execution)",
            "value": 6844,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Execution)",
            "value": 8722,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Execution)",
            "value": 2187,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1460586,
            "range": "± 13850",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1347274,
            "range": "± 10698",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RageKnify@gmail.com",
            "name": "Joã Borges",
            "username": "RageKnify"
          },
          "committer": {
            "email": "RageKnify@gmail.com",
            "name": "Joã Borges",
            "username": "RageKnify"
          },
          "distinct": false,
          "id": "56cd7f38b89599c5d32841f4855f4c648142d17c",
          "message": "Fix bors hanging (#1767)\n\nThis Pull Request fixes the bors hanging we've had recently\r\nThe vm action had been removed but bors was still waiting for it\r\n\r\nIt changes the following:\r\n\r\n- Remove 'Tests on Linux with vm enabled' from the actions to be waited for",
          "timestamp": "2021-12-31T16:50:45Z",
          "tree_id": "129e18bda125f27020faf2a3e21e2d0b5f7fa2e3",
          "url": "https://github.com/boa-dev/boa/commit/56cd7f38b89599c5d32841f4855f4c648142d17c"
        },
        "date": 1640971185215,
        "tool": "cargo",
        "benches": [
          {
            "name": "Create Realm",
            "value": 411,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 5097,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Parser)",
            "value": 17377,
            "range": "± 739",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 20291,
            "range": "± 876",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Parser)",
            "value": 11625,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Parser)",
            "value": 12292,
            "range": "± 374",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Parser)",
            "value": 13138,
            "range": "± 449",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Parser)",
            "value": 7949,
            "range": "± 368",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Parser)",
            "value": 10389,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 10038,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Parser)",
            "value": 12491,
            "range": "± 832",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Parser)",
            "value": 14981,
            "range": "± 1027",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Parser)",
            "value": 16626,
            "range": "± 1257",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Parser)",
            "value": 182359,
            "range": "± 6111",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Parser)",
            "value": 9275,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Parser)",
            "value": 13839,
            "range": "± 349",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Parser)",
            "value": 6812,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Parser)",
            "value": 13722,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Parser)",
            "value": 17720,
            "range": "± 683",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Parser)",
            "value": 17363,
            "range": "± 430",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Parser)",
            "value": 6956,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Parser)",
            "value": 36235,
            "range": "± 1548",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Parser)",
            "value": 31771,
            "range": "± 621",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 967,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2974,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 3419,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Compiler)",
            "value": 1796,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Compiler)",
            "value": 1920,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Compiler)",
            "value": 2286,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Compiler)",
            "value": 1815,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Compiler)",
            "value": 1810,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Compiler)",
            "value": 2208,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 2198,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1749,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Compiler)",
            "value": 2722,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 8866,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Compiler)",
            "value": 2161,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Compiler)",
            "value": 3082,
            "range": "± 105",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1532,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Compiler)",
            "value": 1980,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Compiler)",
            "value": 2409,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Compiler)",
            "value": 2947,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Compiler)",
            "value": 1185,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 6737,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 6442,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 6555,
            "range": "± 242",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Execution)",
            "value": 56469,
            "range": "± 2141",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3453209,
            "range": "± 106194",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Execution)",
            "value": 7897,
            "range": "± 609",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Execution)",
            "value": 8250,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Execution)",
            "value": 8517,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Execution)",
            "value": 12436,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Execution)",
            "value": 12479,
            "range": "± 306",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Execution)",
            "value": 16166,
            "range": "± 542",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Execution)",
            "value": 16095,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Execution)",
            "value": 13145,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Execution)",
            "value": 3948839,
            "range": "± 114445",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1635033,
            "range": "± 60447",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Execution)",
            "value": 7932,
            "range": "± 404",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Execution)",
            "value": 9501,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Execution)",
            "value": 6929,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Execution)",
            "value": 6670,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Execution)",
            "value": 8347,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Execution)",
            "value": 10646,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Execution)",
            "value": 2675,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1784708,
            "range": "± 55408",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1643381,
            "range": "± 40050",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "distinct": false,
          "id": "d831ff3dc599eb5fbf1d0e01a0120f530e184c17",
          "message": "Bump webpack-dev-server from 4.7.1 to 4.7.2 (#1766)\n\nBumps [webpack-dev-server](https://github.com/webpack/webpack-dev-server) from 4.7.1 to 4.7.2.\n<details>\n<summary>Release notes</summary>\n<p><em>Sourced from <a href=\"https://github.com/webpack/webpack-dev-server/releases\">webpack-dev-server's releases</a>.</em></p>\n<blockquote>\n<h2>v4.7.2</h2>\n<h3><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.7.1...v4.7.2\">4.7.2</a> (2021-12-29)</h3>\n<h3>Bug Fixes</h3>\n<ul>\n<li>apply <code>onAfterSetupMiddleware</code> after <code>setupMiddlewares</code> (as behavior earlier) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/f6bc644bb81b966e030d8f8a54d5a99cd61ec8f2\">f6bc644</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a href=\"https://github.com/webpack/webpack-dev-server/blob/master/CHANGELOG.md\">webpack-dev-server's changelog</a>.</em></p>\n<blockquote>\n<h3><a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.7.1...v4.7.2\">4.7.2</a> (2021-12-29)</h3>\n<h3>Bug Fixes</h3>\n<ul>\n<li>apply <code>onAfterSetupMiddleware</code> after <code>setupMiddlewares</code> (as behavior earlier) (<a href=\"https://github.com/webpack/webpack-dev-server/commit/f6bc644bb81b966e030d8f8a54d5a99cd61ec8f2\">f6bc644</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/75999bb9bb8803de633fcb037405f45a5bf7d029\"><code>75999bb</code></a> chore(release): 4.7.2</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/90a96f7bd8e5338f91ef8e4fd6c2ddc01e8174db\"><code>90a96f7</code></a> ci: fix (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4143\">#4143</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/f6bc644bb81b966e030d8f8a54d5a99cd61ec8f2\"><code>f6bc644</code></a> fix: compatible with <code>onAfterSetupMiddleware</code></li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/317e4b9d5c94212d2d481e7cea4ab3f40809cca6\"><code>317e4b9</code></a> docs: fix testing instructions (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4133\">#4133</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/ff4550e498988d872f04d0fcebc27c1f946c2097\"><code>ff4550e</code></a> test: remove redundant test cases related to 3rd party code (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4131\">#4131</a>)</li>\n<li><a href=\"https://github.com/webpack/webpack-dev-server/commit/0dd1ee6dcff7245eb15b0ca980953e2154cf77a5\"><code>0dd1ee6</code></a> test: add e2e tests for <code>setupExitSignals</code> option (<a href=\"https://github-redirect.dependabot.com/webpack/webpack-dev-server/issues/4130\">#4130</a>)</li>\n<li>See full diff in <a href=\"https://github.com/webpack/webpack-dev-server/compare/v4.7.1...v4.7.2\">compare view</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility score](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=webpack-dev-server&package-manager=npm_and_yarn&previous-version=4.7.1&new-version=4.7.2)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't alter it yourself. You can also trigger a rebase manually by commenting `@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits that have been made to it\n- `@dependabot merge` will merge this PR after your CI passes on it\n- `@dependabot squash and merge` will squash and merge this PR after your CI passes on it\n- `@dependabot cancel merge` will cancel a previously requested merge and block automerging\n- `@dependabot reopen` will reopen this PR if it is closed\n- `@dependabot close` will close this PR and stop Dependabot recreating it. You can achieve the same result by closing it manually\n- `@dependabot ignore this major version` will close this PR and stop Dependabot creating any more for this major version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop Dependabot creating any more for this minor version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop Dependabot creating any more for this dependency (unless you reopen the PR or upgrade to it yourself)\n\n\n</details>",
          "timestamp": "2022-01-01T21:40:08Z",
          "tree_id": "0e10ef0c9d7125e1d1b0745ab649df181c8c36c6",
          "url": "https://github.com/boa-dev/boa/commit/d831ff3dc599eb5fbf1d0e01a0120f530e184c17"
        },
        "date": 1641074860057,
        "tool": "cargo",
        "benches": [
          {
            "name": "Create Realm",
            "value": 335,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 4616,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Parser)",
            "value": 15190,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 17604,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Parser)",
            "value": 10229,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Parser)",
            "value": 10924,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Parser)",
            "value": 11790,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Parser)",
            "value": 6888,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Parser)",
            "value": 9311,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 8893,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Parser)",
            "value": 11182,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Parser)",
            "value": 13082,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Parser)",
            "value": 14635,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Parser)",
            "value": 153926,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Parser)",
            "value": 8283,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Parser)",
            "value": 12075,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Parser)",
            "value": 6138,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Parser)",
            "value": 12053,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Parser)",
            "value": 15338,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Parser)",
            "value": 15256,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Parser)",
            "value": 6095,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Parser)",
            "value": 31784,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Parser)",
            "value": 27780,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 804,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2392,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 2817,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Compiler)",
            "value": 1470,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Compiler)",
            "value": 1577,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Compiler)",
            "value": 1874,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Compiler)",
            "value": 1479,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Compiler)",
            "value": 1481,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Compiler)",
            "value": 1811,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 1810,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1441,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Compiler)",
            "value": 2237,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 7276,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Compiler)",
            "value": 1779,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Compiler)",
            "value": 2508,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1253,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Compiler)",
            "value": 1667,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Compiler)",
            "value": 2004,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Compiler)",
            "value": 2448,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Compiler)",
            "value": 968,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 5465,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 5288,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5252,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Execution)",
            "value": 46412,
            "range": "± 123",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2840190,
            "range": "± 4559",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Execution)",
            "value": 6441,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Execution)",
            "value": 6662,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Execution)",
            "value": 7120,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Execution)",
            "value": 10074,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Execution)",
            "value": 10121,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Execution)",
            "value": 13080,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Execution)",
            "value": 13161,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Execution)",
            "value": 10800,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Execution)",
            "value": 3194023,
            "range": "± 7899",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1345573,
            "range": "± 5338",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Execution)",
            "value": 6378,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Execution)",
            "value": 7666,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Execution)",
            "value": 5553,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Execution)",
            "value": 5434,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Execution)",
            "value": 6803,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Execution)",
            "value": 8649,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Execution)",
            "value": 2181,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1462413,
            "range": "± 10014",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1348632,
            "range": "± 15442",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "distinct": false,
          "id": "89d91f5b10bae4c85c923d1a7683b8cbd48f701f",
          "message": "Bump benchmark-action/github-action-benchmark from 1.11.2 to 1.11.3 (#1769)\n\nBumps [benchmark-action/github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) from 1.11.2 to 1.11.3.\n<details>\n<summary>Release notes</summary>\n<p><em>Sourced from <a href=\"https://github.com/benchmark-action/github-action-benchmark/releases\">benchmark-action/github-action-benchmark's releases</a>.</em></p>\n<blockquote>\n<h2>v1.11.3</h2>\n<p>Fix: fix trailing whitespace characters in cargo benchmarks (<a href=\"https://github-redirect.dependabot.com/benchmark-action/github-action-benchmark/issues/97\">#97</a>)</p>\n</blockquote>\n</details>\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a href=\"https://github.com/benchmark-action/github-action-benchmark/blob/master/CHANGELOG.md\">benchmark-action/github-action-benchmark's changelog</a>.</em></p>\n<blockquote>\n<h1><a href=\"https://github.com/benchmark-action/github-action-benchmark/releases/tag/v1.11.3\">v1.11.3</a> - 31 Dec 2021</h1>\n<ul>\n<li><strong>Fix:</strong> Fix trailing whitespace characters in cargo benchmarks (<a href=\"https://github-redirect.dependabot.com/benchmark-action/github-action-benchmark/issues/97\">#97</a>)</li>\n</ul>\n<p><!-- raw HTML omitted --><!-- raw HTML omitted --></p>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a href=\"https://github.com/benchmark-action/github-action-benchmark/commit/1c1a8fb0ca538ff5572ed02039d91a610726c19e\"><code>1c1a8fb</code></a> v1.11.3</li>\n<li>See full diff in <a href=\"https://github.com/benchmark-action/github-action-benchmark/compare/v1.11.2...v1.11.3\">compare view</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility score](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=benchmark-action/github-action-benchmark&package-manager=github_actions&previous-version=1.11.2&new-version=1.11.3)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't alter it yourself. You can also trigger a rebase manually by commenting `@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits that have been made to it\n- `@dependabot merge` will merge this PR after your CI passes on it\n- `@dependabot squash and merge` will squash and merge this PR after your CI passes on it\n- `@dependabot cancel merge` will cancel a previously requested merge and block automerging\n- `@dependabot reopen` will reopen this PR if it is closed\n- `@dependabot close` will close this PR and stop Dependabot recreating it. You can achieve the same result by closing it manually\n- `@dependabot ignore this major version` will close this PR and stop Dependabot creating any more for this major version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop Dependabot creating any more for this minor version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop Dependabot creating any more for this dependency (unless you reopen the PR or upgrade to it yourself)\n\n\n</details>",
          "timestamp": "2022-01-03T10:43:36Z",
          "tree_id": "1a62602b4a1462a602e8e0b3173db58b99ce0e61",
          "url": "https://github.com/boa-dev/boa/commit/89d91f5b10bae4c85c923d1a7683b8cbd48f701f"
        },
        "date": 1641208504045,
        "tool": "cargo",
        "benches": [
          {
            "name": "Create Realm",
            "value": 336,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 4777,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Parser)",
            "value": 15573,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 18322,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Parser)",
            "value": 10580,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Parser)",
            "value": 11305,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Parser)",
            "value": 12182,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Parser)",
            "value": 7237,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Parser)",
            "value": 9600,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 9324,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Parser)",
            "value": 11588,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Parser)",
            "value": 13448,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Parser)",
            "value": 15019,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Parser)",
            "value": 159461,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Parser)",
            "value": 8624,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Parser)",
            "value": 12555,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Parser)",
            "value": 6353,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Parser)",
            "value": 12502,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Parser)",
            "value": 15931,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Parser)",
            "value": 15814,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Parser)",
            "value": 6163,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Parser)",
            "value": 32867,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Parser)",
            "value": 28523,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 803,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2453,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 2821,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Compiler)",
            "value": 1486,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Compiler)",
            "value": 1581,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Compiler)",
            "value": 1891,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Compiler)",
            "value": 1506,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Compiler)",
            "value": 1511,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Compiler)",
            "value": 1819,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 1818,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1454,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Compiler)",
            "value": 2251,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 7293,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Compiler)",
            "value": 1788,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Compiler)",
            "value": 2526,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1261,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Compiler)",
            "value": 1662,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Compiler)",
            "value": 2007,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Compiler)",
            "value": 2451,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Compiler)",
            "value": 990,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 5547,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 5369,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5195,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Execution)",
            "value": 45731,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2841912,
            "range": "± 39004",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Execution)",
            "value": 6369,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Execution)",
            "value": 6545,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Execution)",
            "value": 6974,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Execution)",
            "value": 10047,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Execution)",
            "value": 10086,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Execution)",
            "value": 13027,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Execution)",
            "value": 13093,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Execution)",
            "value": 10699,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Execution)",
            "value": 3191434,
            "range": "± 5358",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1342082,
            "range": "± 6034",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Execution)",
            "value": 6395,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Execution)",
            "value": 7702,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Execution)",
            "value": 5537,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Execution)",
            "value": 5370,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Execution)",
            "value": 6944,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Execution)",
            "value": 8663,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Execution)",
            "value": 2196,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1452493,
            "range": "± 8931",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1346863,
            "range": "± 7419",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "distinct": false,
          "id": "7fba7c0c45c6a79114d01721f41ca27722fdbd5c",
          "message": "Bump indexmap from 1.7.0 to 1.8.0 (#1776)\n\nBumps [indexmap](https://github.com/bluss/indexmap) from 1.7.0 to 1.8.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a href=\"https://github.com/bluss/indexmap/blob/master/RELEASES.rst\">indexmap's changelog</a>.</em></p>\n<blockquote>\n<ul>\n<li>\n<p>1.8.0</p>\n<ul>\n<li>\n<p>The new <code>IndexMap::into_keys</code> and <code>IndexMap::into_values</code> will consume\nthe map into keys or values, respectively, matching Rust 1.54's <code>HashMap</code>\nmethods, by <a href=\"https://github.com/taiki-e\"><code>@​taiki-e</code></a> in PR 195_.</p>\n</li>\n<li>\n<p>More of the iterator types implement <code>Debug</code>, <code>ExactSizeIterator</code>, and\n<code>FusedIterator</code>, by <a href=\"https://github.com/cuviper\"><code>@​cuviper</code></a> in PR 196_.</p>\n</li>\n<li>\n<p><code>IndexMap</code> and <code>IndexSet</code> now implement rayon's <code>ParallelDrainRange</code>,\nby <a href=\"https://github.com/cuviper\"><code>@​cuviper</code></a> in PR 197_.</p>\n</li>\n<li>\n<p><code>IndexMap::with_hasher</code> and <code>IndexSet::with_hasher</code> are now <code>const</code>\nfunctions, allowing static maps and sets, by <a href=\"https://github.com/mwillsey\"><code>@​mwillsey</code></a> in PR 203_.</p>\n</li>\n<li>\n<p><code>IndexMap</code> and <code>IndexSet</code> now implement <code>From</code> for arrays, matching\nRust 1.56's implementation for <code>HashMap</code>, by <a href=\"https://github.com/rouge8\"><code>@​rouge8</code></a> in PR 205_.</p>\n</li>\n<li>\n<p><code>IndexMap</code> and <code>IndexSet</code> now have methods <code>sort_unstable_keys</code>,\n<code>sort_unstable_by</code>, <code>sorted_unstable_by</code>, and <code>par_*</code> equivalents,\nwhich sort in-place without preserving the order of equal items, by\n<a href=\"https://github.com/bhgomes\"><code>@​bhgomes</code></a> in PR 211_.</p>\n</li>\n</ul>\n</li>\n</ul>\n<p>.. _195: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/195\">bluss/indexmap#195</a>\n.. _196: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/196\">bluss/indexmap#196</a>\n.. _197: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/197\">bluss/indexmap#197</a>\n.. _203: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/203\">bluss/indexmap#203</a>\n.. _205: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/205\">bluss/indexmap#205</a>\n.. _211: <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/pull/211\">bluss/indexmap#211</a></p>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a href=\"https://github.com/bluss/indexmap/commit/916d1c96d2070d736c0ab5d5ba294b1c5593f009\"><code>916d1c9</code></a> Merge pull request <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/issues/213\">#213</a> from cuviper/release-1.7.1</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/5386d2bf703f48550f9ac6e03c4e28b09cbc689e\"><code>5386d2b</code></a> Release 1.8.0 instead</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/f090281240c05639c665170a2c633c96adfacc07\"><code>f090281</code></a> Release 1.7.1</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/5a14f7bb8af6e3c8c4fe52bdd2978da07126cbbe\"><code>5a14f7b</code></a> Move recent changes to RELEASES.rst</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/13468f20f51666969b588f0bff7b1749726bf8ca\"><code>13468f2</code></a> Merge pull request <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/issues/211\">#211</a> from bhgomes/add-sort-unstable-methods</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/8bb46ca2e4cc192ab86b6dc80015d8b5a424fe4b\"><code>8bb46ca</code></a> Merge pull request <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/issues/205\">#205</a> from rouge8/from-array</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/6fca269adf18b1dd0ef0e62f5e8744c7cba51725\"><code>6fca269</code></a> No extra space is used in unstable sorts</li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/5d2ce528b3c431722581526b175a51528ae0efa0\"><code>5d2ce52</code></a> Require rustc 1.51+ for <code>IndexMap::from(array)</code> and <code>IndexSet::from(array)</code></li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/f0159f656d95d19b681e63b827538f6d0ca3367b\"><code>f0159f6</code></a> Add <code>IndexMap::from(array)</code> and <code>IndexSet::from(array)</code></li>\n<li><a href=\"https://github.com/bluss/indexmap/commit/4d6dde35b59009e6097a58c6ebbb0cb9b549709d\"><code>4d6dde3</code></a> Merge pull request <a href=\"https://github-redirect.dependabot.com/bluss/indexmap/issues/197\">#197</a> from cuviper/par_drain</li>\n<li>Additional commits viewable in <a href=\"https://github.com/bluss/indexmap/compare/1.7.0...1.8.0\">compare view</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility score](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=indexmap&package-manager=cargo&previous-version=1.7.0&new-version=1.8.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't alter it yourself. You can also trigger a rebase manually by commenting `@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits that have been made to it\n- `@dependabot merge` will merge this PR after your CI passes on it\n- `@dependabot squash and merge` will squash and merge this PR after your CI passes on it\n- `@dependabot cancel merge` will cancel a previously requested merge and block automerging\n- `@dependabot reopen` will reopen this PR if it is closed\n- `@dependabot close` will close this PR and stop Dependabot recreating it. You can achieve the same result by closing it manually\n- `@dependabot ignore this major version` will close this PR and stop Dependabot creating any more for this major version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop Dependabot creating any more for this minor version (unless you reopen the PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop Dependabot creating any more for this dependency (unless you reopen the PR or upgrade to it yourself)\n\n\n</details>",
          "timestamp": "2022-01-11T17:41:09Z",
          "tree_id": "dc87106f2a219587c8f3058d1a7bfbd584e9a42c",
          "url": "https://github.com/boa-dev/boa/commit/7fba7c0c45c6a79114d01721f41ca27722fdbd5c"
        },
        "date": 1641925057949,
        "tool": "cargo",
        "benches": [
          {
            "name": "Create Realm",
            "value": 323,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Parser)",
            "value": 4178,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Parser)",
            "value": 13775,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Parser)",
            "value": 18264,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Parser)",
            "value": 9238,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Parser)",
            "value": 9858,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Parser)",
            "value": 10713,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Parser)",
            "value": 7128,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Parser)",
            "value": 8377,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Parser)",
            "value": 8031,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Parser)",
            "value": 10112,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Parser)",
            "value": 11737,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Parser)",
            "value": 13172,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Parser)",
            "value": 132985,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Parser)",
            "value": 7550,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Parser)",
            "value": 10913,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Parser)",
            "value": 5576,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Parser)",
            "value": 11051,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Parser)",
            "value": 13990,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Parser)",
            "value": 13825,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Parser)",
            "value": 5459,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Parser)",
            "value": 32741,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Parser)",
            "value": 25341,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Compiler)",
            "value": 807,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Compiler)",
            "value": 2152,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Compiler)",
            "value": 2487,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Compiler)",
            "value": 1318,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Compiler)",
            "value": 1404,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Compiler)",
            "value": 1715,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Compiler)",
            "value": 1327,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Compiler)",
            "value": 1326,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Compiler)",
            "value": 1587,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Compiler)",
            "value": 1587,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Compiler)",
            "value": 1287,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Compiler)",
            "value": 2024,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Compiler)",
            "value": 6089,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Compiler)",
            "value": 1574,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Compiler)",
            "value": 2221,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Compiler)",
            "value": 1109,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Compiler)",
            "value": 1476,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Compiler)",
            "value": 1768,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Compiler)",
            "value": 2149,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Compiler)",
            "value": 890,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Compiler)",
            "value": 4940,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Compiler)",
            "value": 4808,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4603,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "For loop (Execution)",
            "value": 41545,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2532937,
            "range": "± 2988",
            "unit": "ns/iter"
          },
          {
            "name": "Object Creation (Execution)",
            "value": 5587,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "Static Object Property Access (Execution)",
            "value": 5775,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Dynamic Object Property Access (Execution)",
            "value": 6135,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal Creation (Execution)",
            "value": 8909,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Creation (Execution)",
            "value": 8940,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp Literal (Execution)",
            "value": 11743,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "RegExp (Execution)",
            "value": 11767,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "Array access (Execution)",
            "value": 9486,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "Array creation (Execution)",
            "value": 3010638,
            "range": "± 7840",
            "unit": "ns/iter"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1229794,
            "range": "± 5700",
            "unit": "ns/iter"
          },
          {
            "name": "String concatenation (Execution)",
            "value": 5668,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "String comparison (Execution)",
            "value": 6760,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "String copy (Execution)",
            "value": 4843,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "Number Object Access (Execution)",
            "value": 4736,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "Boolean Object Access (Execution)",
            "value": 5932,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "String Object Access (Execution)",
            "value": 7672,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "Arithmetic operations (Execution)",
            "value": 1951,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "Clean js (Execution)",
            "value": 1291363,
            "range": "± 12190",
            "unit": "ns/iter"
          },
          {
            "name": "Mini js (Execution)",
            "value": 1197867,
            "range": "± 9974",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}