# .github/main.workflow
workflow "benchmark pull requests" {
  on = "pull_request"
  resolves = ["run benchmark"]
}

action "run benchmark" {
  uses = "matchai/criterion-compare-action@master"
  secrets = ["GITHUB_TOKEN"]
}