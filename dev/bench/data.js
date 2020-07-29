window.BENCHMARK_DATA = {
  "lastUpdate": 1596037502360,
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
          "id": "73d7a64bb4da6040e3424297082d47debb3c91b8",
          "message": "add array prototype to arrays built in from_json (#476)",
          "timestamp": "2020-06-10T20:24:23+02:00",
          "tree_id": "c9cdda127f09fa56ee3ac6c3dcf29e59ab45fe67",
          "url": "https://github.com/boa-dev/boa/commit/73d7a64bb4da6040e3424297082d47debb3c91b8"
        },
        "date": 1591814080065,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 462.09,
            "range": "+/- 5.280",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.89,
            "range": "+/- 0.297",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.783,
            "range": "+/- 0.604",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1135,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.561,
            "range": "+/- 0.271",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.376,
            "range": "+/- 0.108",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.411,
            "range": "+/- 0.345",
            "unit": "us"
          },
          {
            "name": "",
            "value": 84.818,
            "range": "+/- 1.044",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 85.271,
            "range": "+/- 1.397",
            "unit": "us"
          },
          {
            "name": "",
            "value": 92.987,
            "range": "+/- 1.982",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 91.82,
            "range": "+/- 1.300",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0921,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 976.42,
            "range": "+/- 14.640",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2981,
            "range": "+/- 0.041",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2269,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2172,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.719,
            "range": "+/- 0.216",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.2269,
            "range": "+/- 0.046",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "csuckow99@gmail.com",
            "name": "Colin Suckow",
            "username": "Colin-Suckow"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "96dec21c020265fce3d4e86a18bc07d226c22ad5",
          "message": "Implement PreferredType enum (#470)",
          "timestamp": "2020-06-10T22:55:00+02:00",
          "tree_id": "24b4856e3066d00e9972fab3d73539359883f2be",
          "url": "https://github.com/boa-dev/boa/commit/96dec21c020265fce3d4e86a18bc07d226c22ad5"
        },
        "date": 1591823199309,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 451.53,
            "range": "+/- 7.570",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.09,
            "range": "+/- 0.296",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 63.555,
            "range": "+/- 1.083",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0437,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.573,
            "range": "+/- 0.306",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.756,
            "range": "+/- 0.286",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.687,
            "range": "+/- 0.342",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.852,
            "range": "+/- 1.245",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 83.724,
            "range": "+/- 1.839",
            "unit": "us"
          },
          {
            "name": "",
            "value": 90.366,
            "range": "+/- 1.788",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 94.505,
            "range": "+/- 2.325",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0022,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 949.12,
            "range": "+/- 15.370",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2702,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0066,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1668,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.107,
            "range": "+/- 0.178",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.0233,
            "range": "+/- 0.053",
            "unit": "ms"
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
          "id": "496278c46025d2a5ce1a13a4012bcc2ce28ded04",
          "message": "Fix compilation error",
          "timestamp": "2020-06-11T01:07:15+02:00",
          "tree_id": "3d29fb52953ce4eb1ddc4578d4e3a16e7a50ee3a",
          "url": "https://github.com/boa-dev/boa/commit/496278c46025d2a5ce1a13a4012bcc2ce28ded04"
        },
        "date": 1591831180358,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 539.19,
            "range": "+/- 17.040",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 19.637,
            "range": "+/- 0.561",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 76.256,
            "range": "+/- 2.090",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4019,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 18.383,
            "range": "+/- 0.439",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.404,
            "range": "+/- 0.435",
            "unit": "us"
          },
          {
            "name": "",
            "value": 20.514,
            "range": "+/- 0.630",
            "unit": "us"
          },
          {
            "name": "",
            "value": 99.981,
            "range": "+/- 2.670",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 99.377,
            "range": "+/- 3.145",
            "unit": "us"
          },
          {
            "name": "",
            "value": 108.48,
            "range": "+/- 3.380",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 108.87,
            "range": "+/- 2.930",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2219,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0715,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7615,
            "range": "+/- 0.183",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1065,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3828,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.78,
            "range": "+/- 0.404",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.6548,
            "range": "+/- 0.108",
            "unit": "ms"
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
          "id": "5a45ab532e61cf3e1ec9d6a87d407569323f3afb",
          "message": "[NaN] handle NaN token as identifier (#475)",
          "timestamp": "2020-06-11T18:49:08+02:00",
          "tree_id": "f7ec5b5de4d4bbdf0f1be15d9167a0d053375cae",
          "url": "https://github.com/boa-dev/boa/commit/5a45ab532e61cf3e1ec9d6a87d407569323f3afb"
        },
        "date": 1591894773432,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 511.24,
            "range": "+/- 8.800",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 18.17,
            "range": "+/- 0.267",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 65.333,
            "range": "+/- 0.803",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1389,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.952,
            "range": "+/- 0.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.579,
            "range": "+/- 0.222",
            "unit": "us"
          },
          {
            "name": "",
            "value": 19.157,
            "range": "+/- 0.237",
            "unit": "us"
          },
          {
            "name": "",
            "value": 87.485,
            "range": "+/- 1.757",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 86.833,
            "range": "+/- 1.074",
            "unit": "us"
          },
          {
            "name": "",
            "value": 94.24,
            "range": "+/- 1.165",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 95.62,
            "range": "+/- 1.179",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0902,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 923.77,
            "range": "+/- 7.330",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.32,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1137,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2162,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.799,
            "range": "+/- 0.263",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1834,
            "range": "+/- 0.041",
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
          "id": "a5ed8b77439f0b6314a7f2dac75acc64710138a8",
          "message": "Small optimisation in som types that can be `Copy` (#479)",
          "timestamp": "2020-06-12T09:31:02+02:00",
          "tree_id": "862914f91123b2a7ffd5d2f1fa3291090cffff97",
          "url": "https://github.com/boa-dev/boa/commit/a5ed8b77439f0b6314a7f2dac75acc64710138a8"
        },
        "date": 1591947691946,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 499.56,
            "range": "+/- 7.180",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.827,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.507,
            "range": "+/- 1.145",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1469,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.562,
            "range": "+/- 0.272",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.155,
            "range": "+/- 0.306",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.473,
            "range": "+/- 0.309",
            "unit": "us"
          },
          {
            "name": "",
            "value": 86.742,
            "range": "+/- 2.651",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 86.307,
            "range": "+/- 1.765",
            "unit": "us"
          },
          {
            "name": "",
            "value": 91.695,
            "range": "+/- 1.659",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 93.676,
            "range": "+/- 1.765",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0636,
            "range": "+/- 0.041",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 926.62,
            "range": "+/- 20.100",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2706,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2075,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3346,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.661,
            "range": "+/- 0.153",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.213,
            "range": "+/- 0.063",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "vasquez.gregcs@gmail.com",
            "name": "gvasquez11",
            "username": "gvasquez11"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3ff200ea11d25a51c899bc3e85ceefe1e4f84bfd",
          "message": "updated issue #484 (#485)",
          "timestamp": "2020-06-12T17:47:31+02:00",
          "tree_id": "688be386fd439ba6fcb230b6c9f3f16a8db41e21",
          "url": "https://github.com/boa-dev/boa/commit/3ff200ea11d25a51c899bc3e85ceefe1e4f84bfd"
        },
        "date": 1591977439214,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 420.39,
            "range": "+/- 7.600",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 15.382,
            "range": "+/- 0.561",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.302,
            "range": "+/- 1.897",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.9146,
            "range": "+/- 0.036",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.522,
            "range": "+/- 0.327",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.424,
            "range": "+/- 0.681",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.344,
            "range": "+/- 0.499",
            "unit": "us"
          },
          {
            "name": "",
            "value": 82.187,
            "range": "+/- 3.234",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 79.79,
            "range": "+/- 2.680",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.904,
            "range": "+/- 4.125",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 84.874,
            "range": "+/- 1.800",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8267,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 788.57,
            "range": "+/- 18.110",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.4384,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2767,
            "range": "+/- 0.163",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8432,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.15,
            "range": "+/- 0.361",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.5613,
            "range": "+/- 0.188",
            "unit": "ms"
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
          "id": "a7a5862458dae487dd3f16f1a8b25a277ed1d2a2",
          "message": "Add benchmarks for array access, create and pop operations. (#458)",
          "timestamp": "2020-06-12T20:02:30+02:00",
          "tree_id": "5f5a7d2dc8a8f11c6d1a58fa43379c893551c2e6",
          "url": "https://github.com/boa-dev/boa/commit/a7a5862458dae487dd3f16f1a8b25a277ed1d2a2"
        },
        "date": 1591985701873,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 462.95,
            "range": "+/- 11.170",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 16.902,
            "range": "+/- 0.379",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 65.229,
            "range": "+/- 1.409",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1199,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 34.871,
            "range": "+/- 0.721",
            "unit": "us"
          },
          {
            "name": "",
            "value": 14.022,
            "range": "+/- 0.136",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 8.2826,
            "range": "+/- 0.092",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.776,
            "range": "+/- 0.324",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.85,
            "range": "+/- 0.558",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.512,
            "range": "+/- 0.325",
            "unit": "us"
          },
          {
            "name": "",
            "value": 84.002,
            "range": "+/- 1.890",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 82.869,
            "range": "+/- 1.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 90.739,
            "range": "+/- 1.379",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 94.394,
            "range": "+/- 2.875",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9383,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 861.85,
            "range": "+/- 10.700",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.8739,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4737,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.992,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.708,
            "range": "+/- 0.133",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.9512,
            "range": "+/- 0.056",
            "unit": "ms"
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
          "id": "542b2cc005c46ab7f09758c24f5a6c6ea95ff178",
          "message": "Switch impl (#451)\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-06-12T20:09:02+02:00",
          "tree_id": "cb04e8aaeaa7d4728904b360e1975efbbdabbac5",
          "url": "https://github.com/boa-dev/boa/commit/542b2cc005c46ab7f09758c24f5a6c6ea95ff178"
        },
        "date": 1591986132557,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 495.67,
            "range": "+/- 12.200",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.491,
            "range": "+/- 0.320",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 67.576,
            "range": "+/- 1.105",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2178,
            "range": "+/- 0.043",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 37.707,
            "range": "+/- 1.245",
            "unit": "us"
          },
          {
            "name": "",
            "value": 14.771,
            "range": "+/- 0.136",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 8.8122,
            "range": "+/- 0.091",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.697,
            "range": "+/- 0.326",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.593,
            "range": "+/- 0.338",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.461,
            "range": "+/- 0.339",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.999,
            "range": "+/- 2.071",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 89.624,
            "range": "+/- 2.188",
            "unit": "us"
          },
          {
            "name": "",
            "value": 98.795,
            "range": "+/- 2.439",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 99.098,
            "range": "+/- 1.961",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0863,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 971.88,
            "range": "+/- 24.500",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4395,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2644,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2675,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.393,
            "range": "+/- 0.248",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.5128,
            "range": "+/- 0.087",
            "unit": "ms"
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
          "id": "c42fcf12a7dd9427b8a6925e8d10f8010ea5d2e3",
          "message": "[FunctionDeclaration - execution] evaluate declaration into undefined not function (#473)",
          "timestamp": "2020-06-12T20:08:29+02:00",
          "tree_id": "3c32890ed63efa9b6c0df754ee3890546c4beb61",
          "url": "https://github.com/boa-dev/boa/commit/c42fcf12a7dd9427b8a6925e8d10f8010ea5d2e3"
        },
        "date": 1591986147862,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 530.51,
            "range": "+/- 19.380",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 19.322,
            "range": "+/- 0.551",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 72.672,
            "range": "+/- 1.456",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3363,
            "range": "+/- 0.064",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 40.196,
            "range": "+/- 1.069",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.836,
            "range": "+/- 0.283",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 9.4946,
            "range": "+/- 0.148",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.797,
            "range": "+/- 0.343",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.376,
            "range": "+/- 0.569",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.711,
            "range": "+/- 0.606",
            "unit": "us"
          },
          {
            "name": "",
            "value": 95.678,
            "range": "+/- 2.072",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 98.882,
            "range": "+/- 3.356",
            "unit": "us"
          },
          {
            "name": "",
            "value": 105.32,
            "range": "+/- 3.510",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 104.86,
            "range": "+/- 3.210",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2821,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 982.19,
            "range": "+/- 21.020",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.501,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3393,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.443,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.875,
            "range": "+/- 0.426",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.8087,
            "range": "+/- 0.107",
            "unit": "ms"
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
          "id": "d042ddda3fb5239293b28db8383bc54991c039ee",
          "message": "[parser] folder structure for VariableStatement and ExpressionStatement (#489)",
          "timestamp": "2020-06-13T22:29:30+02:00",
          "tree_id": "61a2b2fd9a6828cd1106ad967f68890b2a45234e",
          "url": "https://github.com/boa-dev/boa/commit/d042ddda3fb5239293b28db8383bc54991c039ee"
        },
        "date": 1592080909847,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 476.77,
            "range": "+/- 8.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 17.002,
            "range": "+/- 0.229",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 62.167,
            "range": "+/- 1.758",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.0416,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 34.82,
            "range": "+/- 0.255",
            "unit": "us"
          },
          {
            "name": "",
            "value": 14.094,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 8.4808,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.193,
            "range": "+/- 0.194",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.109,
            "range": "+/- 0.164",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.987,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "",
            "value": 81.035,
            "range": "+/- 1.985",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 79.325,
            "range": "+/- 0.620",
            "unit": "us"
          },
          {
            "name": "",
            "value": 86.865,
            "range": "+/- 1.116",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 86.303,
            "range": "+/- 0.845",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9553,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 866.09,
            "range": "+/- 8.830",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9831,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.7706,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1179,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.892,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.805,
            "range": "+/- 0.026",
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
          "id": "d2939fffe324a4c3c90d40fab5df0c6d137f3f04",
          "message": "Added string benchmarks, and updated dependencies (#491)",
          "timestamp": "2020-06-14T16:32:07+02:00",
          "tree_id": "b41dc7882c8a75464fe37cf3f7c7b4b361b10251",
          "url": "https://github.com/boa-dev/boa/commit/d2939fffe324a4c3c90d40fab5df0c6d137f3f04"
        },
        "date": 1592146040015,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 517.94,
            "range": "+/- 2.320",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 18.582,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 67.033,
            "range": "+/- 0.468",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2215,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 38.446,
            "range": "+/- 0.534",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.732,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 9.44,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.904,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.687,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.912,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 88.005,
            "range": "+/- 0.612",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.553,
            "range": "+/- 1.222",
            "unit": "us"
          },
          {
            "name": "",
            "value": 95.83,
            "range": "+/- 0.728",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 95.315,
            "range": "+/- 0.487",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.262,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "",
            "value": 20.435,
            "range": "+/- 0.097",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 13.446,
            "range": "+/- 0.108",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1522,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 963.49,
            "range": "+/- 7.930",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4794,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1993,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3035,
            "range": "+/- 0.017",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.054,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.3708,
            "range": "+/- 0.010",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rsficken@gmail.com",
            "name": "Ryan Fickenscher",
            "username": "zanayr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "474252324ef26e53b22958a69307c9bb2645793e",
          "message": "Added `globalThis` property (#495)\n\n* added builtin globalThis\n\n* forgot to initialize globalThis in the builtin mod.rs file\n\n* changed  to  to match naming conventions and fixed issue with suggested edits to globalThis' initial value\n\n* updated the test for the  property as suggested",
          "timestamp": "2020-06-15T23:47:08+02:00",
          "tree_id": "beeed60f6436635b7f265d53528752b4d6889280",
          "url": "https://github.com/boa-dev/boa/commit/474252324ef26e53b22958a69307c9bb2645793e"
        },
        "date": 1592258526309,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 521.36,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 19.811,
            "range": "+/- 0.225",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 74.347,
            "range": "+/- 0.877",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.4121,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 40.946,
            "range": "+/- 0.324",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.068,
            "range": "+/- 0.072",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 9.3994,
            "range": "+/- 0.113",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 17.175,
            "range": "+/- 0.197",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.025,
            "range": "+/- 0.244",
            "unit": "us"
          },
          {
            "name": "",
            "value": 19.543,
            "range": "+/- 0.245",
            "unit": "us"
          },
          {
            "name": "",
            "value": 97.912,
            "range": "+/- 1.189",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 95.688,
            "range": "+/- 1.357",
            "unit": "us"
          },
          {
            "name": "",
            "value": 104.88,
            "range": "+/- 0.980",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 104.4,
            "range": "+/- 1.210",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.136,
            "range": "+/- 0.239",
            "unit": "us"
          },
          {
            "name": "",
            "value": 21.546,
            "range": "+/- 0.318",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 14.433,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2512,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0032,
            "range": "+/- -996.991",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5743,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1385,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3651,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.573,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.648,
            "range": "+/- 0.040",
            "unit": "ms"
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
          "id": "0d52a40c5394f4a5142d979b4cb54a1adc3e9157",
          "message": "Added `String`, `Boolean` and `Number` object benchmarks (#494)",
          "timestamp": "2020-06-15T23:54:12+02:00",
          "tree_id": "ed0572aad88a50a26990012cfef62dae09e2f3b7",
          "url": "https://github.com/boa-dev/boa/commit/0d52a40c5394f4a5142d979b4cb54a1adc3e9157"
        },
        "date": 1592259027973,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 522.43,
            "range": "+/- 10.390",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 19.663,
            "range": "+/- 0.211",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 68.064,
            "range": "+/- 0.816",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2282,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 40.647,
            "range": "+/- 1.046",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.127,
            "range": "+/- 0.160",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 9.5503,
            "range": "+/- 0.073",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 16.443,
            "range": "+/- 0.113",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.245,
            "range": "+/- 0.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 18.817,
            "range": "+/- 0.296",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.126,
            "range": "+/- 0.848",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.47,
            "range": "+/- 0.617",
            "unit": "us"
          },
          {
            "name": "",
            "value": 95.884,
            "range": "+/- 0.543",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 96.414,
            "range": "+/- 0.816",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.897,
            "range": "+/- 0.154",
            "unit": "us"
          },
          {
            "name": "",
            "value": 21.365,
            "range": "+/- 0.339",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 13.989,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "",
            "value": 30.632,
            "range": "+/- 0.323",
            "unit": "us"
          },
          {
            "name": "",
            "value": 34.923,
            "range": "+/- 0.255",
            "unit": "us"
          },
          {
            "name": "",
            "value": 50.793,
            "range": "+/- 0.571",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1582,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 965.84,
            "range": "+/- 3.990",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6521,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2065,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3544,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.183,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.403,
            "range": "+/- 0.034",
            "unit": "ms"
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
          "id": "df13272fc022887b2a39dc305c5396027568d8f7",
          "message": "Object specialization (#419)",
          "timestamp": "2020-06-16T00:56:52+02:00",
          "tree_id": "c9516adc9ebac2413cf37773346efbb77489a357",
          "url": "https://github.com/boa-dev/boa/commit/df13272fc022887b2a39dc305c5396027568d8f7"
        },
        "date": 1592262658033,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 174.77,
            "range": "+/- 3.650",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 8.8024,
            "range": "+/- 0.138",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 64.094,
            "range": "+/- 1.444",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2454,
            "range": "+/- 0.030",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 26.783,
            "range": "+/- 0.733",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2291,
            "range": "+/- 0.082",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.2831,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.852,
            "range": "+/- 0.347",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.132,
            "range": "+/- 0.266",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.969,
            "range": "+/- 0.275",
            "unit": "us"
          },
          {
            "name": "",
            "value": 80.19,
            "range": "+/- 1.486",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 79.107,
            "range": "+/- 1.800",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.999,
            "range": "+/- 3.332",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 86.608,
            "range": "+/- 2.261",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.274,
            "range": "+/- 0.347",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.947,
            "range": "+/- 0.221",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 8.2616,
            "range": "+/- 0.197",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.0317,
            "range": "+/- 0.158",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.5052,
            "range": "+/- 0.238",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.521,
            "range": "+/- 0.350",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1054,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 962.75,
            "range": "+/- 20.360",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2378,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.138,
            "range": "+/- 0.114",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.237,
            "range": "+/- 0.047",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.529,
            "range": "+/- 0.373",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1681,
            "range": "+/- 0.157",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "anirudhmkonduru@gmail.com",
            "name": "Anirudh Konduru",
            "username": "AnirudhKonduru"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "64fca0c16293b9b75fe9ffeeddb3373dbd6fae40",
          "message": "add Infinity gloabal property (#480) (#499)",
          "timestamp": "2020-06-16T07:04:46+02:00",
          "tree_id": "6b9495925aeb057b092a27241859ceba487c4855",
          "url": "https://github.com/boa-dev/boa/commit/64fca0c16293b9b75fe9ffeeddb3373dbd6fae40"
        },
        "date": 1592284754422,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 196.53,
            "range": "+/- 5.090",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.2252,
            "range": "+/- 0.149",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 65.268,
            "range": "+/- 1.662",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3688,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 28.805,
            "range": "+/- 0.788",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0954,
            "range": "+/- 0.109",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.5501,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.925,
            "range": "+/- 0.365",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.045,
            "range": "+/- 0.238",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.71,
            "range": "+/- 0.386",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.411,
            "range": "+/- 2.824",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.502,
            "range": "+/- 2.449",
            "unit": "us"
          },
          {
            "name": "",
            "value": 96.539,
            "range": "+/- 2.777",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 94.992,
            "range": "+/- 2.590",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.498,
            "range": "+/- 0.294",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.63,
            "range": "+/- 0.368",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 8.6232,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.3231,
            "range": "+/- 0.271",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.372,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.413,
            "range": "+/- 0.537",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.251,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.044,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6223,
            "range": "+/- 0.140",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3561,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4337,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.322,
            "range": "+/- 0.305",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.6461,
            "range": "+/- 0.104",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "5161147+neeldug@users.noreply.github.com",
            "name": "neeldug",
            "username": "neeldug"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "b43e92afa5bbd1cbd62e9d8bbceda2dc0f4a6ebc",
          "message": "Added error propagation in Field access (#500)",
          "timestamp": "2020-06-16T22:33:43+02:00",
          "tree_id": "bfde6f18d7eb65da68e1c6093bb24eee3955db12",
          "url": "https://github.com/boa-dev/boa/commit/b43e92afa5bbd1cbd62e9d8bbceda2dc0f4a6ebc"
        },
        "date": 1592340457216,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 191,
            "range": "+/- 1.380",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.0383,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 61.681,
            "range": "+/- 0.518",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.178,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 27.05,
            "range": "+/- 0.260",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1968,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.5395,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.043,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.578,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.629,
            "range": "+/- 0.183",
            "unit": "us"
          },
          {
            "name": "",
            "value": 80.365,
            "range": "+/- 0.834",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 80.379,
            "range": "+/- 0.708",
            "unit": "us"
          },
          {
            "name": "",
            "value": 84.996,
            "range": "+/- 0.937",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 85.013,
            "range": "+/- 0.876",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.791,
            "range": "+/- 0.113",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.509,
            "range": "+/- 0.158",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 7.994,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.77,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.7521,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.601,
            "range": "+/- 0.169",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1584,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 971.03,
            "range": "+/- 10.900",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5264,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2473,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3644,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.218,
            "range": "+/- 0.148",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.4292,
            "range": "+/- 0.026",
            "unit": "ms"
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
          "id": "4ae939ac5244227b8be7436288462198af5d5203",
          "message": "[ReferenceError] complete solution (#488)",
          "timestamp": "2020-06-16T22:58:21+02:00",
          "tree_id": "fd73ff48ff0d8497b259217549abc7a55d5acc6b",
          "url": "https://github.com/boa-dev/boa/commit/4ae939ac5244227b8be7436288462198af5d5203"
        },
        "date": 1592342023056,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 195.01,
            "range": "+/- 5.090",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.0421,
            "range": "+/- 0.224",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 68.002,
            "range": "+/- 0.833",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.3847,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 28.135,
            "range": "+/- 0.401",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9244,
            "range": "+/- 0.086",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.5072,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.407,
            "range": "+/- 0.186",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.875,
            "range": "+/- 0.342",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.08,
            "range": "+/- 0.255",
            "unit": "us"
          },
          {
            "name": "",
            "value": 87.546,
            "range": "+/- 2.580",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.642,
            "range": "+/- 2.762",
            "unit": "us"
          },
          {
            "name": "",
            "value": 93.473,
            "range": "+/- 1.818",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 92.995,
            "range": "+/- 1.762",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.501,
            "range": "+/- 0.277",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.882,
            "range": "+/- 0.281",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 8.3039,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.4747,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.148,
            "range": "+/- 0.217",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.061,
            "range": "+/- 0.290",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1947,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 984.21,
            "range": "+/- 10.580",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4753,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0387,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2774,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.184,
            "range": "+/- 0.264",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.6002,
            "range": "+/- 0.054",
            "unit": "ms"
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
          "id": "e674da46e720225663be79357703511dd4a54695",
          "message": "Fixed global objects initialization order (#502)",
          "timestamp": "2020-06-17T17:06:15+02:00",
          "tree_id": "1e3129a2bc54ff03a8c43038833758229a0a457d",
          "url": "https://github.com/boa-dev/boa/commit/e674da46e720225663be79357703511dd4a54695"
        },
        "date": 1592407164759,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 189.62,
            "range": "+/- 3.260",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.0259,
            "range": "+/- 0.339",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 53.082,
            "range": "+/- 1.211",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.7459,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 23.565,
            "range": "+/- 0.691",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5345,
            "range": "+/- 0.110",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.0802,
            "range": "+/- 0.093",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 9.6526,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.805,
            "range": "+/- 0.304",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.013,
            "range": "+/- 0.315",
            "unit": "us"
          },
          {
            "name": "",
            "value": 65.23,
            "range": "+/- 2.391",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 64.688,
            "range": "+/- 2.136",
            "unit": "us"
          },
          {
            "name": "",
            "value": 78.041,
            "range": "+/- 3.085",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 74.601,
            "range": "+/- 3.026",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.4785,
            "range": "+/- 0.261",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.4085,
            "range": "+/- 0.382",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.7176,
            "range": "+/- 0.278",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8054,
            "range": "+/- 0.145",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.5483,
            "range": "+/- 0.256",
            "unit": "us"
          },
          {
            "name": "",
            "value": 14.203,
            "range": "+/- 0.690",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.89,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 930.01,
            "range": "+/- 19.920",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.7667,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2034,
            "range": "+/- 0.172",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8245,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.619,
            "range": "+/- 0.363",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.3937,
            "range": "+/- 0.145",
            "unit": "ms"
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
          "id": "c19ef724e3230e1f4c3faf05aba8b0f5b3bc125a",
          "message": "Added undefined property to global scope (#501)",
          "timestamp": "2020-06-19T01:44:30+02:00",
          "tree_id": "83e55802999f9e88e62be988519c37043d7a9eab",
          "url": "https://github.com/boa-dev/boa/commit/c19ef724e3230e1f4c3faf05aba8b0f5b3bc125a"
        },
        "date": 1592524799216,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 200.46,
            "range": "+/- 2.560",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.7491,
            "range": "+/- 0.102",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 70.281,
            "range": "+/- 0.778",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.496,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 29.041,
            "range": "+/- 0.254",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.226,
            "range": "+/- 0.048",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.6005,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.849,
            "range": "+/- 0.156",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.391,
            "range": "+/- 0.237",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.491,
            "range": "+/- 0.207",
            "unit": "us"
          },
          {
            "name": "",
            "value": 88.729,
            "range": "+/- 2.304",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.409,
            "range": "+/- 1.263",
            "unit": "us"
          },
          {
            "name": "",
            "value": 96.462,
            "range": "+/- 4.621",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 94.042,
            "range": "+/- 0.937",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.465,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.367,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 8.601,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.2824,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.534,
            "range": "+/- 0.193",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.686,
            "range": "+/- 0.227",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.195,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0256,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7762,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3864,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4149,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.861,
            "range": "+/- 0.309",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.9672,
            "range": "+/- 0.101",
            "unit": "ms"
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
          "id": "8b431a4a1941dd6501b750929adddf806aed71c1",
          "message": "434 json parse enumerability (#504)",
          "timestamp": "2020-06-19T01:51:23+02:00",
          "tree_id": "6dee3232eaf0c12a12fb0083d5c97894b8066c2c",
          "url": "https://github.com/boa-dev/boa/commit/8b431a4a1941dd6501b750929adddf806aed71c1"
        },
        "date": 1592525115715,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 190.9,
            "range": "+/- 2.350",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 8.251,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 57.5,
            "range": "+/- 0.910",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.9771,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 25.014,
            "range": "+/- 0.398",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5764,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.4818,
            "range": "+/- 0.124",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.671,
            "range": "+/- 0.626",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.792,
            "range": "+/- 0.648",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.802,
            "range": "+/- 0.551",
            "unit": "us"
          },
          {
            "name": "",
            "value": 88.163,
            "range": "+/- 6.151",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 80.681,
            "range": "+/- 2.864",
            "unit": "us"
          },
          {
            "name": "",
            "value": 86.892,
            "range": "+/- 4.288",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 89.971,
            "range": "+/- 5.880",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.102,
            "range": "+/- 0.532",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.404,
            "range": "+/- 0.504",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 7.396,
            "range": "+/- 0.180",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0715,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.946,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.199,
            "range": "+/- 0.226",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0121,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 885.47,
            "range": "+/- 15.330",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1156,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8897,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1527,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.112,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.9121,
            "range": "+/- 0.037",
            "unit": "ms"
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
          "id": "a7cffe3c7c6fc3f0588a6fc1c84bd7dfeddd4bfd",
          "message": "Fixed function call with unspecified arguments (#506)\n\nCalling a function with less amount of arguments than\r\nthe function declaration parameters would `panic`.",
          "timestamp": "2020-06-19T03:15:20+02:00",
          "tree_id": "0c6d2ea80d144d77a238302ad0da258188a80604",
          "url": "https://github.com/boa-dev/boa/commit/a7cffe3c7c6fc3f0588a6fc1c84bd7dfeddd4bfd"
        },
        "date": 1592530139424,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 180.58,
            "range": "+/- 4.490",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 7.8483,
            "range": "+/- 0.183",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 52.716,
            "range": "+/- 1.231",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.8685,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 22.432,
            "range": "+/- 0.472",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.1574,
            "range": "+/- 0.101",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.2352,
            "range": "+/- 0.048",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 9.3902,
            "range": "+/- 0.226",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.7618,
            "range": "+/- 0.258",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.571,
            "range": "+/- 0.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 70.14,
            "range": "+/- 1.684",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 70.948,
            "range": "+/- 2.503",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.116,
            "range": "+/- 1.694",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 72.58,
            "range": "+/- 2.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.0478,
            "range": "+/- 0.245",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.7612,
            "range": "+/- 0.261",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.8892,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5685,
            "range": "+/- 0.163",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.2707,
            "range": "+/- 0.222",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.931,
            "range": "+/- 0.334",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8214,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 830.35,
            "range": "+/- 23.230",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.6344,
            "range": "+/- 0.128",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.4846,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.9294,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.163,
            "range": "+/- 0.337",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.4782,
            "range": "+/- 0.090",
            "unit": "ms"
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
          "id": "299a431efedb5d274cba2d964eb13c1fea119f51",
          "message": "JSON.stringify(undefined) should return undefined (#512)",
          "timestamp": "2020-06-19T12:55:39+02:00",
          "tree_id": "f53e0338f2cea727036e1339e63f1ce5c2f68d0c",
          "url": "https://github.com/boa-dev/boa/commit/299a431efedb5d274cba2d964eb13c1fea119f51"
        },
        "date": 1592564904325,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 161.89,
            "range": "+/- 3.420",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 7.3017,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 54.781,
            "range": "+/- 1.018",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.8803,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 21.41,
            "range": "+/- 0.425",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2569,
            "range": "+/- 0.069",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.9314,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 9.079,
            "range": "+/- 0.168",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.2127,
            "range": "+/- 0.259",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.396,
            "range": "+/- 0.202",
            "unit": "us"
          },
          {
            "name": "",
            "value": 66.949,
            "range": "+/- 1.898",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 66.21,
            "range": "+/- 1.579",
            "unit": "us"
          },
          {
            "name": "",
            "value": 70.115,
            "range": "+/- 1.668",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 71.073,
            "range": "+/- 1.899",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8395,
            "range": "+/- 0.194",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.9645,
            "range": "+/- 0.549",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 6.4925,
            "range": "+/- 0.151",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2158,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8699,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.462,
            "range": "+/- 0.315",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.6669,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 756.5,
            "range": "+/- 16.500",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.1729,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.9608,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.7732,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.257,
            "range": "+/- 0.404",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.0657,
            "range": "+/- 0.051",
            "unit": "ms"
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
          "id": "1ffeb5cbe18ebe25805908d485452f8b352024b9",
          "message": "Implement Object.is() method issue #513 (#515)",
          "timestamp": "2020-06-21T04:01:13+02:00",
          "tree_id": "15a282a9b33196e3fc6f2bbf987dbab0c9a7cbaf",
          "url": "https://github.com/boa-dev/boa/commit/1ffeb5cbe18ebe25805908d485452f8b352024b9"
        },
        "date": 1592705804084,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 216.57,
            "range": "+/- 5.400",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 9.3622,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 62.787,
            "range": "+/- 0.888",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.2241,
            "range": "+/- 0.043",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 27.637,
            "range": "+/- 0.738",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5019,
            "range": "+/- 0.110",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.7061,
            "range": "+/- 0.044",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 11.611,
            "range": "+/- 0.181",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.911,
            "range": "+/- 0.249",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.47,
            "range": "+/- 0.311",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.817,
            "range": "+/- 2.198",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 83.87,
            "range": "+/- 2.371",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.641,
            "range": "+/- 2.698",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 90.375,
            "range": "+/- 2.358",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.569,
            "range": "+/- 0.279",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.598,
            "range": "+/- 0.368",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 8.9632,
            "range": "+/- 0.217",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.1812,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.285,
            "range": "+/- 0.343",
            "unit": "us"
          },
          {
            "name": "",
            "value": 17.127,
            "range": "+/- 0.385",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2815,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0271,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.8541,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.7006,
            "range": "+/- 0.168",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3651,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.949,
            "range": "+/- 0.350",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.72,
            "range": "+/- 0.121",
            "unit": "ms"
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
          "id": "1ac5f205eb873c2aac8d665969776b4277caa2ba",
          "message": "Added arithmetic operation benchmark (#516)",
          "timestamp": "2020-06-22T09:42:13+02:00",
          "tree_id": "8533e05d009f729a46d71c23bfbd6c42c3bbbcf4",
          "url": "https://github.com/boa-dev/boa/commit/1ac5f205eb873c2aac8d665969776b4277caa2ba"
        },
        "date": 1592812645888,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 199.52,
            "range": "+/- 3.360",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 8.6633,
            "range": "+/- 0.155",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 59.085,
            "range": "+/- 1.087",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 2.1123,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 25.796,
            "range": "+/- 0.523",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7794,
            "range": "+/- 0.066",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.5071,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.465,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.907,
            "range": "+/- 0.177",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.79,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "",
            "value": 77.233,
            "range": "+/- 1.547",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 78.803,
            "range": "+/- 1.275",
            "unit": "us"
          },
          {
            "name": "",
            "value": 82.455,
            "range": "+/- 1.606",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 84.816,
            "range": "+/- 2.843",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.6064,
            "range": "+/- 0.321",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.335,
            "range": "+/- 0.279",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 7.8012,
            "range": "+/- 0.172",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5131,
            "range": "+/- 0.191",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.3416,
            "range": "+/- 0.217",
            "unit": "us"
          },
          {
            "name": "",
            "value": 16.108,
            "range": "+/- 0.421",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.8296,
            "range": "+/- 0.072",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0944,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 941.85,
            "range": "+/- 19.940",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3982,
            "range": "+/- 0.072",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1753,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1829,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.607,
            "range": "+/- 0.189",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.2574,
            "range": "+/- 0.111",
            "unit": "ms"
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
          "id": "69f48862eaac6ea2acca508276d00b57ac69e5dd",
          "message": "update pass through math methods (#519)\n\n* update pass through math methods\n\n* cargo fmt",
          "timestamp": "2020-06-23T06:50:54+02:00",
          "tree_id": "d53db025f8fa747078d76a52f8f134d1126485b3",
          "url": "https://github.com/boa-dev/boa/commit/69f48862eaac6ea2acca508276d00b57ac69e5dd"
        },
        "date": 1592888693892,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 196.31,
            "range": "+/- 2.660",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 8.304,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 56.137,
            "range": "+/- 0.494",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.9611,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 25.034,
            "range": "+/- 0.184",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5704,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 2.324,
            "range": "+/- 0.018",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.26,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.269,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.616,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "",
            "value": 76.355,
            "range": "+/- 1.033",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.161,
            "range": "+/- 1.291",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.362,
            "range": "+/- 1.307",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 78.934,
            "range": "+/- 0.985",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.921,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 10.643,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 7.366,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1298,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.7902,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "",
            "value": 15.371,
            "range": "+/- 0.172",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.67,
            "range": "+/- 0.022",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9641,
            "range": "+/- 0.014",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 884.13,
            "range": "+/- 6.540",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0887,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8266,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1286,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.967,
            "range": "+/- 0.198",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.9119,
            "range": "+/- 0.041",
            "unit": "ms"
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
          "id": "8f8498eac17164c8de2f599bd0b7ba2e8053ec30",
          "message": "`Value` refactor (#498)\n\n- Refactor `String` => `Rc<str>`\r\n - Refactor `Symbol` => `Rc<Symbol>`\r\n - Refactor `BigInt` => `RcBigInt`\r\n - Changed function signature, from `&mut Value` to `&Value`\r\n - Removed `Interpreter::value_to_rust_number()\r\n - Abstracted `Gc<GcCell<Object>>` to `GcObject`\r\n - Removed unnecessary `Box`s in global environment\r\n - Extracted `extensible` from internal slots\r\n - Made `to_primitive` throw errors\r\n - Removed `strict` parameter in `SameValue` function.\r\n - The `SameValue` function is not dependent on strict mode.",
          "timestamp": "2020-06-23T08:22:15+02:00",
          "tree_id": "7b4b0685ae06e32f4a0f027349f16d901bc420d6",
          "url": "https://github.com/boa-dev/boa/commit/8f8498eac17164c8de2f599bd0b7ba2e8053ec30"
        },
        "date": 1592894195146,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 129.85,
            "range": "+/- 2.610",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.0202,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.803,
            "range": "+/- 0.655",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 987.71,
            "range": "+/- 16.280",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.462,
            "range": "+/- 0.313",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.7705,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4108,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.0952,
            "range": "+/- 0.148",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.1291,
            "range": "+/- 0.162",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.809,
            "range": "+/- 0.138",
            "unit": "us"
          },
          {
            "name": "",
            "value": 71.031,
            "range": "+/- 1.707",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 71.51,
            "range": "+/- 1.631",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.473,
            "range": "+/- 2.118",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 76.419,
            "range": "+/- 2.754",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5962,
            "range": "+/- 0.132",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6522,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.3323,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.98,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3113,
            "range": "+/- 0.168",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.04,
            "range": "+/- 0.251",
            "unit": "us"
          },
          {
            "name": "",
            "value": 552.25,
            "range": "+/- 13.110",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0655,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 958.32,
            "range": "+/- 23.470",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2927,
            "range": "+/- 0.175",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.8901,
            "range": "+/- 0.138",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0822,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.253,
            "range": "+/- 0.403",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.9762,
            "range": "+/- 0.100",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rsficken@gmail.com",
            "name": "Ryan Fickenscher",
            "username": "zanayr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "24418e7520210add12e5966432d8100f39085587",
          "message": "TypeError when to_object is passed null or undefined (#518)\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-06-23T13:45:18+02:00",
          "tree_id": "8ab07271850f8180c896afc750999a72f1c7bb4b",
          "url": "https://github.com/boa-dev/boa/commit/24418e7520210add12e5966432d8100f39085587"
        },
        "date": 1592913596051,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 140.48,
            "range": "+/- 2.690",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.1923,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.083,
            "range": "+/- 0.352",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0096,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.265,
            "range": "+/- 0.234",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0007,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5287,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.559,
            "range": "+/- 0.205",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7611,
            "range": "+/- 0.207",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.4294,
            "range": "+/- 0.251",
            "unit": "us"
          },
          {
            "name": "",
            "value": 77.627,
            "range": "+/- 1.666",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 76.133,
            "range": "+/- 0.820",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.668,
            "range": "+/- 1.434",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 81.534,
            "range": "+/- 2.224",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0758,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2716,
            "range": "+/- 0.171",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.2539,
            "range": "+/- 0.412",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.4316,
            "range": "+/- 0.153",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4311,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.1901,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "",
            "value": 582.27,
            "range": "+/- 8.300",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1936,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0162,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.5926,
            "range": "+/- 0.088",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2206,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2945,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.285,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.4337,
            "range": "+/- 0.062",
            "unit": "ms"
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
          "id": "3fe894273c1e8b407bf64f4f306e0f5c7b597392",
          "message": "clean up the rest of the math methods (#523)\n\n* clean up the rest of the math methods\n\n* wrap match with Ok(Value::from(...))\n\n* cargo fmt",
          "timestamp": "2020-06-24T14:05:45+02:00",
          "tree_id": "30893f33f7a105ca0e365648f5da1d2254e6f4a7",
          "url": "https://github.com/boa-dev/boa/commit/3fe894273c1e8b407bf64f4f306e0f5c7b597392"
        },
        "date": 1593001211959,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 136.37,
            "range": "+/- 1.280",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2204,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.722,
            "range": "+/- 0.241",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0039,
            "range": "+/- -996.558",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 13.996,
            "range": "+/- 0.222",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.8914,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4593,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.375,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.4778,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2569,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "",
            "value": 72.839,
            "range": "+/- 1.181",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 72.678,
            "range": "+/- 1.084",
            "unit": "us"
          },
          {
            "name": "",
            "value": 76.849,
            "range": "+/- 0.837",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 76.783,
            "range": "+/- 0.957",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.754,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9171,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.7037,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2019,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2301,
            "range": "+/- 0.019",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.0294,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 580.81,
            "range": "+/- 7.200",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0897,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 945.28,
            "range": "+/- 5.410",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3644,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1085,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2166,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.963,
            "range": "+/- 0.325",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.0837,
            "range": "+/- 0.034",
            "unit": "ms"
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
          "id": "c4a652a517d171ce6e4a226fce98d03a1e9b15b8",
          "message": "Fix sending `this` value to function environments (#526)",
          "timestamp": "2020-06-25T16:52:42+02:00",
          "tree_id": "b6a925a1df65076183399e4702d89d09f8df00cd",
          "url": "https://github.com/boa-dev/boa/commit/c4a652a517d171ce6e4a226fce98d03a1e9b15b8"
        },
        "date": 1593097662308,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 160.41,
            "range": "+/- 5.040",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 5.3923,
            "range": "+/- 0.242",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 28.27,
            "range": "+/- 1.036",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.3388,
            "range": "+/- 0.045",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 17.49,
            "range": "+/- 0.452",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.3828,
            "range": "+/- 0.112",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.7155,
            "range": "+/- 0.056",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 7.8691,
            "range": "+/- 0.352",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.0819,
            "range": "+/- 0.268",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.0293,
            "range": "+/- 0.434",
            "unit": "us"
          },
          {
            "name": "",
            "value": 98.716,
            "range": "+/- 4.346",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 99.216,
            "range": "+/- 4.686",
            "unit": "us"
          },
          {
            "name": "",
            "value": 101.98,
            "range": "+/- 4.973",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 103.61,
            "range": "+/- 4.810",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.6351,
            "range": "+/- 0.306",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.2664,
            "range": "+/- 0.323",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.8394,
            "range": "+/- 0.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4697,
            "range": "+/- 0.208",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2837,
            "range": "+/- 0.176",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.287,
            "range": "+/- 0.487",
            "unit": "us"
          },
          {
            "name": "",
            "value": 651.8,
            "range": "+/- 13.200",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.6039,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.1548,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 6.4433,
            "range": "+/- 0.251",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.7291,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.6417,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 16.775,
            "range": "+/- 0.489",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 7.7494,
            "range": "+/- 0.269",
            "unit": "ms"
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
          "id": "8b40e9eec2c190f8df7cde6713c7fe44e2564756",
          "message": "updating docs to add summarize (#527)\n\n* updating docs to add summarize\r\n\r\n* add table",
          "timestamp": "2020-06-25T18:46:39+01:00",
          "tree_id": "dd0fa3be5528cb3cfed99b124a25533816401a42",
          "url": "https://github.com/boa-dev/boa/commit/8b40e9eec2c190f8df7cde6713c7fe44e2564756"
        },
        "date": 1593108020012,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 139.73,
            "range": "+/- 2.090",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.1663,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.546,
            "range": "+/- 0.701",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0011,
            "range": "+/- -996.083",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 13.888,
            "range": "+/- 0.139",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9068,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4645,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3583,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5072,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1267,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.046,
            "range": "+/- 1.418",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 72.542,
            "range": "+/- 0.766",
            "unit": "us"
          },
          {
            "name": "",
            "value": 77.932,
            "range": "+/- 2.354",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 76.882,
            "range": "+/- 1.122",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8376,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1485,
            "range": "+/- 0.109",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6896,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.257,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4339,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.9556,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "",
            "value": 575.32,
            "range": "+/- 3.080",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.104,
            "range": "+/- 0.009",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 931.81,
            "range": "+/- 8.370",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3294,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0707,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2496,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.957,
            "range": "+/- 0.167",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1474,
            "range": "+/- 0.037",
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
          "id": "477d408c102172c4f42b45162fe9ed61e97c46e5",
          "message": "Upgraded dependencies before the 0.9 release (#537)",
          "timestamp": "2020-07-02T11:35:55+02:00",
          "tree_id": "a1164550ec27fbe7c26808bee414f467a3dbfa6c",
          "url": "https://github.com/boa-dev/boa/commit/477d408c102172c4f42b45162fe9ed61e97c46e5"
        },
        "date": 1593683462641,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 152.64,
            "range": "+/- 2.780",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.7663,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 26.978,
            "range": "+/- 0.425",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1937,
            "range": "+/- 0.031",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.759,
            "range": "+/- 0.386",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1631,
            "range": "+/- 0.058",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5597,
            "range": "+/- 0.030",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 7.2034,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.4602,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.2657,
            "range": "+/- 0.158",
            "unit": "us"
          },
          {
            "name": "",
            "value": 89.14,
            "range": "+/- 2.717",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 88.861,
            "range": "+/- 2.301",
            "unit": "us"
          },
          {
            "name": "",
            "value": 94.495,
            "range": "+/- 5.194",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 94.72,
            "range": "+/- 2.561",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7049,
            "range": "+/- 0.132",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.0332,
            "range": "+/- 0.246",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.1804,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.6906,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0177,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.9012,
            "range": "+/- 0.149",
            "unit": "us"
          },
          {
            "name": "",
            "value": 603.66,
            "range": "+/- 15.900",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2623,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0514,
            "range": "+/- 0.015",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.741,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.5546,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4266,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.896,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.8146,
            "range": "+/- 0.086",
            "unit": "ms"
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
          "id": "109efcb3df282b6cae828b8847d3206c0dd81132",
          "message": "v0.9 changelog (#528)\n\nv0.9",
          "timestamp": "2020-07-02T17:57:29+01:00",
          "tree_id": "bea2a1f1e3e98c69221bbf73b18d23a5884f83e1",
          "url": "https://github.com/boa-dev/boa/commit/109efcb3df282b6cae828b8847d3206c0dd81132"
        },
        "date": 1593709876010,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 145.44,
            "range": "+/- 3.230",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.5828,
            "range": "+/- 0.155",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 25.303,
            "range": "+/- 0.681",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1435,
            "range": "+/- 0.032",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.129,
            "range": "+/- 0.416",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9539,
            "range": "+/- 0.078",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4509,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.7729,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1019,
            "range": "+/- 0.206",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.82,
            "range": "+/- 0.216",
            "unit": "us"
          },
          {
            "name": "",
            "value": 85.726,
            "range": "+/- 2.302",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 84.021,
            "range": "+/- 2.788",
            "unit": "us"
          },
          {
            "name": "",
            "value": 87.742,
            "range": "+/- 2.033",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 91.049,
            "range": "+/- 2.732",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.4335,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5848,
            "range": "+/- 0.195",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.1249,
            "range": "+/- 0.141",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.581,
            "range": "+/- 0.158",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.766,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.8055,
            "range": "+/- 0.263",
            "unit": "us"
          },
          {
            "name": "",
            "value": 617.52,
            "range": "+/- 16.940",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2943,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.019,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6217,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.366,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4216,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.535,
            "range": "+/- 0.295",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.6994,
            "range": "+/- 0.097",
            "unit": "ms"
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
          "id": "8be0cb55f0e1e05097db22bf0038afe1abeb46ff",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-07-02T17:59:13+01:00",
          "tree_id": "484c925b1a97b12ac7e62496230c3596eb774864",
          "url": "https://github.com/boa-dev/boa/commit/8be0cb55f0e1e05097db22bf0038afe1abeb46ff"
        },
        "date": 1593709955190,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 136.1,
            "range": "+/- 5.650",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.3577,
            "range": "+/- 0.159",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.742,
            "range": "+/- 0.551",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1346,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.011,
            "range": "+/- 0.596",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6749,
            "range": "+/- 0.134",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3784,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.5097,
            "range": "+/- 0.322",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.4591,
            "range": "+/- 0.252",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.3661,
            "range": "+/- 0.322",
            "unit": "us"
          },
          {
            "name": "",
            "value": 76.435,
            "range": "+/- 3.848",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.49,
            "range": "+/- 3.335",
            "unit": "us"
          },
          {
            "name": "",
            "value": 78.509,
            "range": "+/- 3.101",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 83.71,
            "range": "+/- 4.010",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0662,
            "range": "+/- 0.259",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8461,
            "range": "+/- 0.236",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6971,
            "range": "+/- 0.239",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2337,
            "range": "+/- 0.196",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3655,
            "range": "+/- 0.301",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.9295,
            "range": "+/- 0.399",
            "unit": "us"
          },
          {
            "name": "",
            "value": 573.02,
            "range": "+/- 32.140",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.109,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 926.86,
            "range": "+/- 39.860",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3725,
            "range": "+/- 0.311",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0631,
            "range": "+/- 0.232",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2857,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.795,
            "range": "+/- 0.523",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.5089,
            "range": "+/- 0.099",
            "unit": "ms"
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
          "id": "29d159739f9fca022ddf6b84cec9fecdfa3489ee",
          "message": "Update CHANGELOG.md",
          "timestamp": "2020-07-02T19:20:38+01:00",
          "tree_id": "ee6bb06f0a13f69f2bf7ffdf0c3d418ae00584d8",
          "url": "https://github.com/boa-dev/boa/commit/29d159739f9fca022ddf6b84cec9fecdfa3489ee"
        },
        "date": 1593714774891,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 135.29,
            "range": "+/- 2.640",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.0179,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.359,
            "range": "+/- 0.318",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 970.17,
            "range": "+/- 10.630",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.998,
            "range": "+/- 0.219",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.7824,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4491,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.0958,
            "range": "+/- 0.138",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2711,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9696,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "",
            "value": 71.86,
            "range": "+/- 1.426",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 71.218,
            "range": "+/- 1.044",
            "unit": "us"
          },
          {
            "name": "",
            "value": 76.131,
            "range": "+/- 1.347",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 77.306,
            "range": "+/- 1.532",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7352,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8406,
            "range": "+/- 0.114",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6394,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0138,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1909,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.4985,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "",
            "value": 557.26,
            "range": "+/- 8.630",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0322,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 916.66,
            "range": "+/- 14.470",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2008,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9975,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.112,
            "range": "+/- 0.034",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.266,
            "range": "+/- 0.179",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.9557,
            "range": "+/- 0.060",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jase.williams@gmail.com",
            "name": "jasonwilliams",
            "username": "jasonwilliams"
          },
          "committer": {
            "email": "jase.williams@gmail.com",
            "name": "jasonwilliams",
            "username": "jasonwilliams"
          },
          "distinct": true,
          "id": "73f65f7800917c92f86134eaa21751c1ca93d986",
          "message": "0.9.0",
          "timestamp": "2020-07-02T21:03:22+01:00",
          "tree_id": "5cec978c6763d12a9ff0361f20b171126138b67b",
          "url": "https://github.com/boa-dev/boa/commit/73f65f7800917c92f86134eaa21751c1ca93d986"
        },
        "date": 1593721062275,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 140.09,
            "range": "+/- 1.020",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.217,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.683,
            "range": "+/- 0.235",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0285,
            "range": "+/- 0.012",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.16,
            "range": "+/- 0.171",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9688,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4894,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3612,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6192,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2968,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.701,
            "range": "+/- 0.929",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.57,
            "range": "+/- 1.324",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.542,
            "range": "+/- 0.903",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 79.274,
            "range": "+/- 0.763",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9023,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.223,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.7268,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.3585,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5012,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.1845,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "",
            "value": 582.59,
            "range": "+/- 2.900",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1155,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 932.6,
            "range": "+/- 6.350",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3449,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1714,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3039,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.096,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.0615,
            "range": "+/- 0.137",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "tjd.rodgers@gmail.com",
            "name": "Te-jé Rodgers",
            "username": "mr-rodgers"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d9ef1fb426979e00173956d797afaa30d9168a90",
          "message": "add remaining math methods (#524) (#525)\n\n* add `Math.clz32` method (#524)\r\n\r\n* fix doc urls for clz32\r\n\r\n* [#524] optimize impl for `Math.clz32`\r\n\r\n* [#524] add implementation for `Math.expm1()`\r\n\r\n* [#524] add implementation for `Math.fround()`\r\n\r\n* [#524] implement `Math.hypot()`\r\n\r\n* [#524] implement `Math.log1p()`\r\n\r\n* [#524] implement `Math.imul()`\r\n\r\n* improve `Math.clz32()` implementation\r\n\r\n* [#524] add tests for more states",
          "timestamp": "2020-07-02T23:42:12+02:00",
          "tree_id": "b51563e4e7dd54e5f094cce4f88c8d5273ae964b",
          "url": "https://github.com/boa-dev/boa/commit/d9ef1fb426979e00173956d797afaa30d9168a90"
        },
        "date": 1593727023847,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 159.09,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.5615,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 25.27,
            "range": "+/- 0.736",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1592,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.945,
            "range": "+/- 0.428",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9261,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4579,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.7604,
            "range": "+/- 0.165",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2611,
            "range": "+/- 0.206",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.9849,
            "range": "+/- 0.233",
            "unit": "us"
          },
          {
            "name": "",
            "value": 82.326,
            "range": "+/- 2.306",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 83.505,
            "range": "+/- 2.073",
            "unit": "us"
          },
          {
            "name": "",
            "value": 88.435,
            "range": "+/- 2.566",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 89.61,
            "range": "+/- 2.923",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6082,
            "range": "+/- 0.192",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.4871,
            "range": "+/- 0.268",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.4651,
            "range": "+/- 0.096",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.6162,
            "range": "+/- 0.144",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7708,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.6971,
            "range": "+/- 0.205",
            "unit": "us"
          },
          {
            "name": "",
            "value": 604.79,
            "range": "+/- 11.430",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3164,
            "range": "+/- 0.065",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0401,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6821,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.356,
            "range": "+/- 0.203",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3767,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.973,
            "range": "+/- 0.559",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.9327,
            "range": "+/- 0.129",
            "unit": "ms"
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
          "id": "f957eacbaef18a70d4987e8c22ec18cfd27f1571",
          "message": "fix json.stringify symbol handling (#535)",
          "timestamp": "2020-07-02T23:43:40+02:00",
          "tree_id": "48ca4b30adab3a43d344360fb42ef3c64f1acf82",
          "url": "https://github.com/boa-dev/boa/commit/f957eacbaef18a70d4987e8c22ec18cfd27f1571"
        },
        "date": 1593727024541,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 145.68,
            "range": "+/- 4.670",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2793,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.564,
            "range": "+/- 0.569",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0829,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.107,
            "range": "+/- 0.275",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.7245,
            "range": "+/- 0.063",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.39,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3871,
            "range": "+/- 0.174",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8841,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5028,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "",
            "value": 80.124,
            "range": "+/- 1.186",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 79.672,
            "range": "+/- 1.755",
            "unit": "us"
          },
          {
            "name": "",
            "value": 84.89,
            "range": "+/- 1.562",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 81.877,
            "range": "+/- 2.397",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2768,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2944,
            "range": "+/- 0.204",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.2406,
            "range": "+/- 0.145",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2085,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3726,
            "range": "+/- 0.108",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.0131,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "",
            "value": 575.9,
            "range": "+/- 16.560",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1238,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 955.22,
            "range": "+/- 24.980",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.205,
            "range": "+/- 0.133",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.888,
            "range": "+/- 0.128",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1877,
            "range": "+/- 0.069",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.9,
            "range": "+/- 0.402",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.26,
            "range": "+/- 0.098",
            "unit": "ms"
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
          "id": "357c7d07f7d5b106281e271e48805333daeeccbd",
          "message": "Fix all `Value` operations and add unsigned shift right (#520)",
          "timestamp": "2020-07-03T00:08:09+02:00",
          "tree_id": "7fe87d0afdac8d228f6d877c36bbb666fecd975c",
          "url": "https://github.com/boa-dev/boa/commit/357c7d07f7d5b106281e271e48805333daeeccbd"
        },
        "date": 1593728491734,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 145.91,
            "range": "+/- 1.750",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.1256,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.439,
            "range": "+/- 0.394",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0127,
            "range": "+/- 0.011",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.048,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.847,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4175,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3197,
            "range": "+/- 0.072",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5135,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0925,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.405,
            "range": "+/- 1.114",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 73.591,
            "range": "+/- 0.720",
            "unit": "us"
          },
          {
            "name": "",
            "value": 78.471,
            "range": "+/- 1.198",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 78.097,
            "range": "+/- 0.991",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9156,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8841,
            "range": "+/- 0.102",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.672,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1446,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2535,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.7252,
            "range": "+/- 0.092",
            "unit": "us"
          },
          {
            "name": "",
            "value": 428.96,
            "range": "+/- 5.660",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1923,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 944.06,
            "range": "+/- 7.740",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3246,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0238,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2059,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.717,
            "range": "+/- 0.170",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1923,
            "range": "+/- 0.053",
            "unit": "ms"
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
          "id": "070b78c3573c071b2848d7d2336929355c098e5d",
          "message": "Feature `SyntaxError` (#536)",
          "timestamp": "2020-07-03T00:09:22+02:00",
          "tree_id": "fe7008078594ffa7d2cc950c1b9de9946a4feec3",
          "url": "https://github.com/boa-dev/boa/commit/070b78c3573c071b2848d7d2336929355c098e5d"
        },
        "date": 1593728620285,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 162.86,
            "range": "+/- 3.110",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.5758,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.649,
            "range": "+/- 0.536",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1176,
            "range": "+/- 0.021",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.86,
            "range": "+/- 0.307",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9383,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4713,
            "range": "+/- 0.021",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.8753,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0827,
            "range": "+/- 0.164",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8797,
            "range": "+/- 0.273",
            "unit": "us"
          },
          {
            "name": "",
            "value": 81.859,
            "range": "+/- 1.084",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 81.664,
            "range": "+/- 1.021",
            "unit": "us"
          },
          {
            "name": "",
            "value": 86.968,
            "range": "+/- 0.955",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 87.144,
            "range": "+/- 1.215",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.4982,
            "range": "+/- 0.230",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5694,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.0368,
            "range": "+/- 0.074",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5617,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.72,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.58,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "",
            "value": 434.71,
            "range": "+/- 7.200",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2919,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0236,
            "range": "+/- 0.012",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.702,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2689,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3887,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.96,
            "range": "+/- 0.343",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.6987,
            "range": "+/- 0.041",
            "unit": "ms"
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
          "id": "fcb4b8b1f1135a42a96df85465f666c7cc5e9389",
          "message": "Cleanup and added test for `String.prototype.concat` (#538)",
          "timestamp": "2020-07-03T00:46:24+02:00",
          "tree_id": "e1e1e40f241571ce03e07e085f9b841daa58eee1",
          "url": "https://github.com/boa-dev/boa/commit/fcb4b8b1f1135a42a96df85465f666c7cc5e9389"
        },
        "date": 1593730778716,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 152.59,
            "range": "+/- 2.900",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.0302,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.508,
            "range": "+/- 0.307",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 967.57,
            "range": "+/- 11.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.807,
            "range": "+/- 0.275",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.7293,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4084,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3751,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5196,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.163,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "",
            "value": 74.539,
            "range": "+/- 0.823",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 74.301,
            "range": "+/- 1.403",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.115,
            "range": "+/- 0.890",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 77.941,
            "range": "+/- 1.051",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7877,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0998,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6621,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2018,
            "range": "+/- 0.069",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3202,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.6173,
            "range": "+/- 0.142",
            "unit": "us"
          },
          {
            "name": "",
            "value": 418.45,
            "range": "+/- 6.920",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0852,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 932.89,
            "range": "+/- 22.940",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.2418,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0317,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1565,
            "range": "+/- 0.038",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.398,
            "range": "+/- 0.259",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.111,
            "range": "+/- 0.059",
            "unit": "ms"
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
          "id": "641dce135782f0ebd346afabbca588bcaa7b2bbf",
          "message": "Refactor exec/expression into exec/call and exec/new (#529)",
          "timestamp": "2020-07-03T17:14:04+02:00",
          "tree_id": "32415171b10e44b9a579a427d52a9405d1538a18",
          "url": "https://github.com/boa-dev/boa/commit/641dce135782f0ebd346afabbca588bcaa7b2bbf"
        },
        "date": 1593790111955,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 163.94,
            "range": "+/- 2.740",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.5676,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 25.677,
            "range": "+/- 0.599",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1304,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.967,
            "range": "+/- 0.313",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.8691,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4325,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 7.0407,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1446,
            "range": "+/- 0.148",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8352,
            "range": "+/- 0.154",
            "unit": "us"
          },
          {
            "name": "",
            "value": 82.932,
            "range": "+/- 1.292",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 82.736,
            "range": "+/- 1.703",
            "unit": "us"
          },
          {
            "name": "",
            "value": 87.979,
            "range": "+/- 1.939",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 89.035,
            "range": "+/- 2.578",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5707,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7867,
            "range": "+/- 0.189",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.2198,
            "range": "+/- 0.153",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5588,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8117,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.7401,
            "range": "+/- 0.213",
            "unit": "us"
          },
          {
            "name": "",
            "value": 436.12,
            "range": "+/- 7.820",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2783,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0223,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7664,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2624,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4158,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.694,
            "range": "+/- 0.307",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.7613,
            "range": "+/- 0.077",
            "unit": "ms"
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
          "id": "5f7ec6230684602f6826fadccb83bb2a3b105056",
          "message": "Made all `Math` methods spec compliant (#541)",
          "timestamp": "2020-07-04T17:17:34+02:00",
          "tree_id": "c1eda0b407f96a728341661559316415c52d38b3",
          "url": "https://github.com/boa-dev/boa/commit/5f7ec6230684602f6826fadccb83bb2a3b105056"
        },
        "date": 1593876726280,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 160.91,
            "range": "+/- 2.810",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.4993,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.096,
            "range": "+/- 0.371",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.146,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.911,
            "range": "+/- 0.306",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9408,
            "range": "+/- 0.078",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4732,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.942,
            "range": "+/- 0.113",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1043,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.9733,
            "range": "+/- 0.191",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.268,
            "range": "+/- 1.573",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 83.503,
            "range": "+/- 1.990",
            "unit": "us"
          },
          {
            "name": "",
            "value": 91.098,
            "range": "+/- 2.364",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 88.071,
            "range": "+/- 1.869",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2607,
            "range": "+/- 0.131",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.4912,
            "range": "+/- 0.143",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.1972,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.4215,
            "range": "+/- 0.108",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5892,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.431,
            "range": "+/- 0.168",
            "unit": "us"
          },
          {
            "name": "",
            "value": 414.84,
            "range": "+/- 8.160",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3728,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0245,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.8531,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2798,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4736,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.501,
            "range": "+/- 0.270",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 7.2139,
            "range": "+/- 0.068",
            "unit": "ms"
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
          "id": "ca0eaeadb286b5869fe7ae090e0fa3c47b980c13",
          "message": "Removed `console`s dependency of `InternalState` (#544)",
          "timestamp": "2020-07-07T16:17:13+02:00",
          "tree_id": "abd8db4817122f7a2d69964cf82470adbf42f025",
          "url": "https://github.com/boa-dev/boa/commit/ca0eaeadb286b5869fe7ae090e0fa3c47b980c13"
        },
        "date": 1594132392561,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 164.99,
            "range": "+/- 1.730",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.5825,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 25.033,
            "range": "+/- 0.345",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1511,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.093,
            "range": "+/- 0.286",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0976,
            "range": "+/- 0.043",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5594,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.9952,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2481,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.1084,
            "range": "+/- 0.209",
            "unit": "us"
          },
          {
            "name": "",
            "value": 84.205,
            "range": "+/- 1.452",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 84.399,
            "range": "+/- 1.583",
            "unit": "us"
          },
          {
            "name": "",
            "value": 88.098,
            "range": "+/- 1.233",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 89.109,
            "range": "+/- 1.989",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8353,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8609,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.2664,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.6031,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8185,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.7315,
            "range": "+/- 0.146",
            "unit": "us"
          },
          {
            "name": "",
            "value": 441.1,
            "range": "+/- 6.720",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3072,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 1.0434,
            "range": "+/- 0.018",
            "unit": "us"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.8235,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3347,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4674,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 15.204,
            "range": "+/- 0.347",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.8859,
            "range": "+/- 0.060",
            "unit": "ms"
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
          "id": "1e82e7c95a03b6539e9fc4d8637651df1f8e88e1",
          "message": "Merged `create` into `init` for builtins (#547)",
          "timestamp": "2020-07-07T18:21:18+02:00",
          "tree_id": "76fb08b0901daae2f998074ee7c2b4df166296f4",
          "url": "https://github.com/boa-dev/boa/commit/1e82e7c95a03b6539e9fc4d8637651df1f8e88e1"
        },
        "date": 1594139599339,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 137.67,
            "range": "+/- 9.060",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.55,
            "range": "+/- 0.109",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.76,
            "range": "+/- 0.440",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 940.69,
            "range": "+/- 59.430",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.523,
            "range": "+/- 0.289",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.0338,
            "range": "+/- 0.061",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.2621,
            "range": "+/- 0.068",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.4443,
            "range": "+/- 0.245",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5857,
            "range": "+/- 0.157",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9392,
            "range": "+/- 0.150",
            "unit": "us"
          },
          {
            "name": "",
            "value": 64.116,
            "range": "+/- 1.972",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 63.001,
            "range": "+/- 1.463",
            "unit": "us"
          },
          {
            "name": "",
            "value": 66.473,
            "range": "+/- 1.068",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 67.734,
            "range": "+/- 1.538",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.9923,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8655,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1538,
            "range": "+/- 0.168",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0881,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.0108,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.7254,
            "range": "+/- 0.275",
            "unit": "us"
          },
          {
            "name": "",
            "value": 359.75,
            "range": "+/- 14.610",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7547,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 764.9,
            "range": "+/- 16.890",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.295,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 3.8999,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.7907,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 11.643,
            "range": "+/- 0.518",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.1682,
            "range": "+/- 0.101",
            "unit": "ms"
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
          "id": "0fe591b5c560e3e801fb3eebde5c163edace4488",
          "message": "[builtins - object] Object.create (#543)",
          "timestamp": "2020-07-08T10:04:27+02:00",
          "tree_id": "97d4580514f602fb5e235b25cccd70087916a151",
          "url": "https://github.com/boa-dev/boa/commit/0fe591b5c560e3e801fb3eebde5c163edace4488"
        },
        "date": 1594196290587,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 157.54,
            "range": "+/- 4.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2638,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.506,
            "range": "+/- 0.359",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.098,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.303,
            "range": "+/- 0.208",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6659,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3761,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.5744,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7325,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.4481,
            "range": "+/- 0.073",
            "unit": "us"
          },
          {
            "name": "",
            "value": 78.687,
            "range": "+/- 0.900",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 80.035,
            "range": "+/- 1.458",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.071,
            "range": "+/- 1.191",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 81.952,
            "range": "+/- 0.921",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.407,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.6793,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.2493,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.3408,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4867,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.2157,
            "range": "+/- 0.084",
            "unit": "us"
          },
          {
            "name": "",
            "value": 409.6,
            "range": "+/- 3.080",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.139,
            "range": "+/- 0.021",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 964.45,
            "range": "+/- 7.810",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.3772,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9473,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2751,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.954,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.2879,
            "range": "+/- 0.027",
            "unit": "ms"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "30962174+IovoslavIovchev@users.noreply.github.com",
            "name": "Iovoslav Iovchev",
            "username": "IovoslavIovchev"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a933ae8ef3e0926c5bb00f65dd6b81bf57aa096d",
          "message": "rustyline for the cli (#492)\n\nCo-authored-by: Iban Eguia <razican@protonmail.ch>",
          "timestamp": "2020-07-08T12:19:46+02:00",
          "tree_id": "9c4f06512a32099b4a7e40302c105b6e54fc3e60",
          "url": "https://github.com/boa-dev/boa/commit/a933ae8ef3e0926c5bb00f65dd6b81bf57aa096d"
        },
        "date": 1594204337119,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 132.97,
            "range": "+/- 4.290",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7132,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 20.071,
            "range": "+/- 0.775",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 892.97,
            "range": "+/- 27.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.55,
            "range": "+/- 0.341",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.3034,
            "range": "+/- 0.073",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.2551,
            "range": "+/- 0.048",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.7313,
            "range": "+/- 0.249",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8076,
            "range": "+/- 0.202",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6198,
            "range": "+/- 0.156",
            "unit": "us"
          },
          {
            "name": "",
            "value": 62.557,
            "range": "+/- 2.052",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 63.254,
            "range": "+/- 2.216",
            "unit": "us"
          },
          {
            "name": "",
            "value": 67.147,
            "range": "+/- 3.055",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 72.835,
            "range": "+/- 3.909",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4991,
            "range": "+/- 0.230",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.19,
            "range": "+/- 0.274",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1138,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5666,
            "range": "+/- 0.126",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2713,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.8059,
            "range": "+/- 0.219",
            "unit": "us"
          },
          {
            "name": "",
            "value": 410.08,
            "range": "+/- 13.600",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.941,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 841.7,
            "range": "+/- 25.490",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.529,
            "range": "+/- 0.134",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.6086,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1068,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.46,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.6542,
            "range": "+/- 0.098",
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
          "id": "84db01ee09dc48632019be1bd49967caa267f1e5",
          "message": "Improved the description of the issue templates (#554)",
          "timestamp": "2020-07-08T21:23:54+02:00",
          "tree_id": "37a546e93aa359adffe9447e18a9f8630d069c5e",
          "url": "https://github.com/boa-dev/boa/commit/84db01ee09dc48632019be1bd49967caa267f1e5"
        },
        "date": 1594237054813,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 158.14,
            "range": "+/- 3.360",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.3108,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.435,
            "range": "+/- 0.195",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0463,
            "range": "+/- 0.011",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.343,
            "range": "+/- 0.205",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9364,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4732,
            "range": "+/- 0.013",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.4845,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7356,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.3992,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 77.974,
            "range": "+/- 1.351",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.516,
            "range": "+/- 1.036",
            "unit": "us"
          },
          {
            "name": "",
            "value": 78.408,
            "range": "+/- 0.800",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 80.343,
            "range": "+/- 1.186",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.877,
            "range": "+/- 0.033",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0548,
            "range": "+/- 0.046",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.7656,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2926,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4862,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "",
            "value": 9.0283,
            "range": "+/- 0.063",
            "unit": "us"
          },
          {
            "name": "",
            "value": 451.68,
            "range": "+/- 3.080",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2424,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 966.62,
            "range": "+/- 10.540",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.468,
            "range": "+/- 0.037",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.1996,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2749,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.028,
            "range": "+/- 0.137",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.3241,
            "range": "+/- 0.033",
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
          "id": "8b3d52b5f2afde4f37b4658368c762f7c0d7e9de",
          "message": "Added benchmark for goal symbol switching (#556)",
          "timestamp": "2020-07-08T22:55:03+02:00",
          "tree_id": "f26f4dd5294d004037617280882ba318610dc438",
          "url": "https://github.com/boa-dev/boa/commit/8b3d52b5f2afde4f37b4658368c762f7c0d7e9de"
        },
        "date": 1594242556743,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 154.76,
            "range": "+/- 3.610",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2289,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.154,
            "range": "+/- 0.497",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0068,
            "range": "+/- 0.014",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.095,
            "range": "+/- 0.256",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9049,
            "range": "+/- 0.050",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4527,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.2755,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7065,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.158,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.802,
            "range": "+/- 2.362",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 72.124,
            "range": "+/- 1.280",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.983,
            "range": "+/- 1.586",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 75.449,
            "range": "+/- 1.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8025,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8573,
            "range": "+/- 0.091",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6645,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1979,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2591,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.736,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "",
            "value": 441.68,
            "range": "+/- 7.920",
            "unit": "ns"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.081,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 922.01,
            "range": "+/- 21.450",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1872,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.0194,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2102,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.656,
            "range": "+/- 0.233",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.0367,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.5461,
            "range": "+/- 0.105",
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
          "id": "6a721f94ca20c49fc8cd0f7f734dd36919138e5f",
          "message": "Added benchmarks for full program execution (#560)",
          "timestamp": "2020-07-10T17:03:35+02:00",
          "tree_id": "7d0e2a9565902e8a0f00a8fed54c93ff70b86dcf",
          "url": "https://github.com/boa-dev/boa/commit/6a721f94ca20c49fc8cd0f7f734dd36919138e5f"
        },
        "date": 1594394510508,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 155.32,
            "range": "+/- 2.780",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2383,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.034,
            "range": "+/- 0.390",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0019,
            "range": "+/- -995.222",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 13.872,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.8442,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4346,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.3407,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6436,
            "range": "+/- 0.191",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.3812,
            "range": "+/- 0.347",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.62,
            "range": "+/- 1.170",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 71.827,
            "range": "+/- 1.236",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.9,
            "range": "+/- 1.507",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 76.565,
            "range": "+/- 0.980",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7334,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9511,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6815,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.24,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5243,
            "range": "+/- 0.419",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.9002,
            "range": "+/- 0.154",
            "unit": "us"
          },
          {
            "name": "",
            "value": 427.97,
            "range": "+/- 7.610",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 167.68,
            "range": "+/- 5.520",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 193.48,
            "range": "+/- 4.350",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.2094,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 189.47,
            "range": "+/- 3.330",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.1426,
            "range": "+/- 0.054",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.7811,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 195.98,
            "range": "+/- 2.930",
            "unit": "us"
          },
          {
            "name": "",
            "value": 177.97,
            "range": "+/- 3.400",
            "unit": "us"
          },
          {
            "name": "",
            "value": 178.15,
            "range": "+/- 4.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 308.02,
            "range": "+/- 10.900",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 300.72,
            "range": "+/- 4.590",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 261.4,
            "range": "+/- 5.000",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 257.67,
            "range": "+/- 5.300",
            "unit": "us"
          },
          {
            "name": "",
            "value": 171.66,
            "range": "+/- 2.380",
            "unit": "us"
          },
          {
            "name": "",
            "value": 177.45,
            "range": "+/- 4.500",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 167.62,
            "range": "+/- 3.490",
            "unit": "us"
          },
          {
            "name": "",
            "value": 213.71,
            "range": "+/- 3.510",
            "unit": "us"
          },
          {
            "name": "",
            "value": 176.7,
            "range": "+/- 3.340",
            "unit": "us"
          },
          {
            "name": "",
            "value": 178.14,
            "range": "+/- 3.050",
            "unit": "us"
          },
          {
            "name": "",
            "value": 167.47,
            "range": "+/- 3.940",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.0943,
            "range": "+/- 0.024",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 940.25,
            "range": "+/- 15.170",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.4929,
            "range": "+/- 0.233",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2805,
            "range": "+/- 0.238",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2305,
            "range": "+/- 0.028",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.952,
            "range": "+/- 0.264",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.3274,
            "range": "+/- 0.127",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.7718,
            "range": "+/- 0.139",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshwd36@users.noreply.github.com",
            "name": "joshwd36",
            "username": "joshwd36"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ffe8b5f337537a2c736b99cc5c4e9db775a2cb02",
          "message": "Throw a type error when a non-object is called (#561)",
          "timestamp": "2020-07-12T01:15:15+02:00",
          "tree_id": "95629c06ec736bc77d24b375d5d232b66209644d",
          "url": "https://github.com/boa-dev/boa/commit/ffe8b5f337537a2c736b99cc5c4e9db775a2cb02"
        },
        "date": 1594510346456,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 130.5,
            "range": "+/- 3.630",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.6692,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 19.693,
            "range": "+/- 0.493",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 934.17,
            "range": "+/- 15.360",
            "unit": "us"
          },
          {
            "name": "",
            "value": 12.048,
            "range": "+/- 0.361",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.1544,
            "range": "+/- 0.073",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.1719,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.5061,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7194,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.4252,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "",
            "value": 68.514,
            "range": "+/- 2.020",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 66.679,
            "range": "+/- 1.511",
            "unit": "us"
          },
          {
            "name": "",
            "value": 71.345,
            "range": "+/- 1.833",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 71.736,
            "range": "+/- 2.205",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2275,
            "range": "+/- 0.154",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.1867,
            "range": "+/- 0.176",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1385,
            "range": "+/- 0.093",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.871,
            "range": "+/- 0.152",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.6187,
            "range": "+/- 0.136",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.6408,
            "range": "+/- 0.181",
            "unit": "us"
          },
          {
            "name": "",
            "value": 343.79,
            "range": "+/- 7.810",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 143.92,
            "range": "+/- 4.180",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 171.35,
            "range": "+/- 3.880",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.0928,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 172.2,
            "range": "+/- 6.650",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.4622,
            "range": "+/- 0.055",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.4469,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 176.83,
            "range": "+/- 5.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 158.7,
            "range": "+/- 5.290",
            "unit": "us"
          },
          {
            "name": "",
            "value": 161.64,
            "range": "+/- 3.590",
            "unit": "us"
          },
          {
            "name": "",
            "value": 267.63,
            "range": "+/- 6.730",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 270.33,
            "range": "+/- 7.580",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 241.69,
            "range": "+/- 9.050",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 235.03,
            "range": "+/- 4.980",
            "unit": "us"
          },
          {
            "name": "",
            "value": 155.32,
            "range": "+/- 3.730",
            "unit": "us"
          },
          {
            "name": "",
            "value": 157.22,
            "range": "+/- 3.760",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 153.12,
            "range": "+/- 4.070",
            "unit": "us"
          },
          {
            "name": "",
            "value": 189.47,
            "range": "+/- 4.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 160.91,
            "range": "+/- 4.860",
            "unit": "us"
          },
          {
            "name": "",
            "value": 159.5,
            "range": "+/- 2.630",
            "unit": "us"
          },
          {
            "name": "",
            "value": 148.32,
            "range": "+/- 3.770",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.7744,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 816.77,
            "range": "+/- 23.380",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5309,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2431,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0054,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.459,
            "range": "+/- 0.361",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.4484,
            "range": "+/- 0.073",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 7.514,
            "range": "+/- 0.224",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshwd36@users.noreply.github.com",
            "name": "joshwd36",
            "username": "joshwd36"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "df80ee005ba7dea1f9f6ab12d64e0de5282d60b4",
          "message": "Ensure tests test error type (#563)",
          "timestamp": "2020-07-13T11:10:43+02:00",
          "tree_id": "5d770b95177407118377ddc247784dcfb750ff71",
          "url": "https://github.com/boa-dev/boa/commit/df80ee005ba7dea1f9f6ab12d64e0de5282d60b4"
        },
        "date": 1594632463270,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 133.14,
            "range": "+/- 5.090",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.5641,
            "range": "+/- 0.076",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.111,
            "range": "+/- 0.621",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 866.62,
            "range": "+/- 71.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.754,
            "range": "+/- 0.401",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5945,
            "range": "+/- 0.128",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3324,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 4.926,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7779,
            "range": "+/- 0.223",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6072,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 63.562,
            "range": "+/- 2.124",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 61.95,
            "range": "+/- 2.902",
            "unit": "us"
          },
          {
            "name": "",
            "value": 64.854,
            "range": "+/- 2.293",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 60.988,
            "range": "+/- 1.379",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4535,
            "range": "+/- 0.381",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.636,
            "range": "+/- 0.373",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.3393,
            "range": "+/- 0.304",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.3089,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.233,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7682,
            "range": "+/- 0.399",
            "unit": "us"
          },
          {
            "name": "",
            "value": 385.88,
            "range": "+/- 20.460",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 126.64,
            "range": "+/- 3.560",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 183.82,
            "range": "+/- 7.070",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 930.07,
            "range": "+/- 21.990",
            "unit": "us"
          },
          {
            "name": "Array access (Full)",
            "value": 150.84,
            "range": "+/- 3.480",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.5821,
            "range": "+/- 0.209",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.4235,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 157.1,
            "range": "+/- 5.300",
            "unit": "us"
          },
          {
            "name": "",
            "value": 163.01,
            "range": "+/- 10.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 144.48,
            "range": "+/- 8.250",
            "unit": "us"
          },
          {
            "name": "",
            "value": 276.89,
            "range": "+/- 7.930",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 302.76,
            "range": "+/- 6.720",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 234.16,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 251.38,
            "range": "+/- 9.490",
            "unit": "us"
          },
          {
            "name": "",
            "value": 168.1,
            "range": "+/- 5.540",
            "unit": "us"
          },
          {
            "name": "",
            "value": 149.13,
            "range": "+/- 4.680",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 163.28,
            "range": "+/- 4.390",
            "unit": "us"
          },
          {
            "name": "",
            "value": 165.91,
            "range": "+/- 5.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 175.91,
            "range": "+/- 3.740",
            "unit": "us"
          },
          {
            "name": "",
            "value": 156.28,
            "range": "+/- 11.770",
            "unit": "us"
          },
          {
            "name": "",
            "value": 136.69,
            "range": "+/- 5.920",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.8867,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 904.52,
            "range": "+/- 27.830",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.5788,
            "range": "+/- 0.229",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.2554,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.7644,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 10.61,
            "range": "+/- 0.339",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.0945,
            "range": "+/- 0.103",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.2293,
            "range": "+/- 0.269",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshwd36@users.noreply.github.com",
            "name": "joshwd36",
            "username": "joshwd36"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "690b194b43bba9ebe7f72bfced5d03221cc238df",
          "message": "Add missing Number methods. (#562)",
          "timestamp": "2020-07-13T11:11:48+02:00",
          "tree_id": "ddeb3bf42226131201a1acf764d90ab02ab483fa",
          "url": "https://github.com/boa-dev/boa/commit/690b194b43bba9ebe7f72bfced5d03221cc238df"
        },
        "date": 1594632544701,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 138.25,
            "range": "+/- 2.430",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7336,
            "range": "+/- 0.087",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 20.567,
            "range": "+/- 0.461",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 904.09,
            "range": "+/- 26.010",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.12,
            "range": "+/- 0.438",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5842,
            "range": "+/- 0.063",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3182,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.816,
            "range": "+/- 0.169",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.7966,
            "range": "+/- 0.131",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.3897,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "",
            "value": 67.799,
            "range": "+/- 2.031",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 66.41,
            "range": "+/- 1.409",
            "unit": "us"
          },
          {
            "name": "",
            "value": 70.784,
            "range": "+/- 1.576",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 70.769,
            "range": "+/- 1.632",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2958,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2538,
            "range": "+/- 0.116",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.2838,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6775,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1114,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.9893,
            "range": "+/- 0.196",
            "unit": "us"
          },
          {
            "name": "",
            "value": 373.98,
            "range": "+/- 7.350",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 150.13,
            "range": "+/- 7.120",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 173.77,
            "range": "+/- 4.510",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.059,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 168.21,
            "range": "+/- 2.640",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.759,
            "range": "+/- 0.055",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.6204,
            "range": "+/- 0.034",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 178.08,
            "range": "+/- 4.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 160.4,
            "range": "+/- 3.820",
            "unit": "us"
          },
          {
            "name": "",
            "value": 164.24,
            "range": "+/- 3.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 273.35,
            "range": "+/- 6.380",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 278.38,
            "range": "+/- 6.650",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 233.31,
            "range": "+/- 5.440",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 233.94,
            "range": "+/- 4.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 157.77,
            "range": "+/- 4.060",
            "unit": "us"
          },
          {
            "name": "",
            "value": 160.24,
            "range": "+/- 3.570",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 154.23,
            "range": "+/- 4.260",
            "unit": "us"
          },
          {
            "name": "",
            "value": 199.96,
            "range": "+/- 5.400",
            "unit": "us"
          },
          {
            "name": "",
            "value": 164.13,
            "range": "+/- 4.330",
            "unit": "us"
          },
          {
            "name": "",
            "value": 168.51,
            "range": "+/- 3.930",
            "unit": "us"
          },
          {
            "name": "",
            "value": 150.24,
            "range": "+/- 3.240",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.88,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 866.71,
            "range": "+/- 19.920",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.7434,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5731,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0075,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.233,
            "range": "+/- 0.282",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.5586,
            "range": "+/- 0.085",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 7.7139,
            "range": "+/- 0.227",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "brf2117@columbia.edu",
            "name": "benjaminflin",
            "username": "benjaminflin"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "3e2e56641efa804a5bbf1005f8d89ba0e8ce1dbe",
          "message": "Implement Array.prototype.reduce (#555)\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-07-15T09:03:32+02:00",
          "tree_id": "befb56b6cd627eefd8d18bedbe60d2e35ffa0bf0",
          "url": "https://github.com/boa-dev/boa/commit/3e2e56641efa804a5bbf1005f8d89ba0e8ce1dbe"
        },
        "date": 1594797687555,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 159.51,
            "range": "+/- 1.500",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 4.2237,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.682,
            "range": "+/- 0.151",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0299,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.737,
            "range": "+/- 0.212",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9657,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4928,
            "range": "+/- 0.012",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.5658,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7617,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.5305,
            "range": "+/- 0.127",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.734,
            "range": "+/- 0.480",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.767,
            "range": "+/- 0.679",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.436,
            "range": "+/- 0.786",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 79.073,
            "range": "+/- 0.992",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9872,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1141,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.8447,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.2765,
            "range": "+/- 0.056",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.2798,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.907,
            "range": "+/- 0.057",
            "unit": "us"
          },
          {
            "name": "",
            "value": 431.11,
            "range": "+/- 2.970",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 171.24,
            "range": "+/- 1.680",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 202.37,
            "range": "+/- 1.660",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.2504,
            "range": "+/- 0.011",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 199.71,
            "range": "+/- 3.980",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.4095,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.8348,
            "range": "+/- 0.014",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 208.58,
            "range": "+/- 1.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 183.49,
            "range": "+/- 1.780",
            "unit": "us"
          },
          {
            "name": "",
            "value": 188.39,
            "range": "+/- 2.300",
            "unit": "us"
          },
          {
            "name": "",
            "value": 315.87,
            "range": "+/- 2.150",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 314.86,
            "range": "+/- 1.700",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 278.49,
            "range": "+/- 5.120",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 276.89,
            "range": "+/- 4.720",
            "unit": "us"
          },
          {
            "name": "",
            "value": 185.79,
            "range": "+/- 3.860",
            "unit": "us"
          },
          {
            "name": "",
            "value": 188.81,
            "range": "+/- 3.780",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 178.58,
            "range": "+/- 1.310",
            "unit": "us"
          },
          {
            "name": "",
            "value": 230.63,
            "range": "+/- 2.380",
            "unit": "us"
          },
          {
            "name": "",
            "value": 185.2,
            "range": "+/- 1.510",
            "unit": "us"
          },
          {
            "name": "",
            "value": 190.25,
            "range": "+/- 2.000",
            "unit": "us"
          },
          {
            "name": "",
            "value": 178.4,
            "range": "+/- 2.510",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1479,
            "range": "+/- 0.016",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 974.6,
            "range": "+/- 6.480",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.452,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.3788,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.2795,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.961,
            "range": "+/- 0.147",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.2336,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.7306,
            "range": "+/- 0.055",
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
          "id": "08a608a821562371ed56e9b03b274c5cac6588de",
          "message": "Spec Compliant `Number.prototype.toString()`, better `Number` object formating and `-0` (#572)\n\n* `Number` object formating and `-0`\r\n\r\n* Made `Number.prototype.toString()` spec compliant\r\n\r\n* Enabled ignore `toString()` tests",
          "timestamp": "2020-07-19T22:19:45+02:00",
          "tree_id": "adf5b7eebe31c06137f20a7abf395c57517abdf4",
          "url": "https://github.com/boa-dev/boa/commit/08a608a821562371ed56e9b03b274c5cac6588de"
        },
        "date": 1595191124622,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 154.81,
            "range": "+/- 2.690",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.9901,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.155,
            "range": "+/- 0.555",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 958.96,
            "range": "+/- 17.910",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.789,
            "range": "+/- 0.199",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6988,
            "range": "+/- 0.047",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4259,
            "range": "+/- 0.021",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.177,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.3007,
            "range": "+/- 0.109",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8219,
            "range": "+/- 0.136",
            "unit": "us"
          },
          {
            "name": "",
            "value": 62.874,
            "range": "+/- 1.048",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 63.971,
            "range": "+/- 1.457",
            "unit": "us"
          },
          {
            "name": "",
            "value": 67.482,
            "range": "+/- 1.004",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 69.483,
            "range": "+/- 1.534",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8068,
            "range": "+/- 0.144",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.8269,
            "range": "+/- 0.161",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6412,
            "range": "+/- 0.059",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0721,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1032,
            "range": "+/- 0.079",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.7871,
            "range": "+/- 0.171",
            "unit": "us"
          },
          {
            "name": "",
            "value": 371.7,
            "range": "+/- 4.540",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 172.6,
            "range": "+/- 3.210",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 196.46,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.1879,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 190.01,
            "range": "+/- 4.330",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.9918,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.7226,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 198.9,
            "range": "+/- 3.870",
            "unit": "us"
          },
          {
            "name": "",
            "value": 187.31,
            "range": "+/- 2.960",
            "unit": "us"
          },
          {
            "name": "",
            "value": 187.94,
            "range": "+/- 2.110",
            "unit": "us"
          },
          {
            "name": "",
            "value": 294.95,
            "range": "+/- 4.920",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 298.29,
            "range": "+/- 3.270",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 255.38,
            "range": "+/- 4.400",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 262.22,
            "range": "+/- 3.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 184.19,
            "range": "+/- 2.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 189.03,
            "range": "+/- 2.530",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 179.07,
            "range": "+/- 2.700",
            "unit": "us"
          },
          {
            "name": "",
            "value": 185.31,
            "range": "+/- 3.720",
            "unit": "us"
          },
          {
            "name": "",
            "value": 181.82,
            "range": "+/- 2.440",
            "unit": "us"
          },
          {
            "name": "",
            "value": 191.42,
            "range": "+/- 2.930",
            "unit": "us"
          },
          {
            "name": "",
            "value": 173.55,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.2933,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 790.4,
            "range": "+/- 15.190",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9828,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.226,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.0517,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.792,
            "range": "+/- 0.194",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.0422,
            "range": "+/- 0.060",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.1485,
            "range": "+/- 0.157",
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
          "id": "d8eb7caefda3af1d2987cbba374d7f12163ecd19",
          "message": "Extracted `__proto__` from internal slots (#580)",
          "timestamp": "2020-07-20T23:25:03+02:00",
          "tree_id": "c80daec4a42d01769cb4eb4b95f301eec3b822b6",
          "url": "https://github.com/boa-dev/boa/commit/d8eb7caefda3af1d2987cbba374d7f12163ecd19"
        },
        "date": 1595281461241,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 149.41,
            "range": "+/- 2.900",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7431,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 21.867,
            "range": "+/- 0.321",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 970.07,
            "range": "+/- 23.180",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.916,
            "range": "+/- 0.279",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5759,
            "range": "+/- 0.043",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3926,
            "range": "+/- 0.019",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.0425,
            "range": "+/- 0.155",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0117,
            "range": "+/- 0.086",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.2693,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "",
            "value": 66.226,
            "range": "+/- 1.555",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 67.373,
            "range": "+/- 0.987",
            "unit": "us"
          },
          {
            "name": "",
            "value": 71.639,
            "range": "+/- 0.979",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 70.819,
            "range": "+/- 1.567",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8062,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7758,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.4979,
            "range": "+/- 0.043",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.4281,
            "range": "+/- 0.039",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.386,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.3393,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 373.58,
            "range": "+/- 3.550",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 163.04,
            "range": "+/- 1.740",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 189.79,
            "range": "+/- 0.950",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.2107,
            "range": "+/- 0.024",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 188.94,
            "range": "+/- 2.370",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.1832,
            "range": "+/- 0.038",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.8397,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 195.2,
            "range": "+/- 2.490",
            "unit": "us"
          },
          {
            "name": "",
            "value": 173.95,
            "range": "+/- 2.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 170.76,
            "range": "+/- 1.980",
            "unit": "us"
          },
          {
            "name": "",
            "value": 285.23,
            "range": "+/- 4.330",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 284.47,
            "range": "+/- 5.330",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 246.33,
            "range": "+/- 2.870",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 246.28,
            "range": "+/- 3.760",
            "unit": "us"
          },
          {
            "name": "",
            "value": 169.48,
            "range": "+/- 2.150",
            "unit": "us"
          },
          {
            "name": "",
            "value": 178.48,
            "range": "+/- 2.330",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 167.7,
            "range": "+/- 2.860",
            "unit": "us"
          },
          {
            "name": "",
            "value": 168.04,
            "range": "+/- 1.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 171.36,
            "range": "+/- 2.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 179.57,
            "range": "+/- 1.960",
            "unit": "us"
          },
          {
            "name": "",
            "value": 163.78,
            "range": "+/- 1.710",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3533,
            "range": "+/- 0.025",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 800.12,
            "range": "+/- 13.110",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.0954,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.2217,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1583,
            "range": "+/- 0.029",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.809,
            "range": "+/- 0.242",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1388,
            "range": "+/- 0.040",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.4152,
            "range": "+/- 0.142",
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
          "id": "5d4d8fe79446e7b18bd9334995706eba537333f4",
          "message": "Refactor Property Descriptor flags (#553)",
          "timestamp": "2020-07-21T02:26:53+02:00",
          "tree_id": "23674be2a477f50ba1e8fd86af9bb342f89bd25b",
          "url": "https://github.com/boa-dev/boa/commit/5d4d8fe79446e7b18bd9334995706eba537333f4"
        },
        "date": 1595292417359,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 164.86,
            "range": "+/- 2.150",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.9808,
            "range": "+/- 0.061",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 24.138,
            "range": "+/- 0.512",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.0387,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 15.108,
            "range": "+/- 0.209",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0191,
            "range": "+/- 0.033",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5488,
            "range": "+/- 0.010",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 6.2322,
            "range": "+/- 0.089",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.3953,
            "range": "+/- 0.106",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.305,
            "range": "+/- 0.119",
            "unit": "us"
          },
          {
            "name": "",
            "value": 72.256,
            "range": "+/- 1.410",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 71.967,
            "range": "+/- 1.524",
            "unit": "us"
          },
          {
            "name": "",
            "value": 75.539,
            "range": "+/- 0.983",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 75.868,
            "range": "+/- 1.191",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0348,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.1048,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.5914,
            "range": "+/- 0.058",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5379,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5237,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.4487,
            "range": "+/- 0.098",
            "unit": "us"
          },
          {
            "name": "",
            "value": 395.69,
            "range": "+/- 5.890",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 176.28,
            "range": "+/- 3.630",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 206.01,
            "range": "+/- 3.020",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.2664,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 201.1,
            "range": "+/- 4.120",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.4392,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.9444,
            "range": "+/- 0.030",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 207.78,
            "range": "+/- 2.070",
            "unit": "us"
          },
          {
            "name": "",
            "value": 185.49,
            "range": "+/- 2.020",
            "unit": "us"
          },
          {
            "name": "",
            "value": 186.42,
            "range": "+/- 2.070",
            "unit": "us"
          },
          {
            "name": "",
            "value": 311.88,
            "range": "+/- 6.260",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 314.56,
            "range": "+/- 4.600",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 271.48,
            "range": "+/- 4.420",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 269.5,
            "range": "+/- 1.650",
            "unit": "us"
          },
          {
            "name": "",
            "value": 182.69,
            "range": "+/- 1.880",
            "unit": "us"
          },
          {
            "name": "",
            "value": 189.7,
            "range": "+/- 3.230",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 179.33,
            "range": "+/- 2.650",
            "unit": "us"
          },
          {
            "name": "",
            "value": 182.5,
            "range": "+/- 2.340",
            "unit": "us"
          },
          {
            "name": "",
            "value": 187.45,
            "range": "+/- 2.890",
            "unit": "us"
          },
          {
            "name": "",
            "value": 192.01,
            "range": "+/- 4.020",
            "unit": "us"
          },
          {
            "name": "",
            "value": 175.08,
            "range": "+/- 2.410",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.4707,
            "range": "+/- 0.026",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 875.6,
            "range": "+/- 13.390",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6331,
            "range": "+/- 0.078",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.8025,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3213,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.91,
            "range": "+/- 0.266",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.7374,
            "range": "+/- 0.052",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 9.0759,
            "range": "+/- 0.188",
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
          "id": "cf253054d977dbf55bde41385404d66c0a84f588",
          "message": "Fix string prototype `trim` methods (#583)\n\n* Made trim methods ECMAScript specification compliant\n\n* Added tests",
          "timestamp": "2020-07-21T13:21:51+02:00",
          "tree_id": "f67ca84d999528e677200c5fb2c1bf97e8a4a3e7",
          "url": "https://github.com/boa-dev/boa/commit/cf253054d977dbf55bde41385404d66c0a84f588"
        },
        "date": 1595331569085,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 127.63,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 2.9913,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 18.73,
            "range": "+/- 0.076",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 810.22,
            "range": "+/- 0.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 11.407,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.1358,
            "range": "+/- 0.003",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.2088,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 4.6614,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.7778,
            "range": "+/- 0.003",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4329,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 56.092,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 55.693,
            "range": "+/- 0.027",
            "unit": "us"
          },
          {
            "name": "",
            "value": 59.023,
            "range": "+/- 0.055",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 59.069,
            "range": "+/- 0.042",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.4748,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3766,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 3.4785,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 2.7917,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.5024,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6105,
            "range": "+/- 0.002",
            "unit": "us"
          },
          {
            "name": "",
            "value": 309.3,
            "range": "+/- 0.030",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 136.12,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 159.89,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 988.16,
            "range": "+/- 0.300",
            "unit": "us"
          },
          {
            "name": "Array access (Full)",
            "value": 155.56,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.4497,
            "range": "+/- 0.001",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.5156,
            "range": "+/- 0.000",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 162.87,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "",
            "value": 144.09,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 147.91,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 243.69,
            "range": "+/- 0.250",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 246.52,
            "range": "+/- 0.070",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 209.19,
            "range": "+/- 0.110",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 212.57,
            "range": "+/- 0.090",
            "unit": "us"
          },
          {
            "name": "",
            "value": 143.95,
            "range": "+/- 0.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 149.76,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 142.42,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "",
            "value": 142.91,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 143.61,
            "range": "+/- 0.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 147.72,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "",
            "value": 137.66,
            "range": "+/- 0.040",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9229,
            "range": "+/- 0.000",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 672.2,
            "range": "+/- 0.090",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.3841,
            "range": "+/- 0.004",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5424,
            "range": "+/- 0.001",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.881,
            "range": "+/- 0.005",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.349,
            "range": "+/- 0.006",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.4919,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 7.49,
            "range": "+/- 0.003",
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
          "id": "bbed0321999a349dae27088c5cb4b7b48eb58076",
          "message": "Make `String.prototype.repeat()` ECMAScript specification compliant (#582)",
          "timestamp": "2020-07-22T00:30:49+02:00",
          "tree_id": "991e8802bbea37f7f0c56cef756b5f30ef82f54f",
          "url": "https://github.com/boa-dev/boa/commit/bbed0321999a349dae27088c5cb4b7b48eb58076"
        },
        "date": 1595371753721,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 155.61,
            "range": "+/- 3.660",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.7244,
            "range": "+/- 0.060",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.499,
            "range": "+/- 0.555",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.096,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 14.789,
            "range": "+/- 0.310",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.718,
            "range": "+/- 0.039",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.5125,
            "range": "+/- 0.035",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.9958,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2531,
            "range": "+/- 0.139",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.0386,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 73.787,
            "range": "+/- 1.113",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 75.747,
            "range": "+/- 1.781",
            "unit": "us"
          },
          {
            "name": "",
            "value": 80.636,
            "range": "+/- 1.342",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 80.085,
            "range": "+/- 2.306",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2513,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.3971,
            "range": "+/- 0.185",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.6743,
            "range": "+/- 0.083",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.7955,
            "range": "+/- 0.099",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5106,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.7705,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "",
            "value": 384.57,
            "range": "+/- 5.400",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 179.08,
            "range": "+/- 2.660",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 214.47,
            "range": "+/- 5.130",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.3689,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 212.79,
            "range": "+/- 4.640",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.2796,
            "range": "+/- 0.066",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.8623,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 227.5,
            "range": "+/- 8.980",
            "unit": "us"
          },
          {
            "name": "",
            "value": 196.17,
            "range": "+/- 3.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 202.53,
            "range": "+/- 6.010",
            "unit": "us"
          },
          {
            "name": "",
            "value": 326.21,
            "range": "+/- 3.460",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 331.52,
            "range": "+/- 7.840",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 264.37,
            "range": "+/- 3.320",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 278.36,
            "range": "+/- 7.570",
            "unit": "us"
          },
          {
            "name": "",
            "value": 182.23,
            "range": "+/- 2.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 188.98,
            "range": "+/- 5.010",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 184.53,
            "range": "+/- 6.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 182.37,
            "range": "+/- 3.750",
            "unit": "us"
          },
          {
            "name": "",
            "value": 188.06,
            "range": "+/- 2.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 187.72,
            "range": "+/- 2.980",
            "unit": "us"
          },
          {
            "name": "",
            "value": 178.64,
            "range": "+/- 2.750",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3193,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 811.78,
            "range": "+/- 14.390",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.237,
            "range": "+/- 0.129",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.106,
            "range": "+/- 0.117",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.133,
            "range": "+/- 0.036",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.06,
            "range": "+/- 0.335",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.4385,
            "range": "+/- 0.095",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.2538,
            "range": "+/- 0.123",
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
          "id": "795adc519afdcc7fada3453410eaa72ecbb473f8",
          "message": "Better error formatting and cli color (#586)",
          "timestamp": "2020-07-23T17:44:36+02:00",
          "tree_id": "1ca40e72513f697b83a9fe39269be8f4d151e14e",
          "url": "https://github.com/boa-dev/boa/commit/795adc519afdcc7fada3453410eaa72ecbb473f8"
        },
        "date": 1595520251330,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 153.77,
            "range": "+/- 2.160",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.8678,
            "range": "+/- 0.253",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.498,
            "range": "+/- 0.200",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 978.56,
            "range": "+/- 11.340",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.511,
            "range": "+/- 0.180",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.8086,
            "range": "+/- 0.075",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4703,
            "range": "+/- 0.015",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.5372,
            "range": "+/- 0.067",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8407,
            "range": "+/- 0.132",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.9496,
            "range": "+/- 0.283",
            "unit": "us"
          },
          {
            "name": "",
            "value": 70.461,
            "range": "+/- 2.608",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 68.683,
            "range": "+/- 1.531",
            "unit": "us"
          },
          {
            "name": "",
            "value": 70.943,
            "range": "+/- 0.757",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 70.926,
            "range": "+/- 1.279",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3252,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5676,
            "range": "+/- 0.113",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.2149,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.3581,
            "range": "+/- 0.044",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.4837,
            "range": "+/- 0.266",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.139,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "",
            "value": 380.5,
            "range": "+/- 8.710",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 165.04,
            "range": "+/- 2.190",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 195.81,
            "range": "+/- 4.500",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.1872,
            "range": "+/- 0.016",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 190.05,
            "range": "+/- 2.450",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.0329,
            "range": "+/- 0.046",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.7989,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 196.43,
            "range": "+/- 3.170",
            "unit": "us"
          },
          {
            "name": "",
            "value": 177.84,
            "range": "+/- 3.250",
            "unit": "us"
          },
          {
            "name": "",
            "value": 177.14,
            "range": "+/- 2.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 307.59,
            "range": "+/- 18.690",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 301.5,
            "range": "+/- 6.060",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 255.3,
            "range": "+/- 3.630",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 263.09,
            "range": "+/- 3.470",
            "unit": "us"
          },
          {
            "name": "",
            "value": 197.72,
            "range": "+/- 11.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 208.88,
            "range": "+/- 4.820",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 209.66,
            "range": "+/- 6.700",
            "unit": "us"
          },
          {
            "name": "",
            "value": 192.47,
            "range": "+/- 6.940",
            "unit": "us"
          },
          {
            "name": "",
            "value": 194.76,
            "range": "+/- 7.610",
            "unit": "us"
          },
          {
            "name": "",
            "value": 201.89,
            "range": "+/- 11.310",
            "unit": "us"
          },
          {
            "name": "",
            "value": 176.76,
            "range": "+/- 6.270",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.6738,
            "range": "+/- 0.077",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 913.23,
            "range": "+/- 37.880",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.7212,
            "range": "+/- 0.326",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.8873,
            "range": "+/- 0.178",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3348,
            "range": "+/- 0.085",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.878,
            "range": "+/- 0.497",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.8056,
            "range": "+/- 0.151",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 9.7734,
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
          "id": "24a72ea847b52549cfe305249d413d86f8289765",
          "message": "Added keyword and operator colors and matching bracket validator to cli (#590)",
          "timestamp": "2020-07-25T22:48:13+02:00",
          "tree_id": "71f780291da3628ca24a67f8d5c8e30371f4b30c",
          "url": "https://github.com/boa-dev/boa/commit/24a72ea847b52549cfe305249d413d86f8289765"
        },
        "date": 1595711304647,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 151.29,
            "range": "+/- 3.190",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.5489,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 22.673,
            "range": "+/- 0.389",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 935.07,
            "range": "+/- 15.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 13.384,
            "range": "+/- 0.346",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6213,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.4089,
            "range": "+/- 0.025",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.4941,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5314,
            "range": "+/- 0.142",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.2693,
            "range": "+/- 0.138",
            "unit": "us"
          },
          {
            "name": "",
            "value": 67.714,
            "range": "+/- 1.107",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 67.334,
            "range": "+/- 1.503",
            "unit": "us"
          },
          {
            "name": "",
            "value": 68.459,
            "range": "+/- 1.268",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 69.082,
            "range": "+/- 1.486",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5019,
            "range": "+/- 0.111",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5645,
            "range": "+/- 0.182",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.1953,
            "range": "+/- 0.072",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.292,
            "range": "+/- 0.121",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0371,
            "range": "+/- 0.069",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.0872,
            "range": "+/- 0.100",
            "unit": "us"
          },
          {
            "name": "",
            "value": 372.71,
            "range": "+/- 8.850",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 161.84,
            "range": "+/- 4.860",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 188.4,
            "range": "+/- 4.570",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.1735,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 187.92,
            "range": "+/- 3.470",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.1083,
            "range": "+/- 0.072",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.7171,
            "range": "+/- 0.028",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 193.88,
            "range": "+/- 3.810",
            "unit": "us"
          },
          {
            "name": "",
            "value": 173.26,
            "range": "+/- 2.890",
            "unit": "us"
          },
          {
            "name": "",
            "value": 184.42,
            "range": "+/- 5.550",
            "unit": "us"
          },
          {
            "name": "",
            "value": 286.23,
            "range": "+/- 8.600",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 288.99,
            "range": "+/- 7.940",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 247.46,
            "range": "+/- 4.020",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 253.64,
            "range": "+/- 4.720",
            "unit": "us"
          },
          {
            "name": "",
            "value": 171.84,
            "range": "+/- 4.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 179.43,
            "range": "+/- 3.090",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 169.31,
            "range": "+/- 4.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 169.29,
            "range": "+/- 3.120",
            "unit": "us"
          },
          {
            "name": "",
            "value": 177.09,
            "range": "+/- 2.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 182.09,
            "range": "+/- 3.740",
            "unit": "us"
          },
          {
            "name": "",
            "value": 173.37,
            "range": "+/- 5.760",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.3308,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 830.4,
            "range": "+/- 12.260",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.1378,
            "range": "+/- 0.101",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.4924,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.1031,
            "range": "+/- 0.035",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.802,
            "range": "+/- 0.351",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1179,
            "range": "+/- 0.071",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.3201,
            "range": "+/- 0.186",
            "unit": "us"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "rageknify@gmail.com",
            "name": "João Borges",
            "username": "RageKnify"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "667a820deef8dec39dbd44520c36b22fab586aa9",
          "message": "Introduce PropertyKey for field acces, fix #172 (quotes around displayed strings) (#373)\n\nCo-authored-by: HalidOdat <halidodat@gmail.com>",
          "timestamp": "2020-07-28T02:34:19+02:00",
          "tree_id": "d7db30cc634247d0b30f3673edeefa7e6c3d1df0",
          "url": "https://github.com/boa-dev/boa/commit/667a820deef8dec39dbd44520c36b22fab586aa9"
        },
        "date": 1595897578213,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 148.51,
            "range": "+/- 3.530",
            "unit": "us"
          },
          {
            "name": "Symbols (Execution)",
            "value": 2.8619,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 21.069,
            "range": "+/- 0.357",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 920.6,
            "range": "+/- 16.960",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7485,
            "range": "+/- 0.124",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.126,
            "range": "+/- 0.042",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.1064,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 3.9734,
            "range": "+/- 0.049",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.1707,
            "range": "+/- 0.095",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.8312,
            "range": "+/- 0.097",
            "unit": "us"
          },
          {
            "name": "",
            "value": 61.97,
            "range": "+/- 1.330",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 63.604,
            "range": "+/- 0.994",
            "unit": "us"
          },
          {
            "name": "",
            "value": 66.981,
            "range": "+/- 1.231",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 66.601,
            "range": "+/- 1.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.7432,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.8262,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 3.5248,
            "range": "+/- 0.052",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.2949,
            "range": "+/- 0.053",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0445,
            "range": "+/- 0.103",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0793,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "",
            "value": 366.26,
            "range": "+/- 6.360",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 152.74,
            "range": "+/- 2.230",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 176.56,
            "range": "+/- 3.660",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.129,
            "range": "+/- 0.029",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 171.95,
            "range": "+/- 3.470",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.4789,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.4209,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 179.31,
            "range": "+/- 2.740",
            "unit": "us"
          },
          {
            "name": "",
            "value": 158.71,
            "range": "+/- 3.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 162.39,
            "range": "+/- 2.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 265.4,
            "range": "+/- 4.240",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 274.37,
            "range": "+/- 4.000",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 232.24,
            "range": "+/- 3.290",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 232.22,
            "range": "+/- 3.350",
            "unit": "us"
          },
          {
            "name": "",
            "value": 161.86,
            "range": "+/- 2.410",
            "unit": "us"
          },
          {
            "name": "",
            "value": 165.32,
            "range": "+/- 3.070",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 156.15,
            "range": "+/- 2.700",
            "unit": "us"
          },
          {
            "name": "",
            "value": 159.35,
            "range": "+/- 3.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 163.47,
            "range": "+/- 2.430",
            "unit": "us"
          },
          {
            "name": "",
            "value": 167.89,
            "range": "+/- 2.860",
            "unit": "us"
          },
          {
            "name": "",
            "value": 152.76,
            "range": "+/- 3.960",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.1697,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 757.48,
            "range": "+/- 10.180",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.9743,
            "range": "+/- 0.133",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.9797,
            "range": "+/- 0.054",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.029,
            "range": "+/- 0.031",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 13.296,
            "range": "+/- 0.209",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.1776,
            "range": "+/- 0.058",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.473,
            "range": "+/- 0.116",
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
          "id": "7b3f42de54134b4e7c638e3e74ec66124aea4703",
          "message": "Merge pull request #587 from joshwd36/Well-Known-Symbols\n\nImplement Well-Known Symbols",
          "timestamp": "2020-07-29T15:48:42+01:00",
          "tree_id": "0becb51f19681f7381a0b3c2ee4bd755991279e8",
          "url": "https://github.com/boa-dev/boa/commit/7b3f42de54134b4e7c638e3e74ec66124aea4703"
        },
        "date": 1596035315313,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 451.53,
            "range": "+/- 10.280",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.2919,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 23.904,
            "range": "+/- 0.490",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.1191,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 9.372,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.4094,
            "range": "+/- 0.044",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.2086,
            "range": "+/- 0.017",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 4.8883,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.1202,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.6692,
            "range": "+/- 0.075",
            "unit": "us"
          },
          {
            "name": "",
            "value": 76.233,
            "range": "+/- 1.542",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 74.548,
            "range": "+/- 1.839",
            "unit": "us"
          },
          {
            "name": "",
            "value": 77.363,
            "range": "+/- 1.061",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 79.373,
            "range": "+/- 1.395",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.3953,
            "range": "+/- 0.094",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.503,
            "range": "+/- 0.135",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 4.0471,
            "range": "+/- 0.081",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.4379,
            "range": "+/- 0.066",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.471,
            "range": "+/- 0.068",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.7303,
            "range": "+/- 0.118",
            "unit": "us"
          },
          {
            "name": "",
            "value": 375.81,
            "range": "+/- 6.300",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 182.62,
            "range": "+/- 4.460",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 220.9,
            "range": "+/- 5.980",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.3071,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 210.04,
            "range": "+/- 4.960",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 3.7864,
            "range": "+/- 0.049",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.6051,
            "range": "+/- 0.027",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 226.82,
            "range": "+/- 4.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 198.51,
            "range": "+/- 4.800",
            "unit": "us"
          },
          {
            "name": "",
            "value": 203.89,
            "range": "+/- 5.200",
            "unit": "us"
          },
          {
            "name": "",
            "value": 341.02,
            "range": "+/- 9.350",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 378.58,
            "range": "+/- 15.900",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 297.94,
            "range": "+/- 7.340",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 293.52,
            "range": "+/- 4.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 195.36,
            "range": "+/- 4.080",
            "unit": "us"
          },
          {
            "name": "",
            "value": 220.11,
            "range": "+/- 4.640",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 192.27,
            "range": "+/- 2.890",
            "unit": "us"
          },
          {
            "name": "",
            "value": 206.58,
            "range": "+/- 7.510",
            "unit": "us"
          },
          {
            "name": "",
            "value": 206.87,
            "range": "+/- 4.350",
            "unit": "us"
          },
          {
            "name": "",
            "value": 211.26,
            "range": "+/- 5.130",
            "unit": "us"
          },
          {
            "name": "",
            "value": 187.93,
            "range": "+/- 2.790",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.5357,
            "range": "+/- 0.045",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 895.64,
            "range": "+/- 19.060",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.6247,
            "range": "+/- 0.109",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.6493,
            "range": "+/- 0.141",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.3082,
            "range": "+/- 0.051",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 14.918,
            "range": "+/- 0.402",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 6.8129,
            "range": "+/- 0.124",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 8.8882,
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
          "id": "a6710fa6146e69ba1f3f0c8ed01eff13ce5a759d",
          "message": "`RegExp` specialization (#592)",
          "timestamp": "2020-07-29T17:18:57+02:00",
          "tree_id": "b546b42e3dab608d4e1f9aa777acff39db128f68",
          "url": "https://github.com/boa-dev/boa/commit/a6710fa6146e69ba1f3f0c8ed01eff13ce5a759d"
        },
        "date": 1596037018278,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 408.56,
            "range": "+/- 13.030",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 2.9436,
            "range": "+/- 0.105",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 21.387,
            "range": "+/- 0.610",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 974.17,
            "range": "+/- 33.870",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.9716,
            "range": "+/- 0.331",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.1196,
            "range": "+/- 0.117",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.0972,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 4.4631,
            "range": "+/- 0.169",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.4662,
            "range": "+/- 0.144",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.9189,
            "range": "+/- 0.160",
            "unit": "us"
          },
          {
            "name": "",
            "value": 60.205,
            "range": "+/- 2.046",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 61.463,
            "range": "+/- 2.587",
            "unit": "us"
          },
          {
            "name": "",
            "value": 64.998,
            "range": "+/- 2.210",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 65.606,
            "range": "+/- 2.600",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.5457,
            "range": "+/- 0.171",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.5362,
            "range": "+/- 0.213",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 3.391,
            "range": "+/- 0.115",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.2175,
            "range": "+/- 0.109",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.9659,
            "range": "+/- 0.129",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.0275,
            "range": "+/- 0.219",
            "unit": "us"
          },
          {
            "name": "",
            "value": 309.48,
            "range": "+/- 10.850",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 153.37,
            "range": "+/- 4.890",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 184.77,
            "range": "+/- 5.700",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.0773,
            "range": "+/- 0.037",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 174.9,
            "range": "+/- 5.510",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 2.9767,
            "range": "+/- 0.059",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.2993,
            "range": "+/- 0.055",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 183.95,
            "range": "+/- 5.900",
            "unit": "us"
          },
          {
            "name": "",
            "value": 166.75,
            "range": "+/- 5.030",
            "unit": "us"
          },
          {
            "name": "",
            "value": 169.25,
            "range": "+/- 4.220",
            "unit": "us"
          },
          {
            "name": "",
            "value": 259.57,
            "range": "+/- 9.050",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 257.09,
            "range": "+/- 6.990",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 243.16,
            "range": "+/- 8.520",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 237.77,
            "range": "+/- 5.690",
            "unit": "us"
          },
          {
            "name": "",
            "value": 165.46,
            "range": "+/- 5.530",
            "unit": "us"
          },
          {
            "name": "",
            "value": 164.17,
            "range": "+/- 4.690",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 156.18,
            "range": "+/- 3.700",
            "unit": "us"
          },
          {
            "name": "",
            "value": 164.2,
            "range": "+/- 5.790",
            "unit": "us"
          },
          {
            "name": "",
            "value": 165.9,
            "range": "+/- 5.380",
            "unit": "us"
          },
          {
            "name": "",
            "value": 172.46,
            "range": "+/- 5.850",
            "unit": "us"
          },
          {
            "name": "",
            "value": 153.97,
            "range": "+/- 4.160",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 1.9892,
            "range": "+/- 0.064",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 715.97,
            "range": "+/- 18.330",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 4.4425,
            "range": "+/- 0.156",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 4.5274,
            "range": "+/- 0.136",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 1.8006,
            "range": "+/- 0.050",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 12.323,
            "range": "+/- 0.336",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 5.5285,
            "range": "+/- 0.089",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 7.0931,
            "range": "+/- 0.191",
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
          "id": "4474b714b2e0152852a1406b17cd0c742755c80a",
          "message": "Added syntax highlighting for strings (#595)",
          "timestamp": "2020-07-29T17:24:48+02:00",
          "tree_id": "eb00dda5c45a7017b541f371e18ca0000ea5d456",
          "url": "https://github.com/boa-dev/boa/commit/4474b714b2e0152852a1406b17cd0c742755c80a"
        },
        "date": 1596037494740,
        "tool": "criterion",
        "benches": [
          {
            "name": "Create Realm",
            "value": 502.23,
            "range": "+/- 10.130",
            "unit": "ns"
          },
          {
            "name": "Symbols (Execution)",
            "value": 3.9026,
            "range": "+/- 0.080",
            "unit": "us"
          },
          {
            "name": "For loop (Execution)",
            "value": 26.612,
            "range": "+/- 0.835",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Execution)",
            "value": 1.2422,
            "range": "+/- 0.023",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 10.793,
            "range": "+/- 0.290",
            "unit": "us"
          },
          {
            "name": "",
            "value": 3.6841,
            "range": "+/- 0.051",
            "unit": "ms"
          },
          {
            "name": "Array pop (Execution)",
            "value": 1.3344,
            "range": "+/- 0.020",
            "unit": "ms"
          },
          {
            "name": "",
            "value": 5.7751,
            "range": "+/- 0.166",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.9748,
            "range": "+/- 0.122",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.6021,
            "range": "+/- 0.125",
            "unit": "us"
          },
          {
            "name": "",
            "value": 79.159,
            "range": "+/- 1.852",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution)",
            "value": 78.942,
            "range": "+/- 2.005",
            "unit": "us"
          },
          {
            "name": "",
            "value": 83.597,
            "range": "+/- 1.608",
            "unit": "us"
          },
          {
            "name": "RegExp (Execution) #2",
            "value": 82.83,
            "range": "+/- 1.640",
            "unit": "us"
          },
          {
            "name": "",
            "value": 6.5681,
            "range": "+/- 0.169",
            "unit": "us"
          },
          {
            "name": "",
            "value": 8.2538,
            "range": "+/- 0.279",
            "unit": "us"
          },
          {
            "name": "String copy (Execution)",
            "value": 5.0747,
            "range": "+/- 0.082",
            "unit": "us"
          },
          {
            "name": "",
            "value": 4.0806,
            "range": "+/- 0.071",
            "unit": "us"
          },
          {
            "name": "",
            "value": 5.4674,
            "range": "+/- 0.123",
            "unit": "us"
          },
          {
            "name": "",
            "value": 7.7022,
            "range": "+/- 0.107",
            "unit": "us"
          },
          {
            "name": "",
            "value": 466.13,
            "range": "+/- 7.110",
            "unit": "ns"
          },
          {
            "name": "Symbols (Full)",
            "value": 205.29,
            "range": "+/- 4.900",
            "unit": "us"
          },
          {
            "name": "For loop (Full)",
            "value": 234.53,
            "range": "+/- 4.780",
            "unit": "us"
          },
          {
            "name": "Fibonacci (Full)",
            "value": 1.4386,
            "range": "+/- 0.026",
            "unit": "ms"
          },
          {
            "name": "Array access (Full)",
            "value": 230.23,
            "range": "+/- 4.870",
            "unit": "us"
          },
          {
            "name": "Array creation (Full)",
            "value": 4.0141,
            "range": "+/- 0.053",
            "unit": "ms"
          },
          {
            "name": "Array pop (Full)",
            "value": 1.6578,
            "range": "+/- 0.036",
            "unit": "ms"
          },
          {
            "name": "Object Creation (Full)",
            "value": 244.95,
            "range": "+/- 5.630",
            "unit": "us"
          },
          {
            "name": "",
            "value": 218.05,
            "range": "+/- 5.810",
            "unit": "us"
          },
          {
            "name": "",
            "value": 220.98,
            "range": "+/- 4.470",
            "unit": "us"
          },
          {
            "name": "",
            "value": 342.73,
            "range": "+/- 6.190",
            "unit": "us"
          },
          {
            "name": "RegExp (Full)",
            "value": 358.51,
            "range": "+/- 6.440",
            "unit": "us"
          },
          {
            "name": "RegExp Literal (Full)",
            "value": 313.35,
            "range": "+/- 6.760",
            "unit": "us"
          },
          {
            "name": "RegExp (Full) #2",
            "value": 311.35,
            "range": "+/- 6.240",
            "unit": "us"
          },
          {
            "name": "",
            "value": 213.23,
            "range": "+/- 3.270",
            "unit": "us"
          },
          {
            "name": "",
            "value": 223.73,
            "range": "+/- 5.160",
            "unit": "us"
          },
          {
            "name": "String copy (Full)",
            "value": 214.89,
            "range": "+/- 6.990",
            "unit": "us"
          },
          {
            "name": "",
            "value": 215.16,
            "range": "+/- 3.830",
            "unit": "us"
          },
          {
            "name": "",
            "value": 229.78,
            "range": "+/- 4.790",
            "unit": "us"
          },
          {
            "name": "",
            "value": 221.21,
            "range": "+/- 7.140",
            "unit": "us"
          },
          {
            "name": "",
            "value": 203.79,
            "range": "+/- 4.530",
            "unit": "us"
          },
          {
            "name": "Expression (Lexer)",
            "value": 2.689,
            "range": "+/- 0.048",
            "unit": "us"
          },
          {
            "name": "Hello World (Lexer)",
            "value": 963.75,
            "range": "+/- 16.600",
            "unit": "ns"
          },
          {
            "name": "For loop (Lexer)",
            "value": 5.9785,
            "range": "+/- 0.112",
            "unit": "us"
          },
          {
            "name": "Expression (Parser)",
            "value": 5.9491,
            "range": "+/- 0.156",
            "unit": "us"
          },
          {
            "name": "Hello World (Parser)",
            "value": 2.4124,
            "range": "+/- 0.062",
            "unit": "us"
          },
          {
            "name": "For loop (Parser)",
            "value": 16.261,
            "range": "+/- 0.379",
            "unit": "us"
          },
          {
            "name": "Long file (Parser)",
            "value": 7.3675,
            "range": "+/- 0.076",
            "unit": "ms"
          },
          {
            "name": "Goal Symbols (Parser)",
            "value": 9.627,
            "range": "+/- 0.164",
            "unit": "us"
          }
        ]
      }
    ]
  }
}