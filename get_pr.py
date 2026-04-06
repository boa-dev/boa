import urllib.request
import json
import traceback

try:
    with open('pr_output.txt', 'w', encoding='utf-8') as f:
        url = "https://api.github.com/repos/boa-dev/boa/issues/5076/comments"
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read().decode('utf-8'))
            for comment in data:
                f.write(f"[{comment['created_at']}] {comment['user']['login']}: {comment['body']}\n{'-'*40}\n")
                
        url2 = "https://api.github.com/repos/boa-dev/boa/pulls/5076/reviews"
        req2 = urllib.request.Request(url2, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req2) as response2:
            data2 = json.loads(response2.read().decode('utf-8'))
            for review in data2:
                f.write(f"[REVIEW {review['submitted_at']}] {review['user']['login']}: {review['body']}\n{'-'*40}\n")
except Exception as e:
    with open('pr_output.txt', 'w', encoding='utf-8') as f:
        f.write(traceback.format_exc())
