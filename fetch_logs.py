import urllib.request
import json

req = urllib.request.Request("https://api.github.com/repos/boa-dev/boa/pulls/4946")
req.add_header("User-Agent", "Mozilla/5.0")
with urllib.request.urlopen(req) as response:
    pr = json.loads(response.read().decode())
    head_sha = pr['head']['sha']
    
    req_checks = urllib.request.Request(f"https://api.github.com/repos/boa-dev/boa/commits/{head_sha}/check-runs")
    req_checks.add_header("User-Agent", "Mozilla/5.0")
    with urllib.request.urlopen(req_checks) as r_checks:
        checks = json.loads(r_checks.read().decode())
        for check in checks['check_runs']:
            if check['conclusion'] == 'failure' and 'Lint' in check['name']:
                print(f"Found failed Lint check: {check['name']} ({check['id']})")
                if 'output' in check and check['output']:
                    print(json.dumps(check['output'], indent=2))
                else:
                    print("No output found in check run.")
