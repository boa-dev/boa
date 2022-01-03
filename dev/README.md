# Cleaning up benchmarks

We have noticed that sometimes we store empty benchmarks (should be solved now that we use the
upstream action). And if we don't clean-up anything, old benchmarks will stay in the JSON file
forever. In order to avoid this, you can use the `process.py` file.

To use this, rename the `dev/bench/data.js` file to `dev/bench/data.json` and remove the initial
`window.BENCHMARK_DATA = ` from it, leaving it as a full JSON file. Then run the file from the
repository root by running `dev/process.py`. This will create a `dev/bench/data_processed.json`
file. You will then just have to rename that file to `dev/bench/data.js` and add
`window.BENCHMARK_DATA = ` at the beginning of the file to convert it back to a JavaScript file.

While this is not very user friendly, it does the job, and hopefully we should just do it once in a
while to clean-up old benchmarks.
