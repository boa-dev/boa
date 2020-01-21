window.BENCHMARK_DATA = {
  "lastUpdate": 1579642940529,
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
      }
    ]
  }
}