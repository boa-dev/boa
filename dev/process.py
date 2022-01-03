import json
from datetime import datetime, timezone

def new_bench(bench):
    date = datetime.strptime(bench["commit"]["timestamp"], "%Y-%m-%dT%H:%M:%S%z")
    date_diff = now - date
    diff_years = (date_diff.days + date_diff.seconds/86400)/365.2425

    return diff_years < 1

def non_empty_name(bench):
    return bench["name"] != ""

def clean_bench(bench):
    bench["benches"] = list(filter(non_empty_name, bench["benches"]))
    return bench

with open('./dev/bench/data.json') as input_f:
    data = json.load(input_f)
    benches = data["entries"]["Boa Benchmarks"]

    now = datetime.now(timezone.utc)
    total = len(benches)
    print("Benches: ", total)

    data["entries"]["Boa Benchmarks"] = list(map(clean_bench, filter(new_bench, benches)))

    with open('./dev/bench/data_processed.json', 'w') as output_f:
        json.dump(data, output_f, indent=2)
