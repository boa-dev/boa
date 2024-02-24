import json
import os


def suite_conformance(suite):
    global function_suite
    res = {
        "t": 0,
        "o": 0,
        "i": 0,
        "p": 0
    }

    if "s" in suite.keys():
        for subSuite in suite["s"].values():
            conformance = suite_conformance(subSuite)
            res["t"] += conformance["t"]
            res["o"] += conformance["o"]
            res["i"] += conformance["i"]
            res["p"] += conformance["p"]

    if "t" in suite.keys():
        for testName in suite["t"].keys():
            test = suite["t"][testName]

            res["t"] += 1
            if "s" in test.keys() and "r" in test.keys():
                if test["s"] == "O" and test["r"] == "O":
                    res["o"] += 1
                elif test["s"] == "I" and test["r"] == "I":
                    res["i"] += 1
                elif test["s"] == "P" or test["r"] == "P":
                    res["p"] += 1
            elif "s" in test.keys():
                if test["s"] == "O":
                    res["o"] += 1
                elif test["s"] == "I":
                    res["i"] += 1
                elif test["s"] == "P":
                    res["p"] += 1
            else:
                if test["r"] == "O":
                    res["o"] += 1
                elif test["r"] == "I":
                    res["i"] += 1
                elif test["r"] == "P":
                    res["p"] += 1

    return res


def version_conformance(suite):
    res = {}

    if "s" in suite.keys():
        for subSuite in suite["s"].values():
            versions = version_conformance(subSuite)
            for key in versions.keys():
                if key not in res.keys():
                    res[key] = {
                        "t": 0,
                        "o": 0,
                        "i": 0,
                        "p": 0
                    }

                res[key]["t"] += versions[key]["t"]
                res[key]["o"] += versions[key]["o"]
                res[key]["i"] += versions[key]["i"]
                res[key]["p"] += versions[key]["p"]

    if "t" in suite.keys():
        for testName in suite["t"].keys():
            test = suite["t"][testName]

            if "v" in test.keys():
                version = test["v"]
                if version != 255:
                    key = str(version)
                    if key not in res.keys():
                        res[key] = {
                            "t": 0,
                            "o": 0,
                            "i": 0,
                            "p": 0
                        }

                    res[key]["t"] += 1
                    if "s" in test.keys() and "r" in test.keys():
                        if test["s"] == "O" and test["r"] == "O":
                            res[key]["o"] += 1
                        elif test["s"] == "I" and test["r"] == "I":
                            res[key]["i"] += 1
                        elif test["s"] == "P" or test["r"] == "P":
                            res[key]["p"] += 1
                    elif "s" in test.keys():
                        if test["s"] == "O":
                            res[key]["o"] += 1
                        elif test["s"] == "I":
                            res[key]["i"] += 1
                        elif test["s"] == "P":
                            res[key]["p"] += 1
                    else:
                        if test["r"] == "O":
                            res[key]["o"] += 1
                        elif test["r"] == "I":
                            res[key]["i"] += 1
                        elif test["r"] == "P":
                            res[key]["p"] += 1

    return res


def fix_tests(tests):
    fixed = {}

    for test in tests:
        name = test["n"]
        if test["n"] in fixed:
            if test["s"]:
                fixed[name]["s"] = test["r"]
            else:
                fixed[name]["r"] = test["r"]
        else:
            fixed[name] = {}

            if "v" in test.keys():
                fixed[name]["v"] = test["v"]

            if "s" in test.keys():
                if test["s"]:
                    fixed[name]["s"] = test["r"]
                else:
                    fixed[name]["r"] = test["r"]
            else:
                fixed[name]["r"] = test["r"]

    return fixed


def fix_suite(suites):
    fixed = {}
    for suite in suites:
        name = suite["n"]
        fixed[name] = {}

        if "s" in suite.keys():
            fixed[name]["s"] = fix_suite(suite["s"])

        if "t" in suite.keys():
            fixed[name]["t"] = fix_tests(suite["t"])

        fixed[name]["a"] = suite_conformance(fixed[name])
        fixed[name]["v"] = version_conformance(fixed[name])

    return fixed


def fix_all(latest):
    fixed = {
        "c": latest["c"],
        "u": latest["u"],
        "r": fix_suite(latest["r"]["s"]),
        "a": {
            "t": 0,
            "o": 0,
            "i": 0,
            "p": 0
        },
        "v": {},
    }

    for suite in fixed["r"].values():
        fixed["a"]["t"] += suite["a"]["t"]
        fixed["a"]["o"] += suite["a"]["o"]
        fixed["a"]["i"] += suite["a"]["i"]
        fixed["a"]["p"] += suite["a"]["p"]

        for key in suite["v"].keys():
            if key not in fixed["v"].keys():
                fixed["v"][key] = {
                    "t": 0,
                    "o": 0,
                    "i": 0,
                    "p": 0
                }

            fixed["v"][key]["t"] += suite["v"][key]["t"]
            fixed["v"][key]["o"] += suite["v"][key]["o"]
            fixed["v"][key]["i"] += suite["v"][key]["i"]
            fixed["v"][key]["p"] += suite["v"][key]["p"]

    return fixed


def fix_file(file_name):
    with open(file_name) as latest_f:
        latest = json.load(latest_f)
        fixed_latest = fix_all(latest)

        with open(file_name, 'w') as latest_of:
            json.dump(fixed_latest, latest_of, separators=(
                ',', ':'), ensure_ascii=False)

        return fixed_latest


def clean_main(latest):
    with open("./refs/heads/main/results.json") as results_f:
        results = json.load(results_f)
        fixed_results = []
        for result in results:
            fixed_results.append({
                "c": result["c"],
                "u": result["u"],
                "a": result["a"],
            })

        fixed_results[-1] = {
            "c": latest["c"],
            "u": latest["u"],
            "a": latest["a"],
        }

        with open("./refs/heads/main/results.json", 'w') as results_of:
            json.dump(fixed_results, results_of, separators=(
                ',', ':'), ensure_ascii=False)


def clean_old(file_name, results):
    fixed_results = [{
        "c": results["c"],
        "u": results["u"],
        "a": results["a"],
    }]

    with open(file_name, 'w') as results_of:
        json.dump(fixed_results, results_of, separators=(
            ',', ':'), ensure_ascii=False)


for top, dirs, files in os.walk("./refs/tags"):
    for dir in dirs:
        print("Fixing " + dir)
        results = fix_file("./refs/tags/" + dir + "/latest.json")
        clean_old("./refs/tags/" + dir + "/results.json", results)

        if os.path.exists("./refs/tags/" + dir + "/features.json"):
            os.remove("./refs/tags/" + dir + "/features.json")

print("Fixing main branch")
results = fix_file("./refs/heads/main/latest.json")
clean_main(results)
if os.path.exists("./refs/heads/main/features.json"):
    os.remove("./refs/heads/main/features.json")
