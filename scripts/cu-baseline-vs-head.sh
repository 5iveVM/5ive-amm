#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BASELINE_COMMIT="${BASELINE_COMMIT:-8f0b32a7723315e2e8c5e331b7dbc6702ae8fb53}"
HEAD_COMMIT="$(git rev-parse --short=12 HEAD)"
OUT_DIR="${OUT_DIR:-five-solana/tests/benchmarks/reports}"
OUT_JSON="${OUT_JSON:-$OUT_DIR/baseline_vs_head.json}"
OUT_MD="${OUT_MD:-$OUT_DIR/baseline_vs_head.md}"

mkdir -p "$OUT_DIR"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
BASELINE_LINES="$TMP_DIR/baseline.ndjson"
HEAD_LINES="$TMP_DIR/head.ndjson"
BASELINE_REDUCED="$TMP_DIR/baseline_reduced.json"
HEAD_REDUCED="$TMP_DIR/head_reduced.json"

: > "$BASELINE_LINES"
: > "$HEAD_LINES"

# Baseline scenarios from commit tree
while IFS= read -r file; do
  base="$(basename "$file")"
  ts="${base#devnet-}"
  ts="${ts%-cu.json}"
  git show "$BASELINE_COMMIT:$file" | jq -c \
    --arg file "$file" \
    --argjson ts "$ts" \
    '. as $root | $root.scenarios[] | {
      scenario: .name,
      deploy: (.deploy_units // 0),
      execute: (.step_results[0].units // 0),
      total: (.total_units // 0),
      source_file: $file,
      source_ts: $ts,
      network: ($root.network // ""),
      program_id: ($root.program_id // "")
    }' >> "$BASELINE_LINES"
done < <(git ls-tree -r --name-only "$BASELINE_COMMIT" -- five-solana/tests/benchmarks/validator-runs | rg 'devnet-.*-cu.json$' | sort)

# Current head scenarios from workspace files
while IFS= read -r file; do
  base="$(basename "$file")"
  ts="${base#devnet-}"
  ts="${ts%-cu.json}"
  jq -c \
    --arg file "$file" \
    --argjson ts "$ts" \
    '. as $root | $root.scenarios[] | {
      scenario: .name,
      deploy: (.deploy_units // 0),
      execute: (.step_results[0].units // 0),
      total: (.total_units // 0),
      source_file: $file,
      source_ts: $ts,
      network: ($root.network // ""),
      program_id: ($root.program_id // "")
    }' "$file" >> "$HEAD_LINES"
done < <(ls five-solana/tests/benchmarks/validator-runs/devnet-*-cu.json 2>/dev/null | sort)

jq -s '
  reduce .[] as $item ({};
    if (.[$item.scenario] | not) or ($item.source_ts > .[$item.scenario].source_ts)
    then .[$item.scenario] = $item
    else .
    end
  )
' "$BASELINE_LINES" > "$BASELINE_REDUCED"

jq -s '
  reduce .[] as $item ({};
    if (.[$item.scenario] | not) or ($item.source_ts > .[$item.scenario].source_ts)
    then .[$item.scenario] = $item
    else .
    end
  )
' "$HEAD_LINES" > "$HEAD_REDUCED"

GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

jq -n \
  --arg baseline_commit "$BASELINE_COMMIT" \
  --arg head_commit "$HEAD_COMMIT" \
  --arg generated_at "$GENERATED_AT" \
  --slurpfile baseline "$BASELINE_REDUCED" \
  --slurpfile head "$HEAD_REDUCED" '
  def scenarios:
    ["token_full_e2e","external_non_cpi","external_interface_mapping_non_cpi","external_burst_non_cpi","memory_string_heavy","arithmetic_intensive"];

  {
    baseline_commit: $baseline_commit,
    head_commit: $head_commit,
    generated_at: $generated_at,
    scenarios: [
      scenarios[] as $name |
      {
        scenario: $name,
        baseline: ($baseline[0][$name] // null),
        head: ($head[0][$name] // null),
        delta: (
          if (($baseline[0][$name] // null) == null) or (($head[0][$name] // null) == null)
          then null
          else {
            deploy: (($head[0][$name].deploy // 0) - ($baseline[0][$name].deploy // 0)),
            execute: (($head[0][$name].execute // 0) - ($baseline[0][$name].execute // 0)),
            total: (($head[0][$name].total // 0) - ($baseline[0][$name].total // 0))
          }
          end
        )
      }
    ]
  }
' > "$OUT_JSON"

{
  echo "# Baseline vs Head CU Comparison"
  echo
  echo "- Baseline commit: \`$BASELINE_COMMIT\`"
  echo "- Head commit: \`$HEAD_COMMIT\`"
  echo "- Generated at: \`$GENERATED_AT\`"
  echo
  echo "| Scenario | Baseline Deploy | Head Deploy | Delta Deploy | Baseline Execute | Head Execute | Delta Execute | Baseline Total | Head Total | Delta Total |"
  echo "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|"

  jq -r '.scenarios[] | [
      .scenario,
      (.baseline.deploy // "n/a"),
      (.head.deploy // "n/a"),
      (.delta.deploy // "n/a"),
      (.baseline.execute // "n/a"),
      (.head.execute // "n/a"),
      (.delta.execute // "n/a"),
      (.baseline.total // "n/a"),
      (.head.total // "n/a"),
      (.delta.total // "n/a")
    ] | @tsv' "$OUT_JSON" | while IFS=$'\t' read -r s bd hd dd be he de bt ht dt; do
    echo "| $s | $bd | $hd | $dd | $be | $he | $de | $bt | $ht | $dt |"
  done
} > "$OUT_MD"

echo "Wrote: $OUT_JSON"
echo "Wrote: $OUT_MD"
