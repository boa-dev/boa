import json
from datetime import datetime, timezone

def new_valid_bench(bench):
    # This runs showed to be a clear outliers
    outliers = [
        "76a27ce2a5ac953c18fcc9f9376fe9e44af61a0e",
        "f685a6757dffbc5267fb3b21091e6cc66c706184",
        "6093a6689983a8c6bd83cfa93445784ac9b300a8",
        "0027f26d21f8e7b013391a7c87b778b73d732d35",
        "55e85adbc5af7da4d0474a948b6a00c9890aafb7",
        "9f6aa1972ce37f10ac15d71a46a616b0652a2786",
        "ab4d2899d5dc44c91c80729d9cd19e0f8904fd28",

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
