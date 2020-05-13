window.BENCHMARK_DATA = {
  "lastUpdate": 1589397028134,
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
      }
    ]
  }
}