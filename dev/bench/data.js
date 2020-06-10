window.BENCHMARK_DATA = {
  "lastUpdate": 1591808237763,
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
            "name": "Symbols (Execution)",
            "value": 522.82,
            "range": "+/- 10.790",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 501.42,
            "range": "+/- 8.400",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 464.32,
            "range": "+/- 6.890",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 510.95,
            "range": "+/- 9.400",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 451.89,
            "range": "+/- 8.360",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 514.12,
            "range": "+/- 7.240",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 511.42,
            "range": "+/- 4.910",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 491.62,
            "range": "+/- 4.340",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 485.76,
            "range": "+/- 5.980",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 496.77,
            "range": "+/- 12.170",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 469.89,
            "range": "+/- 6.150",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 475.9,
            "range": "+/- 14.740",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 388.25,
            "range": "+/- 10.910",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 514.56,
            "range": "+/- 5.700",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 495.44,
            "range": "+/- 7.350",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 456.58,
            "range": "+/- 4.580",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 468.68,
            "range": "+/- 10.250",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 499.75,
            "range": "+/- 9.590",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 445.14,
            "range": "+/- 13.940",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 379,
            "range": "+/- 10.550",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 502.77,
            "range": "+/- 4.410",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 451.49,
            "range": "+/- 7.690",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
            "name": "Symbols (Execution)",
            "value": 473.36,
            "range": "+/- 5.400",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
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
          "id": "5c07b20113021070be45e6cd376009843675d7b6",
          "message": "i did'nt build",
          "timestamp": "2020-02-15T18:47:31Z",
          "tree_id": "10f5c40f4e44123f9d0944d6ffa406a7b3e121d9",
          "url": "https://github.com/jasonwilliams/boa/commit/5c07b20113021070be45e6cd376009843675d7b6"
        },
        "date": 1581792801202,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 384.04,
            "range": "+/- 4.630",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 460.45,
            "range": "+/- 6.340",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.4916,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.8203,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 399.19,
            "range": "+/- 9.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 959.43,
            "range": "+/- 6.510",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2669,
            "range": "+/- 0.011",
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
          "id": "e9428807f2fd13a009a922522b49819b7cb6d802",
          "message": "should now be able to use master",
          "timestamp": "2020-02-15T19:36:56Z",
          "tree_id": "8641cf7811ce002f08a85a21e2ae713912b8682a",
          "url": "https://github.com/jasonwilliams/boa/commit/e9428807f2fd13a009a922522b49819b7cb6d802"
        },
        "date": 1581795795671,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 410.48,
            "range": "+/- 5.170",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 491.72,
            "range": "+/- 9.550",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.8164,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.24,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 424.47,
            "range": "+/- 3.840",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 992.66,
            "range": "+/- 13.210",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.361,
            "range": "+/- 0.014",
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
          "id": "940da7bf85a03e4420f928a64ae98870d18359a2",
          "message": "feat: Implement Array.isArray (#253)",
          "timestamp": "2020-02-16T15:04:01Z",
          "tree_id": "846e507e722f5a2ebac123cde4267564c3b0e68e",
          "url": "https://github.com/jasonwilliams/boa/commit/940da7bf85a03e4420f928a64ae98870d18359a2"
        },
        "date": 1581865786386,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 396.87,
            "range": "+/- 8.410",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 462.62,
            "range": "+/- 9.990",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.5524,
            "range": "+/- 0.088",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.737,
            "range": "+/- 0.314",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 404.55,
            "range": "+/- 6.390",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0091,
            "range": "+/- -996.709",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2925,
            "range": "+/- 0.031",
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
          "id": "4c18f3acda5b84512303f7a74d63f7432c23405e",
          "message": "updating launch.json for workspace setup",
          "timestamp": "2020-02-17T23:08:54Z",
          "tree_id": "673304a37b2448f46c244df31d9d8a20cb5cf8d0",
          "url": "https://github.com/jasonwilliams/boa/commit/4c18f3acda5b84512303f7a74d63f7432c23405e"
        },
        "date": 1581981294937,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 429.73,
            "range": "+/- 11.210",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 486.65,
            "range": "+/- 10.600",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.8165,
            "range": "+/- 0.074",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.095,
            "range": "+/- 0.205",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 438.74,
            "range": "+/- 8.970",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0332,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2988,
            "range": "+/- 0.021",
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
          "id": "686d17a0029f6dd76ec4fc9eeda92e6fdae47b7f",
          "message": "creating trait for object-internal-methods",
          "timestamp": "2020-02-19T00:34:34Z",
          "tree_id": "0a5f03f99c40e8fdee4a045cf892f0c77d7074dc",
          "url": "https://github.com/jasonwilliams/boa/commit/686d17a0029f6dd76ec4fc9eeda92e6fdae47b7f"
        },
        "date": 1582072790570,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 345.59,
            "range": "+/- 9.230",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 406.13,
            "range": "+/- 10.670",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.7569,
            "range": "+/- 0.067",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 7.6245,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 324.07,
            "range": "+/- 6.160",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 813.65,
            "range": "+/- 15.310",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0341,
            "range": "+/- 0.029",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "contact@alexandregomes.fr",
            "name": "Alexandre GOMES",
            "username": "gomesalexandre"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "92a63b20ea07ddbc2dfadac6a0b8096915893044",
          "message": "fix(parser): handle trailing comma in object literals (#249)",
          "timestamp": "2020-02-19T00:59:32Z",
          "tree_id": "ad1160b7f5c87623a8c287f2769d358480b90651",
          "url": "https://github.com/jasonwilliams/boa/commit/92a63b20ea07ddbc2dfadac6a0b8096915893044"
        },
        "date": 1582074334716,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 400.15,
            "range": "+/- 6.580",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 461.66,
            "range": "+/- 5.930",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.5279,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.006,
            "range": "+/- 0.128",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 412.64,
            "range": "+/- 7.390",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0231,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2768,
            "range": "+/- 0.019",
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
          "id": "edab5ca6cc10d13265f82fa4bc05d6b432a362fc",
          "message": "Removing debug output, switch to normal",
          "timestamp": "2020-02-19T01:06:25Z",
          "tree_id": "5e48d600ad1bd4feac219dd498c833469ca98e14",
          "url": "https://github.com/jasonwilliams/boa/commit/edab5ca6cc10d13265f82fa4bc05d6b432a362fc"
        },
        "date": 1582074745618,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 423.87,
            "range": "+/- 6.870",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 494.48,
            "range": "+/- 7.400",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.7885,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.437,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 431.07,
            "range": "+/- 3.790",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0731,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3455,
            "range": "+/- 0.012",
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
          "id": "fd616c887b312166da498904201e6cede9fa6fd8",
          "message": "Fixed more Lexer Panics (#244)\n\n* Fixed more Lexer Panics",
          "timestamp": "2020-02-20T13:02:40Z",
          "tree_id": "78f0157e22038b36509241020f2e302bbde5afb3",
          "url": "https://github.com/jasonwilliams/boa/commit/fd616c887b312166da498904201e6cede9fa6fd8"
        },
        "date": 1582205539110,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 417.49,
            "range": "+/- 4.720",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 491.1,
            "range": "+/- 5.220",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.8435,
            "range": "+/- 0.044",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.514,
            "range": "+/- 0.132",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 435.61,
            "range": "+/- 4.060",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0961,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3506,
            "range": "+/- 0.015",
            "unit": "us"
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
          "id": "12c99e16581fd0ef9069ea108d52978dfd47f525",
          "message": "Fixed comments lexing (#256)",
          "timestamp": "2020-02-24T17:53:20Z",
          "tree_id": "c278f802ed1f12f60205231142703e8239ec3910",
          "url": "https://github.com/jasonwilliams/boa/commit/12c99e16581fd0ef9069ea108d52978dfd47f525"
        },
        "date": 1582567109777,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 358.41,
            "range": "+/- 11.050",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 416.6,
            "range": "+/- 10.270",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.1218,
            "range": "+/- 0.071",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.6997,
            "range": "+/- 0.288",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 372.46,
            "range": "+/- 9.570",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0577,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0898,
            "range": "+/- 0.030",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "86052d6d75d7ac321e9b6b83dbf3bf2f2377437f",
          "message": "Moved test modules to their own files (#258)",
          "timestamp": "2020-02-26T22:33:59Z",
          "tree_id": "74cd715c4a9027f59fb50c8436e92b50601bfce5",
          "url": "https://github.com/jasonwilliams/boa/commit/86052d6d75d7ac321e9b6b83dbf3bf2f2377437f"
        },
        "date": 1582756792427,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 441,
            "range": "+/- 19.210",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 542.76,
            "range": "+/- 18.830",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.9277,
            "range": "+/- 0.155",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.469,
            "range": "+/- 0.592",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 443.78,
            "range": "+/- 19.270",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0924,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2579,
            "range": "+/- 0.062",
            "unit": "us"
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
          "id": "f628f4cc8cd5be5af3430339be25086ee2975c2c",
          "message": "Fixed empty returns (#251) (#257)",
          "timestamp": "2020-03-06T20:58:27Z",
          "tree_id": "d89f2687a30f42e4283919d8ba2034a48e952a95",
          "url": "https://github.com/jasonwilliams/boa/commit/f628f4cc8cd5be5af3430339be25086ee2975c2c"
        },
        "date": 1583528657386,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 435.85,
            "range": "+/- 10.090",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 515.29,
            "range": "+/- 10.640",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.091,
            "range": "+/- 0.065",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.51,
            "range": "+/- 0.128",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 464.27,
            "range": "+/- 11.830",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0899,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2766,
            "range": "+/- 0.023",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "doneth7@gmail.com",
            "name": "John Doneth",
            "username": "JohnDoneth"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d92da39299e8eb5810b2274eab9ab10dae2cbbe3",
          "message": "Add print & REPL functionality to CLI (#267)\n\n* Add basic REPL functionality\r\n* Add utility function to Realm\r\n* Rework flow to allow files to be loaded as well as open a shell\r\n* Remove shell option (not needed now its the default)\r\n* Update README.md & docs/debugging.md",
          "timestamp": "2020-03-08T17:54:57Z",
          "tree_id": "be197b050c80156297f487e56abac52efabf2e18",
          "url": "https://github.com/jasonwilliams/boa/commit/d92da39299e8eb5810b2274eab9ab10dae2cbbe3"
        },
        "date": 1583690480295,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 463.16,
            "range": "+/- 8.840",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 577.42,
            "range": "+/- 16.740",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.7029,
            "range": "+/- 0.081",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 11.563,
            "range": "+/- 0.241",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 502.14,
            "range": "+/- 13.150",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.2068,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.4145,
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
          "id": "9766409c521af12f93ec53b68169fdafd5bd5b21",
          "message": "Addition of forEach() (#268)\n\n* Addition of forEach()\r\n* fixing LLDB launcher for windows (it needs .exe to work for windows)",
          "timestamp": "2020-03-08T21:45:24Z",
          "tree_id": "9b4f64f4c833da46d4ce90bbed83c56691b3fb61",
          "url": "https://github.com/jasonwilliams/boa/commit/9766409c521af12f93ec53b68169fdafd5bd5b21"
        },
        "date": 1583704269946,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 423.31,
            "range": "+/- 6.340",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 495.75,
            "range": "+/- 5.050",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.7891,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.477,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 434.03,
            "range": "+/- 2.470",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0174,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3809,
            "range": "+/- 0.010",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hello@nickforall.nl",
            "name": "Nick Vernij",
            "username": "Nickforall"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6fa8d484a9a3d73cec1a30eeb941fb3b5f7df917",
          "message": "Implement Array.prototype.filter (#262)",
          "timestamp": "2020-03-09T13:08:19Z",
          "tree_id": "29cbab1a119c954beca015d2152f8c818372b830",
          "url": "https://github.com/jasonwilliams/boa/commit/6fa8d484a9a3d73cec1a30eeb941fb3b5f7df917"
        },
        "date": 1583759645598,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 416.34,
            "range": "+/- 2.290",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 495.34,
            "range": "+/- 5.160",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.7272,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.417,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 430.8,
            "range": "+/- 2.760",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.03,
            "range": "+/- 0.007",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3532,
            "range": "+/- 0.014",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hello@nickforall.nl",
            "name": "Nick Vernij",
            "username": "Nickforall"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2d5bf5595665d65c25ffb7fcde8047819745acba",
          "message": "Fix parsing of floats that start with a zero (#272)",
          "timestamp": "2020-03-13T00:04:39Z",
          "tree_id": "6306a8daed70b757edcc8a17d6d491c2465e61e7",
          "url": "https://github.com/jasonwilliams/boa/commit/2d5bf5595665d65c25ffb7fcde8047819745acba"
        },
        "date": 1584058230605,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 405.57,
            "range": "+/- 5.710",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 477.56,
            "range": "+/- 5.450",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.6292,
            "range": "+/- 0.036",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.244,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 415.06,
            "range": "+/- 4.830",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 993.17,
            "range": "+/- -983.928",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.284,
            "range": "+/- 0.023",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "62383f5a06e9e4c59a10d550b6bf9ab53cb042b4",
          "message": "Bump acorn from 6.4.0 to 6.4.1 (#275)\n\nBumps [acorn](https://github.com/acornjs/acorn) from 6.4.0 to 6.4.1.\r\n- [Release notes](https://github.com/acornjs/acorn/releases)\r\n- [Commits](https://github.com/acornjs/acorn/compare/6.4.0...6.4.1)\r\n\r\nSigned-off-by: dependabot[bot] <support@github.com>\r\n\r\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2020-03-18T11:54:34Z",
          "tree_id": "2f698f1ab29a1ee6357151e0eb0c1eb3346d09d3",
          "url": "https://github.com/jasonwilliams/boa/commit/62383f5a06e9e4c59a10d550b6bf9ab53cb042b4"
        },
        "date": 1584532825844,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 400.05,
            "range": "+/- 10.540",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 446.86,
            "range": "+/- 6.750",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.6384,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 9.383,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 393.56,
            "range": "+/- 6.040",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 955.7,
            "range": "+/- 32.100",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.202,
            "range": "+/- 0.032",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f53b352a4e6984ff8e8a6bcc164707cbd2842227",
          "message": "Added a logo to the project. (#277)\n\n* Added a logo to the project.\r\n\r\n* Changed the logo from a png to a svg.",
          "timestamp": "2020-03-18T18:56:01Z",
          "tree_id": "98c090568d96a83e63a7379b42619e5b6ba64535",
          "url": "https://github.com/jasonwilliams/boa/commit/f53b352a4e6984ff8e8a6bcc164707cbd2842227"
        },
        "date": 1584558136891,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 459.24,
            "range": "+/- 11.140",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 534.57,
            "range": "+/- 23.340",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.3335,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 11.069,
            "range": "+/- 0.232",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 456.34,
            "range": "+/- 6.230",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0626,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3489,
            "range": "+/- 0.013",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hello@nickforall.nl",
            "name": "Nick Vernij",
            "username": "Nickforall"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9b8c803bbe95b2a3d437893c0962f98a8a58585b",
          "message": "Add methods with f64 std equivelant to Math object (#260)\n\n* Add methods with f64 std equivelant to Math object\r\n* Add testS for Math static methods",
          "timestamp": "2020-03-18T22:05:29Z",
          "tree_id": "5268267b0d9b35ac1f8c27caf2658f11f2cb3c2f",
          "url": "https://github.com/jasonwilliams/boa/commit/9b8c803bbe95b2a3d437893c0962f98a8a58585b"
        },
        "date": 1584569461649,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 404.56,
            "range": "+/- 5.440",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 505.87,
            "range": "+/- 7.880",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.5497,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 10.146,
            "range": "+/- 0.196",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 420.23,
            "range": "+/- 7.620",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0443,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.36,
            "range": "+/- 0.043",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5a85c595d4dff8fffd3d7881e4e9bca188691074",
          "message": "Added the ability to dump the token stream or ast in bin. (#278)\n\n* Added the ability to dump the token stream or ast in bin.\r\n\r\nThe dump functionality works both for files and REPL.\r\n\r\nWith --dump-tokens or -t for short it dumps the token stream to stdout  and --dump-ast or -a for short to dump the ast to stdout.\r\n\r\nThe dumping of tokens and ast is mutually exclusive. and when dumping it wont run the code.\r\n\r\n* Fixed some issues with rustfmt.\r\n\r\n* Added serde serialization and deserialization to token and the ast.\r\n\r\n* Added a dynamic multi-format dumping of token stream and ast in bin.\r\n\r\n- Changed the --dump-tokens and --dump-ast to be an optional argument that optionally takes a value of format type ([--opt=[val]]).\r\n- The default format for --dump-tokens and --dump-ast is Debug format which calls std::fmt::Debug.\r\n- Added Json and JsonMinified format for both dumps,  use serde_json internally.\r\n- It is easy to support other format types, such as Toml with toml-rs for example.\r\n\r\n* Made serde an optional dependency.\r\n\r\n- Serde serialization and deserialization can be switched on by using the feature flag \"serde-ast\".\r\n\r\n* Changed the JSON dumping format.\r\n\r\n- Now Json  dumping format prints the data in minefied JSON form by default.\r\n- Removed JsonMinified.\r\n- Added JsonPretty as a way to dump the data in pretty printed JSON format.\r\n\r\n* Updated the docs.",
          "timestamp": "2020-03-25T00:12:16Z",
          "tree_id": "8e55a5b8a2198ea513e42eaa96d1d4c690e446c4",
          "url": "https://github.com/jasonwilliams/boa/commit/5a85c595d4dff8fffd3d7881e4e9bca188691074"
        },
        "date": 1585098332844,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 479.27,
            "range": "+/- 4.010",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 592.94,
            "range": "+/- 6.110",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.4599,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 11.197,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 485.84,
            "range": "+/- 3.910",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1735,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3159,
            "range": "+/- 0.010",
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
          "id": "48c6e886d4fc63324d1695192d8960ac3efe4c21",
          "message": "Parser fixes #225 #240 #273 (#281)\n\nNew parser!\r\nPlus loads of tidy up in various places.\r\n\r\nCo-authored-by: Jason Williams <jwilliams720@bloomberg.net>\r\nCo-authored-by: HalidOdat <halidodat@gmail.com>\r\nCo-authored-by: Iban Eguia <iban.eguia@cern.ch>\r\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-03-31T19:29:21+01:00",
          "tree_id": "4f1e824ab37b8d367f9b0a6c8a308c615454ba98",
          "url": "https://github.com/jasonwilliams/boa/commit/48c6e886d4fc63324d1695192d8960ac3efe4c21"
        },
        "date": 1585679698194,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 439.55,
            "range": "+/- 6.450",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 548.05,
            "range": "+/- 6.780",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.8765,
            "range": "+/- 0.041",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.3732,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 454.1,
            "range": "+/- 8.570",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0798,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1096,
            "range": "+/- 0.022",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c365576f37456a61a157287ca716df23745314ab",
          "message": "Implemented the Array.prototype.some method. (#280)\n\n- Implementd Array.prototype.some method.\r\n- Added tests for Array.prototype.some method.",
          "timestamp": "2020-04-01T17:06:26+02:00",
          "tree_id": "4ed2d69cbff024df090e9f2477048fe59b83cf44",
          "url": "https://github.com/jasonwilliams/boa/commit/c365576f37456a61a157287ca716df23745314ab"
        },
        "date": 1585753961808,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 480.53,
            "range": "+/- 5.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 603.77,
            "range": "+/- 4.930",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.7132,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.7816,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 508.77,
            "range": "+/- 4.060",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1231,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2629,
            "range": "+/- 0.012",
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
          "id": "8943953a7998d81b858c761778a971d05204454e",
          "message": "fix vulnerabiliies via upgrade",
          "timestamp": "2020-04-02T19:50:22+01:00",
          "tree_id": "bf98de10bd0af856bf4bbb418bc3d2242ef21e6d",
          "url": "https://github.com/jasonwilliams/boa/commit/8943953a7998d81b858c761778a971d05204454e"
        },
        "date": 1585853800728,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 480.59,
            "range": "+/- 12.470",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 583.44,
            "range": "+/- 16.330",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.5718,
            "range": "+/- 0.107",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.5286,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 488.94,
            "range": "+/- 14.280",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1612,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2633,
            "range": "+/- 0.038",
            "unit": "us"
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
          "id": "4ed712219970a9aee437a02fa7992f6fea9e23f4",
          "message": "Fixed positions in regexes and strict operators. (#295)\n\nI also removed an unused function in the parser and added a test for #294, currently ignored.",
          "timestamp": "2020-04-04T17:52:51+01:00",
          "tree_id": "1fa31ebc9e7cafce4a9a331b7054410fadd567dd",
          "url": "https://github.com/jasonwilliams/boa/commit/4ed712219970a9aee437a02fa7992f6fea9e23f4"
        },
        "date": 1586019548659,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 491.02,
            "range": "+/- 8.910",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 614.57,
            "range": "+/- 14.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.6218,
            "range": "+/- 0.064",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.5819,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 522.78,
            "range": "+/- 14.880",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1622,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2955,
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
          "id": "8002a959a0537692e804d948c9933f04c0d8bf4e",
          "message": "Update CONTRIBUTING.md",
          "timestamp": "2020-04-08T18:38:03+01:00",
          "tree_id": "d2b3885a28bfc876c9d656a7cd1153eb059594a3",
          "url": "https://github.com/jasonwilliams/boa/commit/8002a959a0537692e804d948c9933f04c0d8bf4e"
        },
        "date": 1586367873339,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 470.38,
            "range": "+/- 14.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 592.06,
            "range": "+/- 16.490",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.723,
            "range": "+/- 0.107",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.4494,
            "range": "+/- 0.097",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 485.04,
            "range": "+/- 8.920",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1683,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2495,
            "range": "+/- 0.033",
            "unit": "us"
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
          "id": "795a70ec8910070a7c48def29a4b10f5144eda64",
          "message": "Use jemallocator (#298)\n\nAdded jemallocator as the global allocator for binary and benchmarks",
          "timestamp": "2020-04-10T13:14:49+02:00",
          "tree_id": "c5cd2377c8cd26b833f18b913e58e185e9fa614b",
          "url": "https://github.com/jasonwilliams/boa/commit/795a70ec8910070a7c48def29a4b10f5144eda64"
        },
        "date": 1586517679051,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 393.77,
            "range": "+/- 6.140",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 425.4,
            "range": "+/- 3.870",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.6612,
            "range": "+/- 0.041",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.9229,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 405.26,
            "range": "+/- 6.560",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0528,
            "range": "+/- 0.008",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2683,
            "range": "+/- 0.010",
            "unit": "us"
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
          "id": "38db4dc316febca9e85c66c6e13061736e13395b",
          "message": "Added a test for #208 (#303)",
          "timestamp": "2020-04-11T20:31:55+02:00",
          "tree_id": "041991c5ce20a5d048a26d7e917eb65db4b00409",
          "url": "https://github.com/jasonwilliams/boa/commit/38db4dc316febca9e85c66c6e13061736e13395b"
        },
        "date": 1586630321683,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 415.51,
            "range": "+/- 17.980",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 435.19,
            "range": "+/- 7.480",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.7824,
            "range": "+/- 0.076",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.9888,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 414.52,
            "range": "+/- 10.850",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1745,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3356,
            "range": "+/- 0.042",
            "unit": "us"
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
          "id": "48ab045ac295602e2db847dd5f5a91c51d07f120",
          "message": "Updated contribution documentation (#297)",
          "timestamp": "2020-04-11T20:33:28+02:00",
          "tree_id": "14055fcf4570e4ee4334335fbaa231ebcd9e468e",
          "url": "https://github.com/jasonwilliams/boa/commit/48ab045ac295602e2db847dd5f5a91c51d07f120"
        },
        "date": 1586630399511,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 383.54,
            "range": "+/- 7.020",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 432.67,
            "range": "+/- 11.200",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.3792,
            "range": "+/- 0.058",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.8724,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 402.85,
            "range": "+/- 7.120",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0755,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.2535,
            "range": "+/- 0.032",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f1f49d14ba7d1c014c953fea57f839998a6c8e96",
          "message": "Fixed center alignment of logo. (#305)",
          "timestamp": "2020-04-12T17:39:37+01:00",
          "tree_id": "3e1c01331e29d2527ce307db6497aeac7f379811",
          "url": "https://github.com/jasonwilliams/boa/commit/f1f49d14ba7d1c014c953fea57f839998a6c8e96"
        },
        "date": 1586709917129,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 305.48,
            "range": "+/- 4.930",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 337.6,
            "range": "+/- 6.200",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6389,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.4357,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 318.33,
            "range": "+/- 6.600",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 827.26,
            "range": "+/- 20.310",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0018,
            "range": "+/- -991.018",
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
          "id": "0274858d88c50a60b7c5669282a5d4c040dc220e",
          "message": "Revert \"Use jemallocator (#298)\" (#310)\n\nThis reverts commit 795a70ec8910070a7c48def29a4b10f5144eda64.",
          "timestamp": "2020-04-13T12:59:12+01:00",
          "tree_id": "86c609b19d882628e5e5cb460c13c6d9ca972861",
          "url": "https://github.com/jasonwilliams/boa/commit/0274858d88c50a60b7c5669282a5d4c040dc220e"
        },
        "date": 1586779528339,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 486.72,
            "range": "+/- 10.370",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 599.1,
            "range": "+/- 10.310",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.4262,
            "range": "+/- 0.066",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.4662,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 504.52,
            "range": "+/- 11.510",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1106,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.3044,
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
          "id": "a0db788ed6183e8b2719d820c88168aac4d92e76",
          "message": "adding test for #273 (#313)",
          "timestamp": "2020-04-13T15:36:04+02:00",
          "tree_id": "f6b945b7d34e9e7b759f06b1eee3a47c182cbbf2",
          "url": "https://github.com/jasonwilliams/boa/commit/a0db788ed6183e8b2719d820c88168aac4d92e76"
        },
        "date": 1586785298912,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 421.04,
            "range": "+/- 8.120",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 534.19,
            "range": "+/- 11.950",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.9211,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.8232,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 423.9,
            "range": "+/- 7.500",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 999.82,
            "range": "+/- -987.989",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.0516,
            "range": "+/- 0.028",
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
          "id": "1184456ab7c871b79942ece60383a9754d89f29f",
          "message": "update changelog for upcoming 0.7.0 (#271)\n\n* update changelog for v0.7.0",
          "timestamp": "2020-04-13T15:52:24+01:00",
          "tree_id": "d843119bda5a547b773d19bd19695d449e8e1e50",
          "url": "https://github.com/jasonwilliams/boa/commit/1184456ab7c871b79942ece60383a9754d89f29f"
        },
        "date": 1586789859689,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 397.64,
            "range": "+/- 15.470",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 497.63,
            "range": "+/- 18.380",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.7326,
            "range": "+/- 0.096",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 2.7728,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 406.63,
            "range": "+/- 11.140",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 889.8,
            "range": "+/- 31.160",
            "unit": "ns"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.019,
            "range": "+/- -994.366",
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
          "id": "ad0d1326509901f6aebd77d283fd7ade80ad3782",
          "message": "updating yanr lock",
          "timestamp": "2020-04-13T16:01:18+01:00",
          "tree_id": "c204beed99c690dd69872654c0ce45581eba7d3e",
          "url": "https://github.com/jasonwilliams/boa/commit/ad0d1326509901f6aebd77d283fd7ade80ad3782"
        },
        "date": 1586790429522,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 436.29,
            "range": "+/- 7.180",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 555.59,
            "range": "+/- 16.050",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 5.0294,
            "range": "+/- 0.052",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.2413,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 462.06,
            "range": "+/- 9.580",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0778,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1038,
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
          "id": "ea127b3d756782e6d84fce4b9d252e24a082d11a",
          "message": "updated to playground output",
          "timestamp": "2020-04-13T16:54:47+01:00",
          "tree_id": "d64a17b0b4839879f1de715f3a8cf32c9e49aa49",
          "url": "https://github.com/jasonwilliams/boa/commit/ea127b3d756782e6d84fce4b9d252e24a082d11a"
        },
        "date": 1586793645387,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 435,
            "range": "+/- 8.750",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 546.38,
            "range": "+/- 12.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.9991,
            "range": "+/- 0.063",
            "unit": "ms"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.1775,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "Hello World (Execution)",
            "value": 447.82,
            "range": "+/- 7.580",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.049,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.1448,
            "range": "+/- 0.020",
            "unit": "us"
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
          "id": "cb589cd8b7a75015a801db597964eaeed0508dea",
          "message": "Added more benchmarks (#323)",
          "timestamp": "2020-04-16T19:36:18+02:00",
          "tree_id": "827567adffbc1db74117525157104d1378c1f9ab",
          "url": "https://github.com/jasonwilliams/boa/commit/cb589cd8b7a75015a801db597964eaeed0508dea"
        },
        "date": 1587059015059,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 387.61,
            "range": "+/- 6.760",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 471.71,
            "range": "+/- 5.790",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 467.05,
            "range": "+/- 8.050",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.1589,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7959,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 879.8,
            "range": "+/- 15.120",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9715,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.3186,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9374,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.333,
            "range": "+/- 0.199",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7dd32a68594585950be3eaa62e480b8fec5ba45a",
          "message": "Added continuous integration for windows (#318)",
          "timestamp": "2020-04-16T19:43:33+02:00",
          "tree_id": "c62401ca2c81969a06563da43ea8ed6a74628b6c",
          "url": "https://github.com/jasonwilliams/boa/commit/7dd32a68594585950be3eaa62e480b8fec5ba45a"
        },
        "date": 1587059484012,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 383.42,
            "range": "+/- 15.480",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 498.27,
            "range": "+/- 25.430",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 469.24,
            "range": "+/- 14.710",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.4118,
            "range": "+/- 0.190",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8371,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 885.87,
            "range": "+/- 51.020",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.026,
            "range": "+/- 0.307",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.0192,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9243,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.792,
            "range": "+/- 0.575",
            "unit": "us"
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
          "id": "05b8989efdd3ecdb1cd498967e78052211248408",
          "message": "Changed the name of the `Symbol creation` benchmark (#327)",
          "timestamp": "2020-04-17T15:23:28+02:00",
          "tree_id": "f9f8f1142a28cf4e12f97af58e972ead25267c7e",
          "url": "https://github.com/jasonwilliams/boa/commit/05b8989efdd3ecdb1cd498967e78052211248408"
        },
        "date": 1587130261990,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 374.08,
            "range": "+/- 6.940",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 472.53,
            "range": "+/- 8.090",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 462.56,
            "range": "+/- 10.200",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.325,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9062,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 864.57,
            "range": "+/- 17.820",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0386,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.3621,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0354,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.434,
            "range": "+/- 0.257",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "contact@julianlaubstein.de",
            "name": "Julian Laubstein",
            "username": "sphinxc0re"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f5d332c86e3e063a48628ad416cb4c6525082058",
          "message": "change boa cli binary name to just \"boa\" (#326)",
          "timestamp": "2020-04-17T15:45:01+02:00",
          "tree_id": "986fa1f908ca7cf141841348036fa95fdca99e8e",
          "url": "https://github.com/jasonwilliams/boa/commit/f5d332c86e3e063a48628ad416cb4c6525082058"
        },
        "date": 1587131530888,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 397.22,
            "range": "+/- 2.980",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 502.16,
            "range": "+/- 13.860",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 487.77,
            "range": "+/- 7.770",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.3459,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.941,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 947.57,
            "range": "+/- 11.270",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4468,
            "range": "+/- 0.114",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5913,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2425,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.147,
            "range": "+/- 0.424",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "44de1901ea517da5ef88375ec4d0e7d191048908",
          "message": "Merge pull request #312 from jasonwilliams/fix_309",
          "timestamp": "2020-04-18T17:31:10+02:00",
          "tree_id": "4b04a46d8c60a964abfcf2136a85976fb2268d62",
          "url": "https://github.com/jasonwilliams/boa/commit/44de1901ea517da5ef88375ec4d0e7d191048908"
        },
        "date": 1587224353755,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 332.59,
            "range": "+/- 2.600",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 361.46,
            "range": "+/- 4.560",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 355.64,
            "range": "+/- 6.740",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.7161,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0681,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 883.14,
            "range": "+/- 9.560",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9544,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.7524,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9831,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.66,
            "range": "+/- 0.314",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "eb020d7f37f30983a0d2ea32a596e66c98d73e2f",
          "message": "Fix #329 (#334)",
          "timestamp": "2020-04-18T17:36:03+02:00",
          "tree_id": "69f27f151d3649678bb38f7ce353a55eb6f2698e",
          "url": "https://github.com/jasonwilliams/boa/commit/eb020d7f37f30983a0d2ea32a596e66c98d73e2f"
        },
        "date": 1587224630780,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 311.8,
            "range": "+/- 11.260",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 322.11,
            "range": "+/- 9.680",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 317.05,
            "range": "+/- 6.830",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.308,
            "range": "+/- 0.061",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7894,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 763.86,
            "range": "+/- 16.700",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.3357,
            "range": "+/- 0.116",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.1431,
            "range": "+/- 0.116",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.7409,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.048,
            "range": "+/- 0.301",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "t.victor@campus.lmu.de",
            "name": "Victor Tuekam",
            "username": "muskuloes"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a9372b7779fe3aa2e4b9f254e56faf83819b2cc0",
          "message": "Method parsing (#339)",
          "timestamp": "2020-04-21T21:02:52+02:00",
          "tree_id": "7455f3dcbf4b4c82dbf78954eb5ba3eb6dc88055",
          "url": "https://github.com/jasonwilliams/boa/commit/a9372b7779fe3aa2e4b9f254e56faf83819b2cc0"
        },
        "date": 1587496288441,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 349.96,
            "range": "+/- 4.730",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 378.98,
            "range": "+/- 3.670",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 368.7,
            "range": "+/- 3.990",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.9344,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2473,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 928.34,
            "range": "+/- 13.880",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2598,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.846,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0954,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.997,
            "range": "+/- 0.123",
            "unit": "us"
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
          "id": "91bece6f6297b4ac90de39681b0715b8d2f5d32d",
          "message": "Improved CI workflows (#330)",
          "timestamp": "2020-04-22T09:47:11+02:00",
          "tree_id": "047d206554631945502b7707edd13ba0bffe9f33",
          "url": "https://github.com/jasonwilliams/boa/commit/91bece6f6297b4ac90de39681b0715b8d2f5d32d"
        },
        "date": 1587541957746,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 340.17,
            "range": "+/- 15.870",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 354.78,
            "range": "+/- 7.940",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 344.21,
            "range": "+/- 8.240",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6346,
            "range": "+/- 0.097",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0085,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 861.1,
            "range": "+/- 17.020",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0194,
            "range": "+/- 0.224",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.7632,
            "range": "+/- 0.142",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0276,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.02,
            "range": "+/- 0.269",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "74c7375a7aa695c25d8368b7a30484971a429a91",
          "message": "Added logo and favicon to boa documentation (#343)\n\n* Added logo and favicon to boa documentation\n\n* Update lib.rs",
          "timestamp": "2020-04-24T23:52:12+02:00",
          "tree_id": "bbcc904de2771a06a00fc1e082da8b6699795ded",
          "url": "https://github.com/jasonwilliams/boa/commit/74c7375a7aa695c25d8368b7a30484971a429a91"
        },
        "date": 1587765638795,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 340.62,
            "range": "+/- 5.940",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 372.51,
            "range": "+/- 4.340",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 347.71,
            "range": "+/- 3.670",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6354,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0957,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 909.54,
            "range": "+/- 10.180",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1257,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9173,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1501,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.094,
            "range": "+/- 0.143",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "radek@krahl.pl",
            "name": "Radek Krahl",
            "username": "ptasz3k"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7abb94abf5fbfa2359e3a96144f68ac0f03123f4",
          "message": "console functions (#315)\n\n* Exec do..while tests.\n\n* Parser do..while test.\n\n* Do..while loop parser and exec implementation\n\n* rustfmt fixes\n\n* Update boa/src/exec/mod.rs\n\nCo-Authored-By: HalidOdat <halidodat@gmail.com>\n\n* Use expect(), make expect() skip newlines\n\n* rustmf fixes\n\n* Revert \"Use expect(), make expect() skip newlines\"\n\nThis reverts commit 517c4d0e065a3cd959e4aae65cc143c2a1019f1c.\n\n* Cargo Test Build task and Debug Test (Windows) launcher\n\n* First attempt at console.assert implementation. Tests are just for\ndebugging. Run `cargo test console_assert -- --nocapture` to see stderror\nmessages.\n\n* Refactoring - remove unnecessary map, variable rename.\n\n* Update boa/src/builtins/console.rs\r\n\r\nchanges from HalidOdat\n\nCo-Authored-By: HalidOdat <halidodat@gmail.com>\n\n* Documentation fixes\n\n* Remove space from documentation comment\n\n* Update boa/src/builtins/console.rs\r\n\r\nSimplify message printing.\n\nCo-Authored-By: Iban Eguia <razican@protonmail.ch>\n\n* Update boa/src/builtins/console.rs\r\n\r\nImprove docs.\n\nCo-Authored-By: Iban Eguia <razican@protonmail.ch>\n\n* Update boa/src/builtins/console.rs\r\n\r\nImprove getting of assertion result.\n\nCo-Authored-By: Iban Eguia <razican@protonmail.ch>\n\n* rustfmt\n\n* console.count() and console.countReset() implementation\n\n* Console state as native rust type, temporarily placed in Realm.\n\n* console.time[,Log,End] methods implementation\n\n* ConsoleState as internal state in console object.\n\n* Fix merge mess\n\n* Formatter function, get_arg_at_index moved out to function\n\n* Fix merge mess, pt. 2\n\n* console.group* functions\n\n* Moved console code to its own subdirectory, formatter tests, fixed utf-8\nhandling.\n\n* Most functions implemented.\n\n* Basic logger and logLevel implementation\n\n* console.group uses logger\n\n* console.dir (and at the same time dirxml) implementation\n\n* Make builtins::value::display_obj(...) public\n\n* Update boa/src/builtins/console/mod.rs\n\nCo-Authored-By: HalidOdat <halidodat@gmail.com>\n\n* Update boa/src/builtins/console/mod.rs\n\nCo-Authored-By: HalidOdat <halidodat@gmail.com>\n\n* Update boa/src/builtins/value/mod.rs\n\nCo-Authored-By: Iban Eguia <razican@protonmail.ch>\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-04-24T23:52:45+02:00",
          "tree_id": "d1a61b05c80159275baf8fb7ce1ab4aec89a2227",
          "url": "https://github.com/jasonwilliams/boa/commit/7abb94abf5fbfa2359e3a96144f68ac0f03123f4"
        },
        "date": 1587765678673,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 398.91,
            "range": "+/- 6.810",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 419.99,
            "range": "+/- 7.030",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 418.75,
            "range": "+/- 6.850",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.0649,
            "range": "+/- 0.062",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2745,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 993.1,
            "range": "+/- -983.458",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4571,
            "range": "+/- 0.188",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2158,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3019,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.207,
            "range": "+/- 0.227",
            "unit": "us"
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
          "id": "19fcd2837dd583e38476f949a10b0ab5fc1cf3b9",
          "message": "Fixing master build (#342)\n\n* Fixing master build\n\n* Using gh-pages as default branch",
          "timestamp": "2020-04-25T17:45:53+02:00",
          "tree_id": "d1651b62aa29d1e24899636838ef257e7fb43a66",
          "url": "https://github.com/jasonwilliams/boa/commit/19fcd2837dd583e38476f949a10b0ab5fc1cf3b9"
        },
        "date": 1587830167380,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 372.82,
            "range": "+/- 4.300",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 397.93,
            "range": "+/- 4.720",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 380.29,
            "range": "+/- 3.160",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6521,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0997,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 940.96,
            "range": "+/- 8.410",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2477,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9582,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1694,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.997,
            "range": "+/- 0.130",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "33094578+coolreader18@users.noreply.github.com",
            "name": "Noah",
            "username": "coolreader18"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "271e2b4da6563eebd8ee5469d643eb163adfa2df",
          "message": "Put JSON functions on the object, not the prototype (#325)\n\n* Move JSON functions to the object, not the prototype\n\n* Fix Value.to_json and add JSON tests\n\n* Update boa/src/builtins/json.rs\n\n* Update boa/src/builtins/json.rs\n\n* Update boa/src/builtins/json.rs\n\n* Update json.rs\n\n* Fix fmt issues.\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-04-26T08:44:32+02:00",
          "tree_id": "2763e01ad4374186428d0ad4cc6c725af62f9985",
          "url": "https://github.com/jasonwilliams/boa/commit/271e2b4da6563eebd8ee5469d643eb163adfa2df"
        },
        "date": 1587884175673,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 334.01,
            "range": "+/- 12.130",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 360.33,
            "range": "+/- 11.020",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 368.93,
            "range": "+/- 12.390",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6156,
            "range": "+/- 0.102",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0059,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 813.96,
            "range": "+/- 21.670",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5471,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2078,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8838,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.901,
            "range": "+/- 0.358",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "41cda68da19f05acd4c3e6a0b1517e5598925601",
          "message": "Merge pull request #293 from HalidOdat/better-documentation",
          "timestamp": "2020-04-27T21:06:00+02:00",
          "tree_id": "92ce0f3a75baf8fa9e98386a4086295ea8d5bff6",
          "url": "https://github.com/jasonwilliams/boa/commit/41cda68da19f05acd4c3e6a0b1517e5598925601"
        },
        "date": 1588014940713,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 335.5,
            "range": "+/- 9.700",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 348.69,
            "range": "+/- 8.760",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 343.59,
            "range": "+/- 7.440",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.2625,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8656,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 841.86,
            "range": "+/- 17.030",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.6624,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5443,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9704,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.715,
            "range": "+/- 0.298",
            "unit": "us"
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
          "id": "bc63b28b6b0f31d99d7cc75e9bd75f6d722d99d2",
          "message": "Modularized parser (#304)",
          "timestamp": "2020-04-28T20:17:08+02:00",
          "tree_id": "56a9f2a7f8aa6e36a14646599f34ce09ca8068a2",
          "url": "https://github.com/jasonwilliams/boa/commit/bc63b28b6b0f31d99d7cc75e9bd75f6d722d99d2"
        },
        "date": 1588098383967,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 339.07,
            "range": "+/- 12.740",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 363.93,
            "range": "+/- 10.320",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 357.9,
            "range": "+/- 14.930",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.4678,
            "range": "+/- 0.067",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9014,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 849.62,
            "range": "+/- 40.390",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5898,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.3757,
            "range": "+/- 0.116",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0683,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.813,
            "range": "+/- 0.302",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "t.victor@campus.lmu.de",
            "name": "Victor Tuekam",
            "username": "muskuloes"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "55c85768c38d45368d23637be76c9e2bcbdb4401",
          "message": "create boa-wasm package (#352)",
          "timestamp": "2020-04-28T20:16:18+02:00",
          "tree_id": "8081e42c3682b31aea03a5de86e90cbf6f0996a7",
          "url": "https://github.com/jasonwilliams/boa/commit/55c85768c38d45368d23637be76c9e2bcbdb4401"
        },
        "date": 1588098442732,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 348.83,
            "range": "+/- 14.520",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 373.3,
            "range": "+/- 11.860",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 368.45,
            "range": "+/- 14.650",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.5957,
            "range": "+/- 0.079",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0031,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 893.96,
            "range": "+/- 34.960",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.8948,
            "range": "+/- 0.188",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.3355,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0612,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.422,
            "range": "+/- 0.451",
            "unit": "us"
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
          "id": "13712d8791069086bcc6661c03d7d86e5119bd4e",
          "message": "Fixed doc link in README (#354)",
          "timestamp": "2020-04-28T23:02:08+02:00",
          "tree_id": "32e819dbbea51e557344cde1ee0249ca2f4041ac",
          "url": "https://github.com/jasonwilliams/boa/commit/13712d8791069086bcc6661c03d7d86e5119bd4e"
        },
        "date": 1588108269497,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 356.83,
            "range": "+/- 5.010",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 382.8,
            "range": "+/- 5.060",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 373.7,
            "range": "+/- 5.940",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.4258,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9992,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 878.14,
            "range": "+/- 14.810",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9334,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1185,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2857,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.879,
            "range": "+/- 0.263",
            "unit": "us"
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
          "id": "e0e17a8f7605b1a6d14f1d0e73c6a940273d42b8",
          "message": "Removed the `serde-ast` feature and the `serde_json` export (#353)",
          "timestamp": "2020-04-28T23:03:12+02:00",
          "tree_id": "c31b59f37320af6c9ef876e9b4cce9ce0e82111c",
          "url": "https://github.com/jasonwilliams/boa/commit/e0e17a8f7605b1a6d14f1d0e73c6a940273d42b8"
        },
        "date": 1588108391233,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 390.02,
            "range": "+/- 10.040",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 420.44,
            "range": "+/- 10.680",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 406.02,
            "range": "+/- 8.390",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.9727,
            "range": "+/- 0.070",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2927,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0168,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6246,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2631,
            "range": "+/- 0.171",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4424,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.449,
            "range": "+/- 0.432",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "84b4da545a3472646d7db62c3c56b37bf1c0dc04",
          "message": "Fix #331 \"We only get `Const::Num`, never `Const::Int`\" (#338)",
          "timestamp": "2020-04-29T01:37:41+02:00",
          "tree_id": "63378fbf6bf5ebe031290ec66144626b8a5c0515",
          "url": "https://github.com/jasonwilliams/boa/commit/84b4da545a3472646d7db62c3c56b37bf1c0dc04"
        },
        "date": 1588117560931,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 381.53,
            "range": "+/- 6.260",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 406.51,
            "range": "+/- 6.810",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 387.57,
            "range": "+/- 5.990",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.6557,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9591,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 931.46,
            "range": "+/- 12.890",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2714,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2373,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3197,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.254,
            "range": "+/- 0.151",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dd0f9678eeca93958f02e907821bab8f379c778a",
          "message": "fix #209 \"Calling Array with one argument\" (#328)\n\n* fix issue 209 \"Calling Array with one argument\"\n\n* Update boa/src/builtins/array/mod.rs\n\nCo-Authored-By: Iban Eguia <razican@protonmail.ch>\n\n* Changed from unimplemented to panic in array\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-04-29T23:53:13+02:00",
          "tree_id": "312b74d77f972c3aa8d6081ff4b73f69420f40cb",
          "url": "https://github.com/jasonwilliams/boa/commit/dd0f9678eeca93958f02e907821bab8f379c778a"
        },
        "date": 1588197721855,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 377.63,
            "range": "+/- 5.880",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 401.03,
            "range": "+/- 3.050",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 399.83,
            "range": "+/- 6.950",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.8183,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0011,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 949.11,
            "range": "+/- 7.760",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2422,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9786,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.346,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.665,
            "range": "+/- 0.163",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bgavin1996@gmail.com",
            "name": "Brian Gavin",
            "username": "brian-gavin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "55ef44ce1311d1ffced0ae14b606ae0a9a0bc453",
          "message": "feat(boa): in operator (#350)",
          "timestamp": "2020-05-02T23:12:16+02:00",
          "tree_id": "d65bb84ddd698b1bca3dbd5b86e72e980745ff52",
          "url": "https://github.com/jasonwilliams/boa/commit/55ef44ce1311d1ffced0ae14b606ae0a9a0bc453"
        },
        "date": 1588454425202,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 347.76,
            "range": "+/- 5.960",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 375.83,
            "range": "+/- 5.260",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 358.31,
            "range": "+/- 4.880",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.4408,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8338,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 864.11,
            "range": "+/- 13.210",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9328,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8204,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1331,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.408,
            "range": "+/- 0.194",
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
          "id": "f02babf0bd254b37920206b0ce0089bb9b01ba50",
          "message": "Refactor old function with new function object (#255)\n\nCo-authored-by: Iban Eguia <iban.eguia@cern.ch>\r\n\r\nCo-authored-by: Jason Williams <jwilliams720@bloomberg.net>\r\nCo-authored-by: Iban Eguia <iban.eguia@cern.ch>",
          "timestamp": "2020-05-03T08:35:12+01:00",
          "tree_id": "dd23fb856d254c26c07bef09777ea8c8c63ce8da",
          "url": "https://github.com/jasonwilliams/boa/commit/f02babf0bd254b37920206b0ce0089bb9b01ba50"
        },
        "date": 1588491817578,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 529.15,
            "range": "+/- 9.790",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 563.12,
            "range": "+/- 10.180",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.0205,
            "range": "+/- 0.044",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.027,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 954.07,
            "range": "+/- 10.880",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2448,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2556,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3674,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.466,
            "range": "+/- 0.158",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akryvomaz@users.noreply.github.com",
            "name": "Alexander Kryvomaz",
            "username": "akryvomaz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "75cf44a08aa62c3352cfa3d04313303c00868e48",
          "message": "Implement for loop (#374)\n\n* implement for loop execution\r\n\r\n* for loop benchmark\r\n\r\n* add more for loop tests\r\n\r\n* Update boa/src/exec/tests.rs\r\n\r\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-05-07T15:11:48+02:00",
          "tree_id": "9d10433c5917cdbdbe1ace45000aaead597beab1",
          "url": "https://github.com/jasonwilliams/boa/commit/75cf44a08aa62c3352cfa3d04313303c00868e48"
        },
        "date": 1588857629328,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 569.1,
            "range": "+/- 24.270",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 585.44,
            "range": "+/- 16.630",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 561.83,
            "range": "+/- 10.960",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 4.0323,
            "range": "+/- 0.070",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0298,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 992.19,
            "range": "+/- 12.770",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3514,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9851,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3301,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.964,
            "range": "+/- 0.257",
            "unit": "us"
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
          "id": "fb47031f442a8b412569c7000df60ea27fee210b",
          "message": "Changed HashMap and HashSet for Fx versions (#368)",
          "timestamp": "2020-05-07T20:06:41+02:00",
          "tree_id": "276c637363ef70428c727f532d791550e61296d3",
          "url": "https://github.com/jasonwilliams/boa/commit/fb47031f442a8b412569c7000df60ea27fee210b"
        },
        "date": 1588875364007,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 460.65,
            "range": "+/- 4.100",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 495.42,
            "range": "+/- 4.660",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 491.72,
            "range": "+/- 5.590",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.1262,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.885,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 918.77,
            "range": "+/- 6.600",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2647,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.072,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2761,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.843,
            "range": "+/- 0.088",
            "unit": "us"
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
          "id": "422d0b7ea1b6d6611911e3f7a72dc31f183e37ae",
          "message": "Optimised all `Vec<Node>` in `Node` to be `Box<[Node]>` (#370)",
          "timestamp": "2020-05-07T20:29:44+02:00",
          "tree_id": "adc7442c1d787561136e3cd43896514ead06de68",
          "url": "https://github.com/jasonwilliams/boa/commit/422d0b7ea1b6d6611911e3f7a72dc31f183e37ae"
        },
        "date": 1588876674206,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 450.74,
            "range": "+/- 5.840",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 483.46,
            "range": "+/- 5.090",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 476.48,
            "range": "+/- 4.910",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0442,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8618,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 894.01,
            "range": "+/- 11.850",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0398,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.868,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1894,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.559,
            "range": "+/- 0.119",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "35f5f0b5b33a42a82e8f737160f08b69303ee526",
          "message": "Code cleanup (#372)",
          "timestamp": "2020-05-07T21:18:16+02:00",
          "tree_id": "4722bbf5ad136c3a0b3d9ed36566ec0e7ea92727",
          "url": "https://github.com/jasonwilliams/boa/commit/35f5f0b5b33a42a82e8f737160f08b69303ee526"
        },
        "date": 1588879621692,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 513.65,
            "range": "+/- 8.270",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 553.11,
            "range": "+/- 10.610",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 524.41,
            "range": "+/- 11.130",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.528,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.119,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0467,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6821,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.238,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4667,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.516,
            "range": "+/- 0.236",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akryvomaz@users.noreply.github.com",
            "name": "Alexander Kryvomaz",
            "username": "akryvomaz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f6e0fdb1971ce32a6b1d74cf9c3e1d51b45092d1",
          "message": "Implement unary increment and decrement (#380)",
          "timestamp": "2020-05-07T22:05:52+02:00",
          "tree_id": "cb5e424c4e63b86f9336fce16aa60c769b53f4b7",
          "url": "https://github.com/jasonwilliams/boa/commit/f6e0fdb1971ce32a6b1d74cf9c3e1d51b45092d1"
        },
        "date": 1588882450682,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 462.55,
            "range": "+/- 3.290",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 503.98,
            "range": "+/- 5.380",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 489.63,
            "range": "+/- 4.320",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.2012,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9534,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 925.19,
            "range": "+/- 9.100",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2016,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0287,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2635,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.877,
            "range": "+/- 0.174",
            "unit": "us"
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
          "id": "9c9c4638e0d396b9379f158d7b04ee24d5cb2082",
          "message": "Implemented function expressions (#382)",
          "timestamp": "2020-05-09T14:41:18+02:00",
          "tree_id": "9dd8d501d878e1bcc31c413ced5003069e871b8e",
          "url": "https://github.com/jasonwilliams/boa/commit/9c9c4638e0d396b9379f158d7b04ee24d5cb2082"
        },
        "date": 1589028703444,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 460.08,
            "range": "+/- 5.280",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 489.53,
            "range": "+/- 8.200",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 479.78,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.1562,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8813,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 918.96,
            "range": "+/- 12.180",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2288,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9788,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2498,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.722,
            "range": "+/- 0.144",
            "unit": "us"
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
          "id": "fecd33d172554ad39d1ba8e4ac103d142b668489",
          "message": "Removed one indirection from objects (#386)",
          "timestamp": "2020-05-09T15:25:05+02:00",
          "tree_id": "4f55c2e0378289d325d40fbebbac9ec4b732ca6b",
          "url": "https://github.com/jasonwilliams/boa/commit/fecd33d172554ad39d1ba8e4ac103d142b668489"
        },
        "date": 1589031195807,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 434.21,
            "range": "+/- 3.910",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 468.73,
            "range": "+/- 7.620",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 458.87,
            "range": "+/- 6.330",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0558,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9385,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 921.82,
            "range": "+/- 3.760",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2788,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0991,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2936,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.843,
            "range": "+/- 0.166",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1e18cb02d0682627d6d0171464eb0d868de7fcce",
          "message": "Value refactor (#383)",
          "timestamp": "2020-05-09T16:06:30+02:00",
          "tree_id": "913f4d831b5800edb3c76829a2a2ec813b27361f",
          "url": "https://github.com/jasonwilliams/boa/commit/1e18cb02d0682627d6d0171464eb0d868de7fcce"
        },
        "date": 1589033683451,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 398.66,
            "range": "+/- 3.650",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 423.9,
            "range": "+/- 4.740",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 418.61,
            "range": "+/- 5.820",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.7639,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9183,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 918.01,
            "range": "+/- 11.060",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1631,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0627,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2605,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.796,
            "range": "+/- 0.068",
            "unit": "us"
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
          "id": "59df3acc6b6602874890a19c2f35bc39d797aec0",
          "message": "Added issue/PR templates (#385)",
          "timestamp": "2020-05-09T17:27:31+02:00",
          "tree_id": "eefc959f930e5d660b09404b10b345c9c2d42315",
          "url": "https://github.com/jasonwilliams/boa/commit/59df3acc6b6602874890a19c2f35bc39d797aec0"
        },
        "date": 1589038548401,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 410.64,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 440.17,
            "range": "+/- 3.510",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 432.74,
            "range": "+/- 2.050",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9115,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9737,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 952.17,
            "range": "+/- 4.930",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3305,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2302,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3805,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.373,
            "range": "+/- 0.073",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dvt.tnhn.krlbs@gmail.com",
            "name": "Tunahan Karlıbaş",
            "username": "dvtkrlbs"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bdad99cb8297f72369d843c319bd30d8d6410466",
          "message": "Implement toString (#381)\n\n* Implement optional parameter `radix` for `Number.prototype.toString( [radix] )\n\nImplement the radix paramet for the `toString`. This implementation is\nconverted from the V8's c++ implementation.\n\n* Use a reversed iterator instead of cursors in the integer part.\n\nInitial version for getting rid of direct slice accesses. Currently\nconverted integer part to iterators. Fraction part is a lot harder since\nthere are two passes to the fraction part (for carry over) and it is\nhard to express that using iterators.\n\n* Format tests",
          "timestamp": "2020-05-10T09:48:09+02:00",
          "tree_id": "e4bd0c8d278368ef41e2a11b3fed1c1f2ba35db1",
          "url": "https://github.com/jasonwilliams/boa/commit/bdad99cb8297f72369d843c319bd30d8d6410466"
        },
        "date": 1589097412914,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 350.81,
            "range": "+/- 10.710",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 398,
            "range": "+/- 9.540",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 373.19,
            "range": "+/- 10.930",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4569,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7433,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 790.65,
            "range": "+/- 30.380",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.6201,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5744,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9309,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.126,
            "range": "+/- 0.446",
            "unit": "us"
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
          "id": "143434f643bad49b6ec49a0bac7bda05e9d06044",
          "message": "Added `BindingIdentifier` parsing. (#389)",
          "timestamp": "2020-05-10T11:53:20+02:00",
          "tree_id": "b525d0f04a92f5bc24c8bc76793a6027f960e081",
          "url": "https://github.com/jasonwilliams/boa/commit/143434f643bad49b6ec49a0bac7bda05e9d06044"
        },
        "date": 1589104919853,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 391.43,
            "range": "+/- 7.200",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 417.27,
            "range": "+/- 5.590",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 409.99,
            "range": "+/- 5.680",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.751,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8809,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 870.58,
            "range": "+/- 8.320",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9419,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.7546,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1697,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.635,
            "range": "+/- 0.264",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akryvomaz@users.noreply.github.com",
            "name": "Alexander Kryvomaz",
            "username": "akryvomaz"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "9cd9a39aa666b65de5518b3fb261819fc5ef86e8",
          "message": "Implement unary void, delete operators (#388)",
          "timestamp": "2020-05-10T17:54:21+02:00",
          "tree_id": "97bbe3f1c9a4db70c3766bb11448138cc9114909",
          "url": "https://github.com/jasonwilliams/boa/commit/9cd9a39aa666b65de5518b3fb261819fc5ef86e8"
        },
        "date": 1589126540181,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 377.44,
            "range": "+/- 11.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 404.76,
            "range": "+/- 9.990",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 398.81,
            "range": "+/- 10.370",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.5871,
            "range": "+/- 0.064",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7594,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 796.41,
            "range": "+/- 20.740",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5331,
            "range": "+/- 0.104",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.3777,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1145,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.955,
            "range": "+/- 0.273",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "n14little@gmail.com",
            "name": "n14little",
            "username": "n14little"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4e5b8ee2b51dac0972cb15eb9762c6d4e0bf6321",
          "message": "run security audit daily at midnight. (#391)\n\n* run security audit daily at midnight.\n\n* add new line to end of security_audit.yml",
          "timestamp": "2020-05-11T19:19:03+02:00",
          "tree_id": "16b3f167891034e405f448cd772ba24342faca83",
          "url": "https://github.com/jasonwilliams/boa/commit/4e5b8ee2b51dac0972cb15eb9762c6d4e0bf6321"
        },
        "date": 1589218059421,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 398.47,
            "range": "+/- 2.360",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 427.85,
            "range": "+/- 6.440",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 425.65,
            "range": "+/- 3.000",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.8031,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.923,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 896.31,
            "range": "+/- 10.270",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1115,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.975,
            "range": "+/- 0.097",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.261,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.573,
            "range": "+/- 0.630",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "abhijeet.bhagat@gmx.com",
            "name": "abhi",
            "username": "abhijeetbhagat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2851eb02a7ce7b7d061aafb4d56abab65a8e6206",
          "message": "Modularize try statement parsing (#390)\n\n* Fix catch parsing - move the cursor to next token\n\n* Refactor catch and finally parsing into separate modules\n\n* Refactor catchparam parsing into separate module and add more tests\n\n* Refactoring - use ? instead of match",
          "timestamp": "2020-05-11T19:21:26+02:00",
          "tree_id": "bdac8653f7bb23465dacd90c5984761f30557df1",
          "url": "https://github.com/jasonwilliams/boa/commit/2851eb02a7ce7b7d061aafb4d56abab65a8e6206"
        },
        "date": 1589218197371,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 402.41,
            "range": "+/- 7.880",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 426.34,
            "range": "+/- 7.790",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 453.32,
            "range": "+/- 15.330",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9239,
            "range": "+/- 0.080",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0789,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 940.24,
            "range": "+/- 17.400",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0836,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0253,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3948,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.156,
            "range": "+/- 0.237",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia Moraza",
            "username": "Razican"
          },
          "committer": {
            "email": "razican@protonmail.ch",
            "name": "Iban Eguia",
            "username": "Razican"
          },
          "distinct": true,
          "id": "44fde226fc907a15aad4d3802c77777f4f9d4a63",
          "message": "Added code of conduct based on Contributor Covenant 2.0\n\nCo-authored-by: Jason Williams <jase.williams@gmail.com>\nCo-authored-by: HalidOdat <halidodat@gmail.com>\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-05-13T17:50:54+02:00",
          "tree_id": "7542538a33d61017e17af80cb6f8ef36f204dad2",
          "url": "https://github.com/jasonwilliams/boa/commit/44fde226fc907a15aad4d3802c77777f4f9d4a63"
        },
        "date": 1589385553015,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 406.77,
            "range": "+/- 6.830",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 443.17,
            "range": "+/- 6.540",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 441.67,
            "range": "+/- 6.580",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0057,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.979,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 945.34,
            "range": "+/- 16.380",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1042,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0521,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3833,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.306,
            "range": "+/- 0.162",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "s1panda@ucsd.edu",
            "name": "Subhankar Panda",
            "username": "subhankar-panda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "fefb5a3b71344763d92382a092bb10eefe7b0ee5",
          "message": "Remove Monaco Editor Webpack Plugin and Manually Vendor Editor Workers (#362)\n\n* Removed *.bundle.js from editor WebWorkers\r\n\r\n* Removed Monaco Editor Plugin\r\n\r\n* package.lock -> yarn.lock",
          "timestamp": "2020-05-13T17:36:43+01:00",
          "tree_id": "f38223bbbc3f1397faf66abab13f1a8406032ec2",
          "url": "https://github.com/jasonwilliams/boa/commit/fefb5a3b71344763d92382a092bb10eefe7b0ee5"
        },
        "date": 1589388285511,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 388.77,
            "range": "+/- 7.170",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 392.19,
            "range": "+/- 9.990",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 408.75,
            "range": "+/- 11.960",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6921,
            "range": "+/- 0.090",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.6714,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 840.53,
            "range": "+/- 27.380",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5749,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4861,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1213,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.085,
            "range": "+/- 0.456",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "linyiyu1992@gmail.com",
            "name": "Yiyu Lin",
            "username": "attliaLin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d4d27296fc6af92b2a637f3851ddfa0933796a63",
          "message": "fix `NaN` is lexed as identifier, not as a number (#397)\n\nclose #393",
          "timestamp": "2020-05-13T19:17:54+01:00",
          "tree_id": "ce7d4e73506fa7b07428a46a4cad4be283f5fd92",
          "url": "https://github.com/jasonwilliams/boa/commit/d4d27296fc6af92b2a637f3851ddfa0933796a63"
        },
        "date": 1589394374290,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 397.09,
            "range": "+/- 4.820",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 422.28,
            "range": "+/- 4.490",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 416.99,
            "range": "+/- 5.300",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.7827,
            "range": "+/- 0.030",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9016,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 960.65,
            "range": "+/- 8.950",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.117,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9345,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2515,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.858,
            "range": "+/- 0.180",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "dj_amazing@sina.com",
            "name": "hello2dj",
            "username": "hello2dj"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "402041d43251017ab772fe6f9920e41f496a359d",
          "message": "impl abstract-equality-comparison (#395)",
          "timestamp": "2020-05-13T21:01:42+02:00",
          "tree_id": "17d67990cb809ab7e493ca9dc859ba4908402d08",
          "url": "https://github.com/jasonwilliams/boa/commit/402041d43251017ab772fe6f9920e41f496a359d"
        },
        "date": 1589397023845,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 400.81,
            "range": "+/- 7.280",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 426.09,
            "range": "+/- 2.810",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 428.08,
            "range": "+/- 6.710",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.7894,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9507,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 966.3,
            "range": "+/- 8.730",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.213,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9622,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2984,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.021,
            "range": "+/- 0.144",
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
          "id": "63f37a2858f0e618ac277a91cb9148e849b5bf45",
          "message": "implement \"this\" (#320)\n\n* implement this\r\n* remove construct/call from Object instead set func\r\n* get_this_binding() was wrong, fixed\r\n* BindingStatus is now properly set\r\n* `this` now works on dynamic functions\r\n* Migrates all builtins to use a single constructor/call fucntion to match the spec\r\n* Ensure new object has an existing prototype\r\n* create_function utility\r\n* needing to clone before passing through",
          "timestamp": "2020-05-16T14:20:50+01:00",
          "tree_id": "2d09323b7a7c4cc878a75aa09efad966d5ed4685",
          "url": "https://github.com/jasonwilliams/boa/commit/63f37a2858f0e618ac277a91cb9148e849b5bf45"
        },
        "date": 1589635733498,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 370.84,
            "range": "+/- 4.870",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 398.32,
            "range": "+/- 9.470",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 398.24,
            "range": "+/- 10.900",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.5218,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8658,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 912.71,
            "range": "+/- 19.200",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9696,
            "range": "+/- 0.088",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6793,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2014,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.514,
            "range": "+/- 0.361",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tyler@givestack.com",
            "name": "Tyler Morten",
            "username": "tylermorten"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d84d9cbe2e1d60c85ba7c6f01b2c1eaf5b0e6e7c",
          "message": "Fix for 0 length new String (#404)\n\n* Fix for 0 length field when constructing a new String.\r\n\r\n* String.length uses character count not byte count. Also, corresponding test\r\n\r\n* Made tests more succinct per suggestion.",
          "timestamp": "2020-05-19T09:44:20+02:00",
          "tree_id": "7af8e7ad1413afe19476d0550ae390c7989d95d7",
          "url": "https://github.com/jasonwilliams/boa/commit/d84d9cbe2e1d60c85ba7c6f01b2c1eaf5b0e6e7c"
        },
        "date": 1589874734378,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 368.93,
            "range": "+/- 6.330",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 395.97,
            "range": "+/- 4.920",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 388.77,
            "range": "+/- 8.490",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.5413,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7987,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 910.39,
            "range": "+/- 11.890",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9606,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.7025,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1611,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.307,
            "range": "+/- 0.172",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5f4a1f22665c720c2cb5905a4133b92665f6b00e",
          "message": "Feature `BigInt` (#358)",
          "timestamp": "2020-05-19T13:06:35+02:00",
          "tree_id": "cd4ee3e6a0ca8388bfae243ddae7812362b91bcf",
          "url": "https://github.com/jasonwilliams/boa/commit/5f4a1f22665c720c2cb5905a4133b92665f6b00e"
        },
        "date": 1589886919330,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 355.61,
            "range": "+/- 6.450",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 382.7,
            "range": "+/- 8.170",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 386.45,
            "range": "+/- 8.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4103,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8227,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 885.27,
            "range": "+/- 18.650",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0433,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.982,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1861,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.712,
            "range": "+/- 0.203",
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
          "id": "05f220d38d2db4bc8330eaeaf221f1a686edc0f8",
          "message": "String wasn't defaulting to empty when called as String() with no argument (#407)",
          "timestamp": "2020-05-20T23:04:45+01:00",
          "tree_id": "e045495afcaeadbcf7903a1f4ace695f7af55e78",
          "url": "https://github.com/jasonwilliams/boa/commit/05f220d38d2db4bc8330eaeaf221f1a686edc0f8"
        },
        "date": 1590012784299,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 412.75,
            "range": "+/- 4.640",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 444.45,
            "range": "+/- 1.550",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 442.73,
            "range": "+/- 4.000",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.8245,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0435,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 975.01,
            "range": "+/- 6.230",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3697,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3256,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4642,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.895,
            "range": "+/- 0.169",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "1396351+RestitutorOrbis@users.noreply.github.com",
            "name": "Javed Nissar",
            "username": "RestitutorOrbis"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "29abfd6147b744b2ddb53b536d7fc67fcfb98107",
          "message": "Resolved #359 (#396)\n\nRemoved Node::TypeOf, implemented UnaryOp::TypeOf, and added tests",
          "timestamp": "2020-05-21T10:45:11+02:00",
          "tree_id": "615914f3777e31b234e9cfd18765d95a2b686951",
          "url": "https://github.com/jasonwilliams/boa/commit/29abfd6147b744b2ddb53b536d7fc67fcfb98107"
        },
        "date": 1590051171891,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 371.05,
            "range": "+/- 5.790",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 400.55,
            "range": "+/- 7.400",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 402.93,
            "range": "+/- 6.890",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.5296,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8645,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 878.23,
            "range": "+/- 15.810",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9096,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8755,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3399,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.794,
            "range": "+/- 0.208",
            "unit": "us"
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
          "id": "323f486fd1b7d55ae5fcda1f666073b46a21b851",
          "message": "Dependency upgrade (#406)",
          "timestamp": "2020-05-21T17:31:32+02:00",
          "tree_id": "3b1363698e0b37050205ba6ea33e9577a41c530e",
          "url": "https://github.com/jasonwilliams/boa/commit/323f486fd1b7d55ae5fcda1f666073b46a21b851"
        },
        "date": 1590075601473,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 327.33,
            "range": "+/- 8.040",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 361.95,
            "range": "+/- 8.310",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 344.41,
            "range": "+/- 6.730",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.311,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.5821,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 807.15,
            "range": "+/- 14.710",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.1897,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.166,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9792,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.979,
            "range": "+/- 0.341",
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
          "id": "b2a60b8945f65d8f83e98cb3df31da0d6fdc2709",
          "message": "v0.8 (#399)\n\n* v.0.8\r\n\r\n* add this to features\r\n\r\n* changelog updates",
          "timestamp": "2020-05-22T18:45:37+01:00",
          "tree_id": "6d828067c39101fb0a975640d72d752e326991a4",
          "url": "https://github.com/jasonwilliams/boa/commit/b2a60b8945f65d8f83e98cb3df31da0d6fdc2709"
        },
        "date": 1590170063688,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 418.64,
            "range": "+/- 7.690",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 454.31,
            "range": "+/- 9.670",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 453.74,
            "range": "+/- 7.170",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9352,
            "range": "+/- 0.056",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0939,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0052,
            "range": "+/- -996.417",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5362,
            "range": "+/- 0.142",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2571,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4901,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.08,
            "range": "+/- 0.290",
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
          "id": "5cfe52964928c75ff7ab7a3d5157587031ded416",
          "message": "title",
          "timestamp": "2020-05-22T18:47:05+01:00",
          "tree_id": "05fdb851138f758b9c2755784c83bc6993be4cc4",
          "url": "https://github.com/jasonwilliams/boa/commit/5cfe52964928c75ff7ab7a3d5157587031ded416"
        },
        "date": 1590170159205,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 434.38,
            "range": "+/- 8.560",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 455.13,
            "range": "+/- 12.100",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 444.1,
            "range": "+/- 8.430",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0565,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0822,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0246,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4717,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1886,
            "range": "+/- 0.139",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4646,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.146,
            "range": "+/- 0.323",
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
          "id": "939fa390f42da13aab923342ee4e94409e1d1cb4",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-05-24T12:17:26+01:00",
          "tree_id": "1f67eb332e5f4cf29d32f111f8f4445a0a382dc0",
          "url": "https://github.com/jasonwilliams/boa/commit/939fa390f42da13aab923342ee4e94409e1d1cb4"
        },
        "date": 1590319521893,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 356.36,
            "range": "+/- 5.820",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 375.61,
            "range": "+/- 6.350",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 372.24,
            "range": "+/- 9.130",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3645,
            "range": "+/- 0.048",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7021,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 847.54,
            "range": "+/- 15.070",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.6178,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.561,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1271,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.889,
            "range": "+/- 0.291",
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
          "id": "db18360ed5d1649841e04556bd3645fdc884e01b",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-05-24T12:21:24+01:00",
          "tree_id": "1d1f0354bf4948616004b93aac48cbe71300d5c6",
          "url": "https://github.com/jasonwilliams/boa/commit/db18360ed5d1649841e04556bd3645fdc884e01b"
        },
        "date": 1590319733208,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 339.97,
            "range": "+/- 11.200",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 365.74,
            "range": "+/- 8.050",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 362.97,
            "range": "+/- 9.770",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2535,
            "range": "+/- 0.052",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.6474,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 813.13,
            "range": "+/- 21.370",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.6218,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4233,
            "range": "+/- 0.108",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.034,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.351,
            "range": "+/- 0.432",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d837e040c9f12ceac4284f4fb557808f006f742d",
          "message": "Number constants (#420)\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-05-26T16:20:40+02:00",
          "tree_id": "78bd300b1c7b34c53664473802c72d2724578258",
          "url": "https://github.com/boa-dev/boa/commit/d837e040c9f12ceac4284f4fb557808f006f742d"
        },
        "date": 1590503365640,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 401.17,
            "range": "+/- 9.770",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 409.8,
            "range": "+/- 8.210",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 414.9,
            "range": "+/- 7.140",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3784,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8908,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 892.8,
            "range": "+/- 16.380",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.7964,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4524,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0379,
            "range": "+/- 0.041",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.265,
            "range": "+/- 0.276",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "32345702+pedropaulosuzuki@users.noreply.github.com",
            "name": "Pedro Paulo",
            "username": "pedropaulosuzuki"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c8218dd91ef3181e048e7a2659a4fbf8d53c7174",
          "message": "Replacement of dead links (#423)\n\n* Update repository link on Cargo.toml\n\n* Update repository for Cargo.toml [WASM]\n\n* Update link on CONTRIBUTING.md\n\nMoving <https://github.com/jasonwilliams/boa/issues> to <https://github.com/boa-dev/boa/issues>\n\n* Remove dead links on CHANGELOG.md\n\nChanges all dead links from <https://github.com/jasonwilliams/boa> to <https://github.com/boa-dev/boa>.\n\n* Fix dead link on boa/Cargo.toml\n\nFrom <https://github.com/jasonwilliams/boa> to <https://github.com/boa-dev/boa>\n\n* CHANGELOG.md - Fix link on comparing v0.7.0 with v0.8.0\n\nPreviously comparing v0.7.0...HEAD",
          "timestamp": "2020-05-29T13:19:14+02:00",
          "tree_id": "4ce63a0e563697be85e26c402307948815e056e5",
          "url": "https://github.com/boa-dev/boa/commit/c8218dd91ef3181e048e7a2659a4fbf8d53c7174"
        },
        "date": 1590751705299,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 456.18,
            "range": "+/- 11.470",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 502.3,
            "range": "+/- 15.240",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 514.74,
            "range": "+/- 21.230",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9229,
            "range": "+/- 0.081",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2748,
            "range": "+/- 0.088",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.068,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.8619,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1347,
            "range": "+/- 0.178",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4002,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.722,
            "range": "+/- 0.697",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "n14little@gmail.com",
            "name": "n14little",
            "username": "n14little"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "82908dfdf457a1e21922bf6684bc77bbe8132423",
          "message": "Json stringify replacer (#402)",
          "timestamp": "2020-05-30T12:00:31+02:00",
          "tree_id": "125ec12918bf45f2aebaa29c9cab3b64070339c4",
          "url": "https://github.com/boa-dev/boa/commit/82908dfdf457a1e21922bf6684bc77bbe8132423"
        },
        "date": 1590833291141,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 389.24,
            "range": "+/- 10.610",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 405.28,
            "range": "+/- 9.690",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 400.68,
            "range": "+/- 8.920",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3124,
            "range": "+/- 0.068",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0715,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 859.09,
            "range": "+/- 35.440",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0536,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6318,
            "range": "+/- 0.114",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0466,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.198,
            "range": "+/- 0.312",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "32345702+pedropaulosuzuki@users.noreply.github.com",
            "name": "Pedro Paulo",
            "username": "pedropaulosuzuki"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "87aea64c1f6965b4994d780da5f3e4e7fa4c02d0",
          "message": "Consistency at README.md (#424)\n\nTries to make the README.md file more consistent. In some places the lines ended with a \".\", while in others it did not. The licenses at the ended were missing some links and some sessions, such as [Benchmark] did not have any text besides the main link, among other things.",
          "timestamp": "2020-05-30T14:32:02+02:00",
          "tree_id": "87b81996bf0fe23408b0d769341db092d01b6e01",
          "url": "https://github.com/boa-dev/boa/commit/87aea64c1f6965b4994d780da5f3e4e7fa4c02d0"
        },
        "date": 1590842399120,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 423.42,
            "range": "+/- 7.790",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 448.93,
            "range": "+/- 9.080",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 440.33,
            "range": "+/- 7.120",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.5463,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.054,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 964.27,
            "range": "+/- 16.990",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4633,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9743,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2021,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.634,
            "range": "+/- 0.375",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "abhijeet.bhagat@gmx.com",
            "name": "abhi",
            "username": "abhijeetbhagat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8255c3a83a12db6f4affdf44167bde2745cecf39",
          "message": "Add support for the reviver function to JSON.parse (#410)",
          "timestamp": "2020-06-01T10:40:41+02:00",
          "tree_id": "29aa368334ad2e60c84c34f02707d9734ac278c4",
          "url": "https://github.com/boa-dev/boa/commit/8255c3a83a12db6f4affdf44167bde2745cecf39"
        },
        "date": 1591001309954,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 415.52,
            "range": "+/- 7.080",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 439.02,
            "range": "+/- 8.240",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 430.45,
            "range": "+/- 6.700",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4571,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0139,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 956.39,
            "range": "+/- 11.640",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1945,
            "range": "+/- 0.104",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8512,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1442,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.019,
            "range": "+/- 0.289",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ce0d801685953384ac762e62540019a2f530a299",
          "message": "Add code coverage: tarpaulin / codecov (#411)\n\n* Adding tarpaulin code coverage step\r\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-06-01T13:49:36+01:00",
          "tree_id": "8a54605a17f06e3f64a826bafbd423e3f33d52fd",
          "url": "https://github.com/boa-dev/boa/commit/ce0d801685953384ac762e62540019a2f530a299"
        },
        "date": 1591016304287,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 441.4,
            "range": "+/- 5.970",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 467.53,
            "range": "+/- 3.230",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 478.37,
            "range": "+/- 5.860",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.8247,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1599,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0325,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5447,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9392,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.328,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.154,
            "range": "+/- 0.167",
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
          "id": "640b76efaa4deb04aa0140fd15301d3d049c9f61",
          "message": "Update README.md",
          "timestamp": "2020-06-01T14:25:00+01:00",
          "tree_id": "9715ea4b9f3e13e0f5b75846faf0e70a1517824e",
          "url": "https://github.com/boa-dev/boa/commit/640b76efaa4deb04aa0140fd15301d3d049c9f61"
        },
        "date": 1591018399390,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 410.61,
            "range": "+/- 6.550",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 453.27,
            "range": "+/- 11.070",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 448.82,
            "range": "+/- 8.830",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6129,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.003,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 973.93,
            "range": "+/- 20.570",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1416,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4667,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1712,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.274,
            "range": "+/- 0.234",
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
          "id": "aca55fd9aa9a342b8e7a5b314b48db21967d569d",
          "message": "Update README.md\n\nremove testing",
          "timestamp": "2020-06-01T14:51:55+01:00",
          "tree_id": "8a54605a17f06e3f64a826bafbd423e3f33d52fd",
          "url": "https://github.com/boa-dev/boa/commit/aca55fd9aa9a342b8e7a5b314b48db21967d569d"
        },
        "date": 1591020042982,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 441.18,
            "range": "+/- 5.600",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 470.87,
            "range": "+/- 5.150",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 456.43,
            "range": "+/- 5.130",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6611,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1236,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0028,
            "range": "+/- -997.572",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5376,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1547,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3118,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.848,
            "range": "+/- 0.105",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b308f823024cf826171d3762a29b4f1f25fbc629",
          "message": "Update to use a newer version of codecov github action (#436)",
          "timestamp": "2020-06-01T15:08:02+01:00",
          "tree_id": "7630fd6db2e0e3540312508b542d65b6c980862e",
          "url": "https://github.com/boa-dev/boa/commit/b308f823024cf826171d3762a29b4f1f25fbc629"
        },
        "date": 1591021089493,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 436.74,
            "range": "+/- 3.640",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 466.66,
            "range": "+/- 5.170",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 454.88,
            "range": "+/- 3.040",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6149,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0866,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 989.56,
            "range": "+/- 3.700",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.519,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0789,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2765,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.677,
            "range": "+/- 0.069",
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
          "id": "073b8cee0608d5f09eccf9f7d4b32cea94463c54",
          "message": "Update rust.yml\n\njust use @v1",
          "timestamp": "2020-06-01T15:10:21+01:00",
          "tree_id": "0f3da3a849ef8dac5915f9a22bc30c551702114e",
          "url": "https://github.com/boa-dev/boa/commit/073b8cee0608d5f09eccf9f7d4b32cea94463c54"
        },
        "date": 1591021150232,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 440.54,
            "range": "+/- 3.860",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 466.26,
            "range": "+/- 5.100",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 451.81,
            "range": "+/- 3.940",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6477,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1338,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0035,
            "range": "+/- -997.701",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5963,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1085,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2818,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.798,
            "range": "+/- 0.164",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5e71718928a77e87015b85bf58dbac6442db644c",
          "message": "Specification compliant `ToString` (`to_string`) (#425)",
          "timestamp": "2020-06-01T17:30:06+02:00",
          "tree_id": "777a7ef2d67b77c0528cb89dadc8a80f87d65d86",
          "url": "https://github.com/boa-dev/boa/commit/5e71718928a77e87015b85bf58dbac6442db644c"
        },
        "date": 1591025914883,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 440.75,
            "range": "+/- 3.780",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 472.14,
            "range": "+/- 4.690",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 477.82,
            "range": "+/- 5.620",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6939,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.124,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0061,
            "range": "+/- 0.013",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5553,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1448,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3242,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.997,
            "range": "+/- 0.193",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d003ad3b6d98a463e6237551b29868b0b759e926",
          "message": "Fixed badges in README (#437)",
          "timestamp": "2020-06-01T17:37:09+02:00",
          "tree_id": "e55247beab34ee8574a423acc379af71bc00d395",
          "url": "https://github.com/boa-dev/boa/commit/d003ad3b6d98a463e6237551b29868b0b759e926"
        },
        "date": 1591026318342,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 429.81,
            "range": "+/- 8.730",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 466.69,
            "range": "+/- 9.330",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 467.21,
            "range": "+/- 8.920",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.6067,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1131,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0224,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6601,
            "range": "+/- 0.088",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1469,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2802,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.142,
            "range": "+/- 0.760",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "145f0e3f03ee719e3c10002389a74e1f040eb1ae",
          "message": "Our benchmarks action now lives in boa-dev (#438)",
          "timestamp": "2020-06-01T18:14:50+02:00",
          "tree_id": "e285dcb87c96bf61f7bf9c0e5f2198787653c227",
          "url": "https://github.com/boa-dev/boa/commit/145f0e3f03ee719e3c10002389a74e1f040eb1ae"
        },
        "date": 1591028610447,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 469.22,
            "range": "+/- 10.570",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 485.23,
            "range": "+/- 6.010",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 474.72,
            "range": "+/- 5.950",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 3.0235,
            "range": "+/- 0.057",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2399,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0462,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7917,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3861,
            "range": "+/- 0.116",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4337,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.249,
            "range": "+/- 0.266",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "n14little@gmail.com",
            "name": "n14little",
            "username": "n14little"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4f2191ae1f82b95dd5ffe6a5cfc6d2dcaaada6c5",
          "message": "Has own property should call get own property (#444)\n\n* object.hasOwnProperty should call getOwnProperty\n\n* should work for properties with undefined and null values\n\n* cargo fmt",
          "timestamp": "2020-06-02T13:45:10+02:00",
          "tree_id": "57a4112849923780f60cc11568ee76eda26fbb66",
          "url": "https://github.com/boa-dev/boa/commit/4f2191ae1f82b95dd5ffe6a5cfc6d2dcaaada6c5"
        },
        "date": 1591098799026,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 412.69,
            "range": "+/- 9.110",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 438.18,
            "range": "+/- 8.540",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 413.38,
            "range": "+/- 6.410",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4239,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9476,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 941.51,
            "range": "+/- 17.770",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2152,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9184,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.181,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.844,
            "range": "+/- 0.203",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bb2b6f638cba59269694d44424d16c9a5ede6a81",
          "message": "Added `TypeError` implementation (#442)",
          "timestamp": "2020-06-02T16:52:28+02:00",
          "tree_id": "3f631a09a4431081a4c2679fc4c3f858a8af52e1",
          "url": "https://github.com/boa-dev/boa/commit/bb2b6f638cba59269694d44424d16c9a5ede6a81"
        },
        "date": 1591110157018,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 471.5,
            "range": "+/- 7.450",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 501.75,
            "range": "+/- 10.070",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 487.01,
            "range": "+/- 6.110",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.7653,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2077,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0971,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 6.0621,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3378,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4081,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.571,
            "range": "+/- 0.205",
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
          "id": "32b0741cc85526b0cdab5509ddc538567129a05b",
          "message": "Profiler using measureme (#317)\n\nProfiler",
          "timestamp": "2020-06-05T14:37:06+01:00",
          "tree_id": "b43860e1b7d703e508b4dcff164598bf188af77d",
          "url": "https://github.com/boa-dev/boa/commit/32b0741cc85526b0cdab5509ddc538567129a05b"
        },
        "date": 1591364868599,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 495.86,
            "range": "+/- 13.640",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 517.17,
            "range": "+/- 7.570",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 521.52,
            "range": "+/- 12.710",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.9466,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2517,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0583,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6957,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3097,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4754,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.886,
            "range": "+/- 0.411",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "halidodat@gmail.com",
            "name": "HalidOdat",
            "username": "HalidOdat"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d847ff826b5164791191163f3e273cb1d2c247a0",
          "message": "Specification compliant `ToBigInt` (`to_bigint`) (#450)\n\n- Merged Ast `BigInt` to Builtin `BigInt`.\r\n - Split `BigInt` logic to separate files.\r\n - Added `builtins/bigint/operations.rs` for `BigInt` operations.\r\n - Added `builtins/bigint/conversions.rs` for `BigInt` conversions.\r\n - Added` builtins/bigint/equality.rs` for `BigInt` equality checking.\r\n - Added tests.",
          "timestamp": "2020-06-06T04:07:42+02:00",
          "tree_id": "3ee9d481622b8e83e8c5a0bf7cdb7776171c06d7",
          "url": "https://github.com/boa-dev/boa/commit/d847ff826b5164791191163f3e273cb1d2c247a0"
        },
        "date": 1591409734644,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 390.76,
            "range": "+/- 12.270",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 417.87,
            "range": "+/- 10.610",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 424.38,
            "range": "+/- 12.980",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4078,
            "range": "+/- 0.057",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.034,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 839.87,
            "range": "+/- 22.040",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5478,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9334,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9566,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.686,
            "range": "+/- 0.567",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "84574b5da87ca8dbe4d975d317e73167135aaba9",
          "message": "Divide the `run()` function (#422)\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-06-06T19:55:45+02:00",
          "tree_id": "b72e2b978aaaa238780b9c3507ad2ae1263918d6",
          "url": "https://github.com/boa-dev/boa/commit/84574b5da87ca8dbe4d975d317e73167135aaba9"
        },
        "date": 1591466584002,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 381.99,
            "range": "+/- 13.270",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 395.37,
            "range": "+/- 9.300",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 393.38,
            "range": "+/- 13.010",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1658,
            "range": "+/- 0.058",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.6802,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 780.55,
            "range": "+/- 18.940",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.2939,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2822,
            "range": "+/- 0.188",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8109,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.411,
            "range": "+/- 0.348",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "691c0d4f3b0fe3731c21b343314fe82ab36bf4c7",
          "message": "Bump websocket-extensions from 0.1.3 to 0.1.4 (#460)\n\nBumps [websocket-extensions](https://github.com/faye/websocket-extensions-node) from 0.1.3 to 0.1.4.\n- [Release notes](https://github.com/faye/websocket-extensions-node/releases)\n- [Changelog](https://github.com/faye/websocket-extensions-node/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/faye/websocket-extensions-node/compare/0.1.3...0.1.4)\n\nSigned-off-by: dependabot[bot] <support@github.com>\n\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2020-06-06T23:34:33+02:00",
          "tree_id": "382bc26fb011779e215c26c1db6a6c3297725ea4",
          "url": "https://github.com/boa-dev/boa/commit/691c0d4f3b0fe3731c21b343314fe82ab36bf4c7"
        },
        "date": 1591479730161,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 417.22,
            "range": "+/- 10.970",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 466.63,
            "range": "+/- 11.170",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 435.52,
            "range": "+/- 14.640",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3212,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8815,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 869.15,
            "range": "+/- 29.370",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.7111,
            "range": "+/- 0.155",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6489,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.009,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.203,
            "range": "+/- 0.403",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d970cf96b57681bf8c73f0b16351f4a724e306c3",
          "message": "Execution benchmarks only take execution into account (#431)",
          "timestamp": "2020-06-07T12:21:31+02:00",
          "tree_id": "fd39bcce2ab77d69c7cb6dfab3db9b9a9eb790fd",
          "url": "https://github.com/boa-dev/boa/commit/d970cf96b57681bf8c73f0b16351f4a724e306c3"
        },
        "date": 1591525792727,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 486.11,
            "range": "+/- 4.480",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.664,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 67.616,
            "range": "+/- 0.642",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1911,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.169,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0006,
            "range": "+/- -996.366",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5571,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3264,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.379,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.394,
            "range": "+/- 0.099",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "863abb37474d6e39ccb81e13fc56aa2739aa8d2c",
          "message": "[TemplateLiteral] Basic lexer implementation (#455)\n\n* Basic template literal lexer implementation\r\n\r\n* Fix formatting",
          "timestamp": "2020-06-08T20:29:15+02:00",
          "tree_id": "3b2bca5fd448af5fc4a3738ce30e3e426287932e",
          "url": "https://github.com/boa-dev/boa/commit/863abb37474d6e39ccb81e13fc56aa2739aa8d2c"
        },
        "date": 1591641455266,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 461.53,
            "range": "+/- 7.740",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.317,
            "range": "+/- 0.587",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.226,
            "range": "+/- 0.629",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1653,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.104,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 974.69,
            "range": "+/- 30.480",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3924,
            "range": "+/- 0.191",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2862,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.31,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.719,
            "range": "+/- 0.212",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a78934d424f17bc8b9132fdb14704b61e240bd40",
          "message": "parseInt, parseFloat implementation (#459)\n\n* Added documentation to make_builtin_fn\r\n\r\n* Simple impl of parseInt()\r\n\r\n* Impl rest of parse_int\r\n\r\n* Fixed handling of strings starting 0x\r\n\r\n* Made NaN return as per js spec\r\n\r\n* Rework to improve clarity\r\n\r\n* parseFloat impl, added tests\r\n\r\n* Addressed comments to PR\r\n\r\n* Removed f64 import\r\n\r\n* Fixed handling of too many/few arguments to parseInt/Float\r\n\r\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-06-09T16:16:57+02:00",
          "tree_id": "43525f5d8132c87f8de1343ba81f82d994291d5b",
          "url": "https://github.com/boa-dev/boa/commit/a78934d424f17bc8b9132fdb14704b61e240bd40"
        },
        "date": 1591712760546,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 517.25,
            "range": "+/- 11.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 19.715,
            "range": "+/- 0.392",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 74.043,
            "range": "+/- 1.048",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4571,
            "range": "+/- 0.052",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3045,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0979,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.8721,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.403,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.5127,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.382,
            "range": "+/- 0.266",
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
          "id": "081884e8d0735c6dc237afc739714b218d318235",
          "message": "Use latest compare action",
          "timestamp": "2020-06-09T21:11:22+01:00",
          "tree_id": "788be43c672bc1ba5a7886b96673f4266baf2582",
          "url": "https://github.com/boa-dev/boa/commit/081884e8d0735c6dc237afc739714b218d318235"
        },
        "date": 1591733975889,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 466.75,
            "range": "+/- 4.630",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.595,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.73,
            "range": "+/- 1.042",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1182,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0955,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 974.21,
            "range": "+/- 6.890",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3895,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1915,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2663,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.948,
            "range": "+/- 0.215",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "n14little@gmail.com",
            "name": "n14little",
            "username": "n14little"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4763907fbfc0d3ea85a4d145022b5eada29ee6df",
          "message": "Add proto object to from json (#462)\n\n* remove From<JSONValue> for Value. Going from JSONValue to Value will soon need the interpreter\n\n* pass interpreter to from_json\n\n* move from_json to Value impl\n\n* move to_json to Value impl alongside from_json\n\n* add prototype to objects created from json\n\n* consume the object and don't clone\n\n* if it fits into i32, use integer; otherwise, use a rational\n\n* WIP: throwing type error\n\n* address most of the error cases\n\n* cargo fmt\n\n* address the rest of the error cases\n\n* return null when JSONNumber::from_f64() returns None\n\n* cargo fmt\n\n* Update boa/src/builtins/value/mod.rs\n\n* use JSONValue and use Result\n\n* Update boa/src/builtins/json/mod.rs\r\n\r\nUse and_then to avoid flatten\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-06-09T22:16:11+02:00",
          "tree_id": "66509e16e727664dc7d45db64a3323bd70421e80",
          "url": "https://github.com/boa-dev/boa/commit/4763907fbfc0d3ea85a4d145022b5eada29ee6df"
        },
        "date": 1591734313068,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 493.07,
            "range": "+/- 15.970",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 18.522,
            "range": "+/- 0.580",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 73.799,
            "range": "+/- 2.660",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3081,
            "range": "+/- 0.080",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1658,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0717,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7907,
            "range": "+/- 0.207",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9903,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4298,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.793,
            "range": "+/- 0.481",
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
          "id": "a4ae22ed60d9ae2f067a7ff323c5064fe9a6df5b",
          "message": "Forgot to build criterion benchmarks",
          "timestamp": "2020-06-09T21:19:24+01:00",
          "tree_id": "a3228a8e22cce5eaf794e22aa58f87d5c1d3f49c",
          "url": "https://github.com/boa-dev/boa/commit/a4ae22ed60d9ae2f067a7ff323c5064fe9a6df5b"
        },
        "date": 1591734457223,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 463.07,
            "range": "+/- 4.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.522,
            "range": "+/- 0.227",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 62.891,
            "range": "+/- 0.412",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1362,
            "range": "+/- 0.012",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0591,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 970.81,
            "range": "+/- 10.470",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3311,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1125,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2825,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.723,
            "range": "+/- 0.153",
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
          "id": "a76b8d489fe47127fe8db91714134834300eab54",
          "message": "Add inputs to yml",
          "timestamp": "2020-06-09T21:27:45+01:00",
          "tree_id": "507c90c4ae62c1652ff29cea1af1f71dde5333ae",
          "url": "https://github.com/boa-dev/boa/commit/a76b8d489fe47127fe8db91714134834300eab54"
        },
        "date": 1591734983872,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 463.55,
            "range": "+/- 4.150",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 18.373,
            "range": "+/- 0.249",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 69.71,
            "range": "+/- 0.551",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2461,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1089,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0254,
            "range": "+/- 0.010",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4019,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1706,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3534,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.013,
            "range": "+/- 0.179",
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
          "id": "f8821bed93aaa3125eafba20b8af05566fe28303",
          "message": "adding package",
          "timestamp": "2020-06-09T22:41:13+01:00",
          "tree_id": "11debb4cc61b0a5bbb9301ad4845b50ec94445da",
          "url": "https://github.com/boa-dev/boa/commit/f8821bed93aaa3125eafba20b8af05566fe28303"
        },
        "date": 1591739359300,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 464.69,
            "range": "+/- 5.180",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.63,
            "range": "+/- 0.208",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 63.785,
            "range": "+/- 0.708",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1287,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1427,
            "range": "+/- 0.023",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 974.05,
            "range": "+/- 11.760",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3869,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1275,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2976,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.818,
            "range": "+/- 0.152",
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
          "id": "b888bae2c8b43b8f7e7c42999b78095cff6c976a",
          "message": "stick to cwd",
          "timestamp": "2020-06-09T22:51:30+01:00",
          "tree_id": "504e54ffee0d9351b68f7823b2bbc3599cbffda7",
          "url": "https://github.com/boa-dev/boa/commit/b888bae2c8b43b8f7e7c42999b78095cff6c976a"
        },
        "date": 1591739969435,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 462.77,
            "range": "+/- 7.070",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.543,
            "range": "+/- 0.225",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 63.452,
            "range": "+/- 0.618",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1809,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0961,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 982.65,
            "range": "+/- 24.140",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4752,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1462,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3214,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.914,
            "range": "+/- 0.273",
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
          "id": "cf3251dd9aeefbdf6d8ac785209921accfd77ccd",
          "message": "references",
          "timestamp": "2020-06-09T23:01:09+01:00",
          "tree_id": "4592d749b264c5b204d2fbcc141284906f4b528b",
          "url": "https://github.com/boa-dev/boa/commit/cf3251dd9aeefbdf6d8ac785209921accfd77ccd"
        },
        "date": 1591740573152,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 461.5,
            "range": "+/- 5.730",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.398,
            "range": "+/- 0.302",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 62.599,
            "range": "+/- 0.865",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0813,
            "range": "+/- 0.022",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0666,
            "range": "+/- 0.020",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 963.95,
            "range": "+/- 9.940",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3163,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1319,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2592,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.671,
            "range": "+/- 0.246",
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
          "id": "2736f7bc1bb3605f0bc14cb1b0a8d4164a07e95e",
          "message": "Update pull_request.yml",
          "timestamp": "2020-06-09T23:07:53+01:00",
          "tree_id": "c11345cdf83702b1dde94e82d93268f7f5f087ab",
          "url": "https://github.com/boa-dev/boa/commit/2736f7bc1bb3605f0bc14cb1b0a8d4164a07e95e"
        },
        "date": 1591740951928,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 432.71,
            "range": "+/- 6.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 16.758,
            "range": "+/- 0.281",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 61.701,
            "range": "+/- 1.574",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0102,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9656,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 908,
            "range": "+/- 11.480",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9824,
            "range": "+/- 0.128",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6579,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1014,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.917,
            "range": "+/- 0.275",
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
          "id": "981404a518ac02b21d85c0a47195e650b36507fe",
          "message": "Update pull_request.yml",
          "timestamp": "2020-06-09T23:47:22+01:00",
          "tree_id": "8422a1271d80148f661287e649b2369386984329",
          "url": "https://github.com/boa-dev/boa/commit/981404a518ac02b21d85c0a47195e650b36507fe"
        },
        "date": 1591743332083,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 463.44,
            "range": "+/- 4.190",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.687,
            "range": "+/- 0.248",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.308,
            "range": "+/- 0.459",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1463,
            "range": "+/- 0.021",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0786,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 995.69,
            "range": "+/- -987.066",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3324,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1433,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2676,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.871,
            "range": "+/- 0.249",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ca80114c531d5bfcf6ce4b7e85af39f50a711653",
          "message": "Fix and tests (#469)",
          "timestamp": "2020-06-10T08:11:03+02:00",
          "tree_id": "37c50d4db150137b78128fef05f04499ebb65f9e",
          "url": "https://github.com/boa-dev/boa/commit/ca80114c531d5bfcf6ce4b7e85af39f50a711653"
        },
        "date": 1591769929662,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 409.23,
            "range": "+/- 12.910",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 16.679,
            "range": "+/- 0.311",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 60.833,
            "range": "+/- 1.256",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.944,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8255,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 876.66,
            "range": "+/- 28.220",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.8477,
            "range": "+/- 0.104",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4025,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8731,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 10.609,
            "range": "+/- 0.317",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "paul@lancasterzone.com",
            "name": "Paul Lancaster",
            "username": "Lan2u"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "634dfb7e80c767946cbafd4a73450ee41ed61a8a",
          "message": "Optimise type comparisons (#441)\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-06-10T12:25:40+02:00",
          "tree_id": "ff4264792f914ab62a89143a0e278efb0d2dd8ed",
          "url": "https://github.com/boa-dev/boa/commit/634dfb7e80c767946cbafd4a73450ee41ed61a8a"
        },
        "date": 1591785240822,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 472.14,
            "range": "+/- 6.930",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 18.029,
            "range": "+/- 0.305",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 65.988,
            "range": "+/- 1.168",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1514,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1267,
            "range": "+/- 0.032",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 968.64,
            "range": "+/- 7.390",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4854,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1781,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3865,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.916,
            "range": "+/- 0.225",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "64dbf13afd15f12f958daa87a3d236dc9af1a9aa",
          "message": "Implemented #427, #429 and #430, and upgraded dependencies (#472)",
          "timestamp": "2020-06-10T18:44:59+02:00",
          "tree_id": "4c26f7064ac38a267733d9302707fa0005b505b3",
          "url": "https://github.com/boa-dev/boa/commit/64dbf13afd15f12f958daa87a3d236dc9af1a9aa"
        },
        "date": 1591808232321,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 455.92,
            "range": "+/- 9.100",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.556,
            "range": "+/- 0.456",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 68.375,
            "range": "+/- 1.700",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2662,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.974,
            "range": "+/- 0.392",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.798,
            "range": "+/- 0.370",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.629,
            "range": "+/- 0.391",
            "unit": "us"
          },
          {
            "name": "",
            "value": 90.763,
            "range": "+/- 1.861",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 89.002,
            "range": "+/- 1.659",
            "unit": "us"
          },
          {
            "name": "",
            "value": 101.23,
            "range": "+/- 2.757",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 99.186,
            "range": "+/- 2.529",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0247,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 972.96,
            "range": "+/- 25.700",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0913,
            "range": "+/- 0.113",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9242,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.16,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.576,
            "range": "+/- 0.341",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.2411,
            "range": "+/- 0.095",
            "unit": "ms"
          }
        ]
      }
    ]
  }
}