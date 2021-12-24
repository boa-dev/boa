window.BENCHMARK_DATA = {
  "lastUpdate": 1640353756129,
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
            "range": "+/- 0.060",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2025,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.12,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2572,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.7797,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.8087,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 871.69,
            "range": "+/- 1.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1937,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3697,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8682,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.5299,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 9.4953,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.61,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 12.546,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.6871,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9407,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.8046,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.2573,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1712,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2445,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "",
            "value": 205.94,
            "range": "+/- 0.130",
            "unit": "ns"
          },
          {
            "name": "Clean js (Execution)",
            "value": 674.58,
            "range": "+/- 0.900",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 622.27,
            "range": "+/- 3.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 302.19,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 374.79,
            "range": "+/- 1.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.6486,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 369.73,
            "range": "+/- 0.280",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.9584,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.1816,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 316.93,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "",
            "value": 320.43,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 324.73,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 323.76,
            "range": "+/- 0.180",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 326.86,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 335.58,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 331.05,
            "range": "+/- 0.290",
            "unit": "us"
          },
          {
            "name": "",
            "value": 317.07,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 322.44,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 311.64,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 351.49,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 313.17,
            "range": "+/- 0.410",
            "unit": "us"
          },
          {
            "name": "",
            "value": 360.62,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 299.46,
            "range": "+/- 0.660",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.0563,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.98,
            "range": "+/- 0.980",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1726,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.1067,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.23,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 727.14,
            "range": "+/- 1.810",
            "unit": "ns"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 11.057,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 31.376,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 27.555,
            "range": "+/- 0.083",
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
            "range": "+/- 0.050",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2468,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.269,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2693,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.765,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.7957,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 870.27,
            "range": "+/- 0.420",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2652,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4075,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9129,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.5158,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 9.6632,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.75,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 12.631,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7267,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.952,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1845,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.8782,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1712,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5238,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 225.33,
            "range": "+/- 3.950",
            "unit": "ns"
          },
          {
            "name": "Clean js (Execution)",
            "value": 592.69,
            "range": "+/- 0.790",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 547.7,
            "range": "+/- 0.850",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 301.98,
            "range": "+/- 0.400",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 330.75,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.3322,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 324.53,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.5667,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.3224,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 315.71,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 320.39,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 322.14,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 363.68,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 321.82,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 331.68,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 334.52,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "",
            "value": 352.76,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "",
            "value": 322.26,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 313.35,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 312.2,
            "range": "+/- 0.460",
            "unit": "us"
          },
          {
            "name": "",
            "value": 315.82,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 318.03,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 340.38,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 932.25,
            "range": "+/- 1.030",
            "unit": "us"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.62,
            "range": "+/- 1.020",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5853,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.7354,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.175,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 646.95,
            "range": "+/- 0.210",
            "unit": "ns"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 11.091,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 28.224,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 24.775,
            "range": "+/- 0.009",
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
            "range": "+/- 0.120",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7571,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 16.09,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0001,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.0097,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.472,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 768.75,
            "range": "+/- 0.420",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5689,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4617,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8854,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.6223,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 8.4105,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.184,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 11.256,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7415,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9609,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.7539,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.2704,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2016,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2518,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 234.09,
            "range": "+/- 0.150",
            "unit": "ns"
          },
          {
            "name": "Clean js (Execution)",
            "value": 672.31,
            "range": "+/- 0.900",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 618.61,
            "range": "+/- 1.020",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 342.56,
            "range": "+/- 0.730",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 372.82,
            "range": "+/- 1.070",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 2.663,
            "range": "+/- 0.008",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 371.38,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.9027,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.3203,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 359.75,
            "range": "+/- 0.290",
            "unit": "us"
          },
          {
            "name": "",
            "value": 364.68,
            "range": "+/- 0.280",
            "unit": "us"
          },
          {
            "name": "",
            "value": 367.64,
            "range": "+/- 0.930",
            "unit": "us"
          },
          {
            "name": "",
            "value": 367.35,
            "range": "+/- 0.370",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 368.88,
            "range": "+/- 1.910",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 371.19,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 378.1,
            "range": "+/- 0.300",
            "unit": "us"
          },
          {
            "name": "",
            "value": 352.71,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 364.37,
            "range": "+/- 0.390",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 353.26,
            "range": "+/- 0.320",
            "unit": "us"
          },
          {
            "name": "",
            "value": 351.73,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 314,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 363.34,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 337.43,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.049,
            "range": "+/- 0.002",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 996.58,
            "range": "+/- 1.330",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6143,
            "range": "+/- 0.011",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.7383,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.149,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 647.98,
            "range": "+/- 0.380",
            "unit": "ns"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 9.7356,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 27.686,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 24.281,
            "range": "+/- 0.016",
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
            "range": "+/- 15.910",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.5811,
            "range": "+/- 0.178",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.159,
            "range": "+/- 0.722",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.1741,
            "range": "+/- 0.085",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 9.132,
            "range": "+/- 0.272",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.2127,
            "range": "+/- 0.079",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 993.36,
            "range": "+/- -978.023",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8927,
            "range": "+/- 0.253",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2428,
            "range": "+/- 0.193",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7485,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.82,
            "range": "+/- 0.405",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 11.801,
            "range": "+/- 0.358",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.025,
            "range": "+/- 0.512",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 16.262,
            "range": "+/- 0.431",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.415,
            "range": "+/- 0.215",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.9005,
            "range": "+/- 0.233",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.1516,
            "range": "+/- 0.179",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1751,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4708,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8895,
            "range": "+/- 0.326",
            "unit": "us"
          },
          {
            "name": "",
            "value": 286.69,
            "range": "+/- 11.610",
            "unit": "ns"
          },
          {
            "name": "Clean js (Execution)",
            "value": 806.28,
            "range": "+/- 25.610",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 713.58,
            "range": "+/- 23.360",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 438.21,
            "range": "+/- 12.250",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 480.71,
            "range": "+/- 13.660",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 3.6164,
            "range": "+/- 0.091",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 476.64,
            "range": "+/- 11.690",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.6968,
            "range": "+/- 0.084",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.642,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 469.85,
            "range": "+/- 10.410",
            "unit": "us"
          },
          {
            "name": "",
            "value": 472.35,
            "range": "+/- 11.670",
            "unit": "us"
          },
          {
            "name": "",
            "value": 470.27,
            "range": "+/- 13.810",
            "unit": "us"
          },
          {
            "name": "",
            "value": 473.34,
            "range": "+/- 12.960",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 475.56,
            "range": "+/- 12.470",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 493.47,
            "range": "+/- 13.080",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 493.13,
            "range": "+/- 11.590",
            "unit": "us"
          },
          {
            "name": "",
            "value": 467.05,
            "range": "+/- 10.620",
            "unit": "us"
          },
          {
            "name": "",
            "value": 469.25,
            "range": "+/- 13.700",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 471.83,
            "range": "+/- 10.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 466.59,
            "range": "+/- 9.150",
            "unit": "us"
          },
          {
            "name": "",
            "value": 472.71,
            "range": "+/- 11.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 470.88,
            "range": "+/- 11.960",
            "unit": "us"
          },
          {
            "name": "",
            "value": 444.24,
            "range": "+/- 11.540",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.3081,
            "range": "+/- 0.043",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 1.2401,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 6.2477,
            "range": "+/- 0.289",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.6762,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 17.679,
            "range": "+/- 0.539",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 817.13,
            "range": "+/- 32.670",
            "unit": "ns"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 13.036,
            "range": "+/- 0.504",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 37.76,
            "range": "+/- 1.274",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 33.18,
            "range": "+/- 0.648",
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
            "range": "+/- 6.010",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.5901,
            "range": "+/- 0.192",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.487,
            "range": "+/- 0.644",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9769,
            "range": "+/- 0.080",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 8.585,
            "range": "+/- 0.176",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.9769,
            "range": "+/- 0.071",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 991.5,
            "range": "+/- -978.116",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1103,
            "range": "+/- 0.179",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2562,
            "range": "+/- 0.142",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7305,
            "range": "+/- 0.221",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.971,
            "range": "+/- 0.315",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 12.228,
            "range": "+/- 0.384",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.409,
            "range": "+/- 0.570",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 16.288,
            "range": "+/- 0.312",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5938,
            "range": "+/- 0.224",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.587,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.1595,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9481,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1971,
            "range": "+/- 0.129",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.6638,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 294.45,
            "range": "+/- 10.030",
            "unit": "ns"
          },
          {
            "name": "Clean js (Execution)",
            "value": 734.41,
            "range": "+/- 18.850",
            "unit": "us"
          },
          {
            "name": "Mini js (Execution)",
            "value": 691.43,
            "range": "+/- 24.170",
            "unit": "us"
          },
          {
            "name": "Symbols (Full)",
            "value": 430.47,
            "range": "+/- 12.340",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 478.85,
            "range": "+/- 13.740",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 3.5538,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 454.79,
            "range": "+/- 10.890",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.3446,
            "range": "+/- 0.092",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.6567,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 484.35,
            "range": "+/- 12.800",
            "unit": "us"
          },
          {
            "name": "",
            "value": 476.59,
            "range": "+/- 7.950",
            "unit": "us"
          },
          {
            "name": "",
            "value": 434.58,
            "range": "+/- 10.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 443.17,
            "range": "+/- 15.060",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 464.56,
            "range": "+/- 16.260",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 454.31,
            "range": "+/- 9.500",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 498.01,
            "range": "+/- 16.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 431.8,
            "range": "+/- 12.810",
            "unit": "us"
          },
          {
            "name": "",
            "value": 475.37,
            "range": "+/- 11.840",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 435.97,
            "range": "+/- 8.260",
            "unit": "us"
          },
          {
            "name": "",
            "value": 458.12,
            "range": "+/- 12.320",
            "unit": "us"
          },
          {
            "name": "",
            "value": 454.27,
            "range": "+/- 13.500",
            "unit": "us"
          },
          {
            "name": "",
            "value": 468.44,
            "range": "+/- 10.750",
            "unit": "us"
          },
          {
            "name": "",
            "value": 451.19,
            "range": "+/- 10.420",
            "unit": "us"
          },
          {
            "name": "Clean js (Full)",
            "value": 1.3158,
            "range": "+/- 0.041",
            "unit": "ms"
          },
          {
            "name": "Mini js (Full)",
            "value": 1.2071,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 6.1983,
            "range": "+/- 0.172",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 3.7261,
            "range": "+/- 0.145",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 18.261,
            "range": "+/- 0.477",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 826.1,
            "range": "+/- 24.540",
            "unit": "ns"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 12.479,
            "range": "+/- 0.372",
            "unit": "us"
          },
          {
            "name": "Clean js (Parser)",
            "value": 36.423,
            "range": "+/- 1.191",
            "unit": "us"
          },
          {
            "name": "Mini js (Parser)",
            "value": 31.451,
            "range": "+/- 0.729",
            "unit": "us"
          }
        ]
      }
    ]
  }
}