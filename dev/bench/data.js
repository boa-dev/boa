window.BENCHMARK_DATA = {
  "lastUpdate": 1581377868184,
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
      }
    ]
  }
}