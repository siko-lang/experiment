#!/usr/bin/python3

import os
import subprocess
import sys

success = 0
failure = 0
skipped = 0

def test(root, entry):
    print("- %s" % entry)
    global success, failure, skipped
    skip_path = os.path.join(root, entry, "SKIP")
    if os.path.exists(skip_path):
        skipped += 1
        return
    input_path = os.path.join(root, entry, "main.sk")
    output_path = os.path.join(root, entry, "main.ll")
    llvm_output_path = os.path.join(root, entry, "main.bin")
    std = []
    for m in os.listdir("./std"):
        std.append(os.path.join("./std", m))
    args = ["./siko", input_path, "-o", output_path]
    #print(args)
    r = subprocess.run(args)
    if r.returncode != 0:
        failure += 1
        return
    r = subprocess.run(["clang", "-Wno-override-module", output_path, "-o", llvm_output_path])
    #r = subprocess.run(["rustc", output_path, "-o", rust_output_path])
    if r.returncode != 0:
        failure += 1
        return
    r = subprocess.run([llvm_output_path])
    if r.returncode != 0:
        failure += 1
        return
    success += 1

filters = []
for arg in sys.argv[1:]:
    filters.append(arg)

no_std_path = os.path.join(".", "test", "no_std")

for entry in os.listdir(no_std_path):
    if len(filters) > 0 and entry not in filters:
        continue
    test(no_std_path, entry)
percent = 0
if (success+failure) != 0:
    percent = success/(success+failure)*100
print("Success %s/%s/%s - %.2f%%" % (success, success + failure, skipped, percent))
