window.BENCHMARK_DATA = {
  "lastUpdate": 1581792026075,
  "repoUrl": "https://github.com/jasonwilliams/boa",
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
          "distinct": true,
          "id": "55b3f1dc3d9f15288e240b36ca86143744a2030e",
          "message": "change title",
          "timestamp": "2020-01-20T23:24:00Z",
          "tree_id": "0213da72485228468d0a03e0ee78f08a68dd9826",
          "url": "https://github.com/jasonwilliams/boa/commit/55b3f1dc3d9f15288e240b36ca86143744a2030e"
        },
        "date": 1579563057830,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 440.27,
            "range": "+/- 12.140",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 522.82,
            "range": "+/- 10.790",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.2155,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.036,
            "range": "+/- 0.207",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 460.25,
            "range": "+/- 11.310",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0539,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3541,
            "range": "+/- 0.026",
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
          "id": "495f0a48686b362613b0befc8a6e8a91563a81f6",
          "message": "String.prototype.replace() (#217)\n\n* String Replace addition\r\n\r\n* Function argument now fully implemented\r\n\r\n* adding substitutions\r\n\r\n* finish off String.prototype.replace\r\n\r\n* use is_some()\r\n\r\n* fixing string\r\n\r\n* clippy",
          "timestamp": "2020-01-20T23:57:18Z",
          "tree_id": "92f8653a94efe6bcd11c24d67566851a703e8fdf",
          "url": "https://github.com/jasonwilliams/boa/commit/495f0a48686b362613b0befc8a6e8a91563a81f6"
        },
        "date": 1579565022755,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 414.6,
            "range": "+/- 6.690",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 501.42,
            "range": "+/- 8.400",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.1035,
            "range": "+/- 0.067",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.5606,
            "range": "+/- 0.145",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 434.01,
            "range": "+/- 7.200",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0516,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0471,
            "range": "+/- 0.018",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "vrafaeli@msn.com",
            "name": "croraf",
            "username": "croraf"
          },
          "committer": {
            "email": "936006+jasonwilliams@users.noreply.github.com",
            "name": "Jason Williams",
            "username": "jasonwilliams"
          },
          "distinct": true,
          "id": "eaeb299a9e8f6ca9cad4ba237af74c161b5e5120",
          "message": "Fix lexing of \"0_\" token (#231)\n\n* Fix lexing of 0_ token\r\n* Fix bugs and return to non-strict\r\n* Extract read_integer_in_base",
          "timestamp": "2020-01-21T21:35:34Z",
          "tree_id": "2ca3fc540ddca94c0fa9d4f4c884a3b7c8998922",
          "url": "https://github.com/jasonwilliams/boa/commit/eaeb299a9e8f6ca9cad4ba237af74c161b5e5120"
        },
        "date": 1579642938885,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 385.58,
            "range": "+/- 6.000",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 464.32,
            "range": "+/- 6.890",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.6512,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 8.9589,
            "range": "+/- 0.139",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 405.76,
            "range": "+/- 7.370",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 901.86,
            "range": "+/- 17.520",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 970.06,
            "range": "+/- 23.050",
            "unit": "ns"
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
          "id": "fe4a889e1e5876d66a6ce636fd69c1012cb98d20",
          "message": "fix clippy, revert to just correctness, perf and style (#232)",
          "timestamp": "2020-01-21T22:47:12Z",
          "tree_id": "dc7a858c5c1e23293d3a213f78755736fc4850e5",
          "url": "https://github.com/jasonwilliams/boa/commit/fe4a889e1e5876d66a6ce636fd69c1012cb98d20"
        },
        "date": 1579647258565,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 428.02,
            "range": "+/- 7.290",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 510.95,
            "range": "+/- 9.400",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.2585,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.096,
            "range": "+/- 0.256",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 451.28,
            "range": "+/- 8.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0493,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0932,
            "range": "+/- 0.008",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "csacxc@gmail.com",
            "name": "cisen",
            "username": "cisen"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d8f33abe06e49ad937c0b4cc203eefee7803cb63",
          "message": "fix: Array.prototype.toString should be called by ES value (#227)\n\n* feat: Implement Array.prototype.toString\r\n\r\n* fix: fix the missing arguments for Array.prototype.toString's inner join\r\n\r\n* refactor: use fmt to beautify the code\r\n\r\n* refactor: Array.prototype.toString——smplify error formating\r\n\r\n* fix: Array.prototype.toString should be called by ES value\r\n\r\n* fix: fix the error message\r\n\r\n* refactor: Array.prototype.toString remove the duplicated logic",
          "timestamp": "2020-01-31T06:55:52+02:00",
          "tree_id": "0605bbc174c46603f8a2ef846bb4e3a987d6d133",
          "url": "https://github.com/jasonwilliams/boa/commit/d8f33abe06e49ad937c0b4cc203eefee7803cb63"
        },
        "date": 1580446939985,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 380.07,
            "range": "+/- 9.110",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 451.89,
            "range": "+/- 8.360",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.4964,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 8.998,
            "range": "+/- 0.213",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 408.4,
            "range": "+/- 9.150",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 989.45,
            "range": "+/- 15.630",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0767,
            "range": "+/- 0.022",
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
          "id": "33490e1edb99cd867133a2a02381e64afe787710",
          "message": "updating clippy rules on all files (#238)",
          "timestamp": "2020-02-02T00:31:00Z",
          "tree_id": "acb58a23afd3b59b3b9e1116551d0b43cf76fce7",
          "url": "https://github.com/jasonwilliams/boa/commit/33490e1edb99cd867133a2a02381e64afe787710"
        },
        "date": 1580603863145,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 434.04,
            "range": "+/- 3.020",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 514.12,
            "range": "+/- 7.240",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.019,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.9598,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 447.9,
            "range": "+/- 2.750",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0902,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1436,
            "range": "+/- 0.007",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "nathaniel.daniel23@outlook.com",
            "name": "Nathaniel",
            "username": "adumbidiot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6947122815f33b57b51062720380ca9ae68b47ad",
          "message": "Fixed compilation without \"wasm-bindgen\" feature (#236)\n\n* Fixed compilation without \"wasm-bindgen\" feature\r\n\r\n* updating clippy rules on all files (#238)\r\n\r\n* Fixed compilation without \"wasm-bindgen\" feature\r\n\r\nCo-authored-by: Jason Williams <936006+jasonwilliams@users.noreply.github.com>",
          "timestamp": "2020-02-02T13:40:08Z",
          "tree_id": "ea6b2111b9065bbe66f02ec12c28f47f22d99532",
          "url": "https://github.com/jasonwilliams/boa/commit/6947122815f33b57b51062720380ca9ae68b47ad"
        },
        "date": 1580651217138,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 432.61,
            "range": "+/- 4.950",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 511.42,
            "range": "+/- 4.910",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.037,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.015,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 450.17,
            "range": "+/- 4.160",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0441,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1718,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "nathaniel.daniel23@outlook.com",
            "name": "Nathaniel",
            "username": "adumbidiot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "18523c57f1d1b0cca2854010fc93f0d11649b49f",
          "message": "Fixed some panics in the lexer (#242)\n\n* Fixed some panics in the lexer\r\n* Applied Requested Fixes\r\n* Applied Requested Fixes\r\n* Gave `ParseError` a basic `Display` impl",
          "timestamp": "2020-02-04T10:32:31Z",
          "tree_id": "f45053aaa1b4376cdd37da79620a4faa020a2b8e",
          "url": "https://github.com/jasonwilliams/boa/commit/18523c57f1d1b0cca2854010fc93f0d11649b49f"
        },
        "date": 1580812755392,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 417.2,
            "range": "+/- 3.800",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 491.62,
            "range": "+/- 4.340",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.8759,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.4553,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 426.69,
            "range": "+/- 5.980",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0207,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1218,
            "range": "+/- 0.011",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "alexandrego2003@gmail.com",
            "name": "Alexandre GOMES",
            "username": "gomesalexandre"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "448835295a1cb2cbb216c0459759f208e132606c",
          "message": "fix addition/subtraction with no space (#235)",
          "timestamp": "2020-02-04T21:25:26Z",
          "tree_id": "ceff7d1b5d38a9303ddedc599a3a819ac09ffc55",
          "url": "https://github.com/jasonwilliams/boa/commit/448835295a1cb2cbb216c0459759f208e132606c"
        },
        "date": 1580851931885,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 419.56,
            "range": "+/- 5.420",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 485.76,
            "range": "+/- 5.980",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.9043,
            "range": "+/- 0.062",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.508,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 435.12,
            "range": "+/- 5.520",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0291,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3025,
            "range": "+/- 0.025",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "3e48f54ca5cd970c927e2723b40dc998bd8038f6",
          "message": "rust-lldb is no longer needed, sourcemaps should move into launch.json, rust-analyzer is now in the marketplace",
          "timestamp": "2020-02-10T22:54:37Z",
          "tree_id": "3481f7681585ca5b24766efff03115b47ca9b78d",
          "url": "https://github.com/jasonwilliams/boa/commit/3e48f54ca5cd970c927e2723b40dc998bd8038f6"
        },
        "date": 1581375689851,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 402.81,
            "range": "+/- 8.440",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 496.77,
            "range": "+/- 12.170",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.8743,
            "range": "+/- 0.081",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.0869,
            "range": "+/- 0.212",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 423.19,
            "range": "+/- 11.480",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 954.83,
            "range": "+/- 34.010",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0479,
            "range": "+/- 0.017",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "nathaniel.daniel23@outlook.com",
            "name": "Nathaniel",
            "username": "adumbidiot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "080a3359fd73c8e8eb0bab16a26434b0aa8b93e6",
          "message": "Fixed parsing of floats with scientific notation (#245)\n\n* Fixed parsing of scientific notation with floats\r\n\r\n* Reorganize tests",
          "timestamp": "2020-02-10T23:31:29Z",
          "tree_id": "39e494f20bae8826551725f70111969dd1492b41",
          "url": "https://github.com/jasonwilliams/boa/commit/080a3359fd73c8e8eb0bab16a26434b0aa8b93e6"
        },
        "date": 1581377866687,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 398.63,
            "range": "+/- 6.210",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 469.89,
            "range": "+/- 6.150",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.8164,
            "range": "+/- 0.078",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.4264,
            "range": "+/- 0.145",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 423.9,
            "range": "+/- 6.670",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 956.83,
            "range": "+/- 15.510",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2632,
            "range": "+/- 0.026",
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
          "id": "3f83d17d3420aaa0555832d70b0f0f860b3fb1d6",
          "message": "Update benchmark.yml",
          "timestamp": "2020-02-10T23:53:56Z",
          "tree_id": "f7114bf89a9c9a99e497b6d61751b8e209416b6c",
          "url": "https://github.com/jasonwilliams/boa/commit/3f83d17d3420aaa0555832d70b0f0f860b3fb1d6"
        },
        "date": 1581379235583,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 395.16,
            "range": "+/- 12.250",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 475.9,
            "range": "+/- 14.740",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.7477,
            "range": "+/- 0.104",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.2429,
            "range": "+/- 0.256",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 431.07,
            "range": "+/- 13.340",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 925.03,
            "range": "+/- 30.860",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 978.26,
            "range": "+/- 31.030",
            "unit": "ns"
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
          "id": "cb850fc13e94e1baec09267bd010a4cd4565d73d",
          "message": "Update pull_request.yml\n\nhttps://github.com/jasonwilliams/boa/pull/247#issuecomment-585474183",
          "timestamp": "2020-02-12T23:40:57Z",
          "tree_id": "c1762b0d5eca363b6376815474635f9dbbc628a8",
          "url": "https://github.com/jasonwilliams/boa/commit/cb850fc13e94e1baec09267bd010a4cd4565d73d"
        },
        "date": 1581551213242,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 329.65,
            "range": "+/- 10.130",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 388.25,
            "range": "+/- 10.910",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 3.942,
            "range": "+/- 0.098",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 8.2087,
            "range": "+/- 0.497",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 377.23,
            "range": "+/- 11.980",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 852.26,
            "range": "+/- 31.540",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 934.21,
            "range": "+/- 35.880",
            "unit": "ns"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "iban.eguia@cern.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5f6e4c22c17c9bd60768e03456d4bf1347e5f7f4",
          "message": "Moved to a workspace architecture (#247)\n\n* Moved to a workspace architecture",
          "timestamp": "2020-02-14T11:28:59Z",
          "tree_id": "94b8fd57c3202f3d9293cb687f74b5bbdd4becf2",
          "url": "https://github.com/jasonwilliams/boa/commit/5f6e4c22c17c9bd60768e03456d4bf1347e5f7f4"
        },
        "date": 1581680101503,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 433.38,
            "range": "+/- 5.100",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 514.56,
            "range": "+/- 5.700",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.2958,
            "range": "+/- 0.079",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.922,
            "range": "+/- 0.149",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 458.81,
            "range": "+/- 10.880",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0578,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3612,
            "range": "+/- 0.020",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jwilliams720@bloomberg.net",
            "name": "Jason Williams"
          },
          "committer": {
            "email": "jwilliams720@bloomberg.net",
            "name": "Jason Williams"
          },
          "distinct": true,
          "id": "019033eff066e8c6ba9456139690eb214a0bf61d",
          "message": "cargo update",
          "timestamp": "2020-02-14T12:34:45Z",
          "tree_id": "d16f7eb8eee5bc7ee3c54da5db4deffe636b30d6",
          "url": "https://github.com/jasonwilliams/boa/commit/019033eff066e8c6ba9456139690eb214a0bf61d"
        },
        "date": 1581684055139,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 416.79,
            "range": "+/- 3.900",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 495.44,
            "range": "+/- 7.350",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.0911,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.154,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 430.49,
            "range": "+/- 4.580",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0047,
            "range": "+/- -998.899",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3107,
            "range": "+/- 0.020",
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
          "id": "0cb6d7403ef09920d37f4a465d6068d714e4b5a2",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-02-14T16:22:56Z",
          "tree_id": "c9f625f1ebcedd790d0edc3cca040c217b5c7415",
          "url": "https://github.com/jasonwilliams/boa/commit/0cb6d7403ef09920d37f4a465d6068d714e4b5a2"
        },
        "date": 1581697703333,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 386.61,
            "range": "+/- 4.580",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 456.58,
            "range": "+/- 4.580",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.5115,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.7483,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 396.79,
            "range": "+/- 4.510",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 993.29,
            "range": "+/- -986.009",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2969,
            "range": "+/- 0.018",
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
          "id": "fbede2142e2cf89be847fadd342023cc3be79d6a",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-02-15T00:13:13Z",
          "tree_id": "a9e84dc16a438ce186f33f63c603c08942e043f1",
          "url": "https://github.com/jasonwilliams/boa/commit/fbede2142e2cf89be847fadd342023cc3be79d6a"
        },
        "date": 1581725933852,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 389.44,
            "range": "+/- 12.480",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 468.68,
            "range": "+/- 10.250",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.7231,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.507,
            "range": "+/- 0.289",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 411.62,
            "range": "+/- 12.550",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 888.88,
            "range": "+/- 22.580",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1593,
            "range": "+/- 0.028",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "b2e3bc0f64579b634c8dd6e0b6599e9d32f0c031",
          "message": "attempting to use cache to speed up benchmarks PR",
          "timestamp": "2020-02-15T16:33:23Z",
          "tree_id": "ef7a8cdefa0fd3db5c38052995322b2820b0f553",
          "url": "https://github.com/jasonwilliams/boa/commit/b2e3bc0f64579b634c8dd6e0b6599e9d32f0c031"
        },
        "date": 1581784766086,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 425.75,
            "range": "+/- 8.690",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 499.75,
            "range": "+/- 9.590",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.0297,
            "range": "+/- 0.066",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.377,
            "range": "+/- 0.234",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 447.27,
            "range": "+/- 8.270",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0035,
            "range": "+/- -992.295",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.274,
            "range": "+/- 0.026",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "fce25be382d6c41a56cecd6479bb7ad83df25e82",
          "message": "cache after files have been stored",
          "timestamp": "2020-02-15T16:38:35Z",
          "tree_id": "3ae7c5d1f2f8c844044b7e2afb3bea7fbccb4201",
          "url": "https://github.com/jasonwilliams/boa/commit/fce25be382d6c41a56cecd6479bb7ad83df25e82"
        },
        "date": 1581785030831,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 384.05,
            "range": "+/- 11.730",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 445.14,
            "range": "+/- 13.940",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.2843,
            "range": "+/- 0.082",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.0477,
            "range": "+/- 0.316",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 397.36,
            "range": "+/- 12.320",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 880.19,
            "range": "+/- 30.020",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1163,
            "range": "+/- 0.046",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "046f68f6f958d17b052c7bc54aeeba5b3a8aba57",
          "message": "adding some logging of output files",
          "timestamp": "2020-02-15T17:09:21Z",
          "tree_id": "10f5c40f4e44123f9d0944d6ffa406a7b3e121d9",
          "url": "https://github.com/jasonwilliams/boa/commit/046f68f6f958d17b052c7bc54aeeba5b3a8aba57"
        },
        "date": 1581786901837,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 320.9,
            "range": "+/- 10.140",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 379,
            "range": "+/- 10.550",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 3.8278,
            "range": "+/- 0.094",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 8.4283,
            "range": "+/- 0.206",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 335.67,
            "range": "+/- 8.320",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 799.02,
            "range": "+/- 19.950",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0704,
            "range": "+/- 0.056",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "0fa4003f718b35702b38d319357b43b6821ded27",
          "message": "compare action",
          "timestamp": "2020-02-15T18:10:17Z",
          "tree_id": "59328990a5331699166f7a1bf131daf50c006d51",
          "url": "https://github.com/jasonwilliams/boa/commit/0fa4003f718b35702b38d319357b43b6821ded27"
        },
        "date": 1581790572815,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 426.97,
            "range": "+/- 4.480",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 502.77,
            "range": "+/- 4.410",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 5.0174,
            "range": "+/- 0.044",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.885,
            "range": "+/- 0.143",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 444.02,
            "range": "+/- 8.420",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0942,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.4693,
            "range": "+/- 0.017",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "6333e7b9ccc3ab6f423d869762ebf3b24b153636",
          "message": "back to debugging",
          "timestamp": "2020-02-15T18:20:49Z",
          "tree_id": "10f5c40f4e44123f9d0944d6ffa406a7b3e121d9",
          "url": "https://github.com/jasonwilliams/boa/commit/6333e7b9ccc3ab6f423d869762ebf3b24b153636"
        },
        "date": 1581791178441,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 380.11,
            "range": "+/- 7.750",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 451.49,
            "range": "+/- 7.690",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.5577,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.103,
            "range": "+/- 0.328",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 412.98,
            "range": "+/- 10.770",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 966.66,
            "range": "+/- 18.320",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2899,
            "range": "+/- 0.034",
            "unit": "us"
          }
        ]
      },
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
          "distinct": true,
          "id": "95899e9bc5dfedd55611cdf09cfcf899bab9b8c7",
          "message": "weird caching happening, even when pointing to specific commit",
          "timestamp": "2020-02-15T18:34:46Z",
          "tree_id": "36bbb6e3db9b718113eb95b578f2cd8adc4921a5",
          "url": "https://github.com/jasonwilliams/boa/commit/95899e9bc5dfedd55611cdf09cfcf899bab9b8c7"
        },
        "date": 1581792024069,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 399.91,
            "range": "+/- 3.770",
            "unit": "us"
          },
          {
            "name": "Symbol Creation",
            "value": 473.36,
            "range": "+/- 5.400",
            "unit": "us"
          },
          {
            "name": "fibonacci (Execution)",
            "value": 4.6101,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.211,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 408.2,
            "range": "+/- 3.570",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 962.93,
            "range": "+/- 10.010",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2935,
            "range": "+/- 0.014",
            "unit": "us"
          }
        ]
      }
    ]
  }
}