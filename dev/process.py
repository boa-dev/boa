import json
from datetime import datetime, timezone

def new_valid_bench(bench):
    # This runs showed to be a clear outliers
    outliers = [
        "13df9a1984d729328fda88ca9432a84eb4990d3b",
        "71daf2a06ca411ab821931d4202875be1b0b4026",
        "518bad810974aa4cf6fcd3d864756f298d1b204d",
        "39cd3b8bdffada751621132de1f8b65d04627d6d",
        "a2a33e749fa201205bb10a828afd24a3919fb4d6",
        "fcf456e5ded032841e9c89255ef820070e3120e1",
        "099154cb34c4f7fec6cf8d45f1f96b6797e10dd4",
        "b0cf873a466bf3c71d258b20c81103f1ca696e97",
    ]

    date = datetime.strptime(bench["commit"]["timestamp"], "%Y-%m-%dT%H:%M:%S%z")
    date_diff = now - date
    diff_years = (date_diff.days + date_diff.seconds/86400)/365.2425

    return diff_years < 1 and bench["commit"]["id"] not in outliers

def relevant_bench(bench):
    irrelevant_benches = [
        "RegExp (Execution) #2",
        "Expression (Parser)",
        "Hello World (Parser)",
        "Long file (Parser)",
        "Goal Symbols (Parser)",
    ]

    return bench["name"] not in irrelevant_benches and "(Full)" not in bench["name"]

def non_empty_name(bench):
    return bench["name"] != ""

def clean_bench(bench):
    bench["benches"] = list(filter(relevant_bench, filter(non_empty_name, bench["benches"])))
    return bench

with open('./dev/bench/data.json') as input_f:
    data = json.load(input_f)
    benches = data["entries"]["Boa Benchmarks"]

    now = datetime.now(timezone.utc)
    total = len(benches)
    print("Benches: ", total)

    data["entries"]["Boa Benchmarks"] = list(map(clean_bench, filter(new_valid_bench, benches)))

    with open('./dev/bench/data_processed.json', 'w') as output_f:
        json.dump(data, output_f, indent=2, ensure_ascii=False)
