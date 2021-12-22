window.BENCHMARK_DATA = {
  "lastUpdate": 1640146565748,
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
      }
    ]
  }
}