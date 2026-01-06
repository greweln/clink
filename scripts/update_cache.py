import subprocess
import os

# Script to read, deduplicate, and update the repository cache file, then run `git fetch` in each unique repository path.
cache_file = os.path.expanduser("~/.cache/clink")

entries = {}

with open(cache_file, 'r') as f:
    for line in f:
        repo, last_update = line.rsplit(':', 1)
        last_update = int(last_update)


        if repo not in entries or last_update > entries[repo]:
            entries[repo] = last_update

with open(cache_file, 'w') as f:
    for repo, last_update in entries.items():
        f.write(f"{repo}:{last_update}\n")

for repo in entries.keys():
    # Only try to fetch if the directory still exists on disk
    if os.path.exists(repo):
        subprocess.run(['git', '-C', repo, 'fetch'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

