import urllib.request
import json
import traceback

try:
    with open('pr_output2.txt', 'w', encoding='utf-8') as f:
        url = "https://api.github.com/repos/boa-dev/boa/pulls/5076/comments"
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            for comment in data:
                f.write(f"[LINE COMMENT {comment['created_at']}] {comment['user']['login']}: {comment['body']}\n{'-'*40}\n")
except Exception as e:
    with open('pr_output2.txt', 'w', encoding='utf-8') as f:
        f.write(traceback.format_exc())
