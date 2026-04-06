import subprocess
import sys

try:
    result = subprocess.run(
        ["cargo", "check", "--manifest-path", "core/engine/Cargo.toml"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    with open("cargo_error.txt", "w", encoding="utf-8") as f:
        f.write("STDOUT:\n")
        f.write(result.stdout)
        f.write("\nSTDERR:\n")
        f.write(result.stderr)
        f.write("\nRETURN CODE: " + str(result.returncode))
    print("Cargo check finished and output saved to cargo_error.txt")
except Exception as e:
    print("Error running cargo: " + str(e))
