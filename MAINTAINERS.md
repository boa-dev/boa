# Maintainers Guide

This document contains information for maintainers of the project.

## CI GIF Example

If you want a small animated demo in CI, consider using `vhs` to record a short terminal session and upload a GIF via artifacts or use a GIF action. Example (adapt to your CI):

```bash
# Install vhs and asciinema locally (or in CI image)
npm install -g @softprops/vhs

# Record a short script to a GIF (vhs saves demo.gif)
vhs record --output demo.cast --command "cargo run --bin boa -- examples/helloworld.js"
vhs render demo.cast --output demo.gif

# Upload demo.gif as a GitHub Action artifact and/or embed in the README
```

TODO: Add a small GitHub Actions workflow step to generate and publish the GIF when merging to `main` (keep the demo <10s to limit CI time).