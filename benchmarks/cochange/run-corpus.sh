#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: run-corpus.sh <local-bare-repository-root> <output-directory>" >&2
  exit 2
fi

repo_root=$1
output_dir=$2
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
project_root=$(cd -- "$script_dir/../../.." && pwd)
mkdir -p "$output_dir/queries"
results_file="$output_dir/results.jsonl"
: > "$results_file"

benchmark_revision=$(git -C "$project_root" rev-parse HEAD)
declare -A heads=(
  [bat]=e95c32b6f0c31eae638e492f9cc58a075b0e9950
  [delta]=3c2269c6b845913965575f11b0196d70ef5352f3
  [fd]=cc4e40fa4e32d61733cd72926e233a3fb6c8d56d
  [fzf]=26ff936df038b6a62ec42cf18db27b2ea2bb8ec3
  [hyperfine]=c5161524980522320c7d3425e714cc0965a869a5
  [jq]=992db9b7e5e3059b764bf430bd97716829e99578
  [just]=c7317c85c21ebd83ea4bb8de26f8afec44c7e597
  [ripgrep]=91f5d22ef48bde459d90a8c7dbeab416e97d0f2b
  [tokei]=c14f744716272fadeb27a74443cdffa0af35f82f
  [zoxide]=f0b31631d295b0623bc440016ed0aba3c7c49dde
)
declare -A sources=(
  [bat]=https://github.com/sharkdp/bat.git
  [delta]=https://github.com/dandavison/delta.git
  [fd]=https://github.com/sharkdp/fd.git
  [fzf]=https://github.com/junegunn/fzf.git
  [hyperfine]=https://github.com/sharkdp/hyperfine.git
  [jq]=https://github.com/jqlang/jq.git
  [just]=https://github.com/casey/just.git
  [ripgrep]=https://github.com/BurntSushi/ripgrep.git
  [tokei]=https://github.com/XAMPPRocky/tokei.git
  [zoxide]=https://github.com/ajeetdsouza/zoxide.git
)

for name in bat delta fd fzf hyperfine jq just ripgrep tokei zoxide; do
  repository="$repo_root/$name.git"
  if [[ ! -d "$repository" ]]; then
    echo "missing local bare repository: $repository" >&2
    exit 2
  fi
  actual_head=$(git -C "$repository" rev-parse HEAD)
  if [[ "$actual_head" != "${heads[$name]}" ]]; then
    echo "frozen HEAD mismatch for $name: expected ${heads[$name]}, got $actual_head" >&2
    exit 2
  fi
  raw_file="$output_dir/.${name}.raw.json"
  query_file="$output_dir/queries/$name.jsonl"
  cargo bench --manifest-path "$project_root/Cargo.toml" --bench cochange -- "$repository" 0.8 --per-query "$query_file" > "$raw_file"
  python3 - "$name" "${sources[$name]}" "$actual_head" "$benchmark_revision" "$raw_file" "$query_file" "$results_file" <<'PY'
import json
import os
import sys

name, source, head, benchmark_revision, raw_path, query_path, results_path = sys.argv[1:]
with open(raw_path, encoding="utf-8") as f:
    result = json.load(f)
result["repo"] = name

query_tmp = query_path + ".tmp"
with open(query_path, encoding="utf-8") as src, open(query_tmp, "w", encoding="utf-8") as dst:
    for line in src:
        query = json.loads(line)
        query["repo"] = name
        query["source_head"] = head
        query["benchmark_revision"] = benchmark_revision
        dst.write(json.dumps(query, ensure_ascii=False, separators=(",", ":")) + "\n")
os.replace(query_tmp, query_path)

row = {
    "repo": name,
    "source": source,
    "source_head": head,
    "benchmark_revision": benchmark_revision,
    "seconds": result["latency_ms"] / 1000,
    "exit_code": 0,
    "result": result,
}
with open(results_path, "a", encoding="utf-8") as f:
    f.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
PY
  rm -f "$raw_file"
done

echo "wrote portable results to $results_file and per-query rows to $output_dir/queries/"
