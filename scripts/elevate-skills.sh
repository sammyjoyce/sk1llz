#!/usr/bin/env bash
#
# elevate-skills.sh — Loop over every skill in the repo and use Codex CLI
# to evaluate it with the skill-judge rubric, then rewrite it to A+ quality.
#
# Usage:
#   ./scripts/elevate-skills.sh                # Run all skills (sequential)
#   ./scripts/elevate-skills.sh --dry-run      # Print prompts without executing
#   ./scripts/elevate-skills.sh --filter rust  # Only skills matching "rust"
#   ./scripts/elevate-skills.sh --model gpt-5.4 --reasoning-effort high
#                                              # Override Codex defaults
#   ./scripts/elevate-skills.sh --concurrency 4
#                                              # Run up to 4 agents in parallel
#   ./scripts/elevate-skills.sh --resume-from domains/trading/swing-trading
#                                              # Skip skills until this path
#
# Requirements:
#   - codex CLI installed and authenticated
#   - Run from the repo root
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG_DIR="$REPO_ROOT/.skill-elevation-logs"
mkdir -p "$LOG_DIR"

# ── Args ──────────────────────────────────────────────────────────────────────
DRY_RUN=false
FILTER=""
RESUME_FROM=""
CONCURRENCY=1      # how many parallel agents (increase with caution)
SKIP_TEMPLATE=true # skip the meta/skill-template placeholder
MODEL=""
REASONING_EFFORT=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--dry-run)
		DRY_RUN=true
		shift
		;;
	--filter)
		FILTER="$2"
		shift 2
		;;
	--resume-from)
		RESUME_FROM="$2"
		shift 2
		;;
	--model)
		MODEL="$2"
		shift 2
		;;
	--reasoning-effort)
		REASONING_EFFORT="$2"
		shift 2
		;;
	--mode)
		echo "--mode was amp-specific and is no longer supported; use --model or rely on Codex config defaults." >&2
		exit 1
		;;
	--concurrency)
		CONCURRENCY="$2"
		shift 2
		;;
	*)
		echo "Unknown arg: $1"
		exit 1
		;;
	esac
done

if ! $DRY_RUN && ! command -v codex >/dev/null 2>&1; then
	echo "codex CLI not found in PATH" >&2
	exit 1
fi

# ── Discover skills ───────────────────────────────────────────────────────────
mapfile -t SKILL_PATHS < <(
	find "$REPO_ROOT" -name 'SKILL.md' -not -path '*/.git/*' | shuf
)

echo "Found ${#SKILL_PATHS[@]} skills in repo."

# ── Build the prompt ──────────────────────────────────────────────────────────
build_prompt() {
	local skill_path="$1"
	local rel_path="${skill_path#$REPO_ROOT/}"
	local skill_dir="$(dirname "$rel_path")"
	local skill_name
	skill_name=$(grep '^name:' "$skill_path" | head -1 | sed 's/^name: *//')

	cat <<PROMPT
You are performing a focused skill elevation task. Your ONLY job is to rewrite
the file at "$rel_path" so it scores A+ (108+/120) on the skill-judge rubric.

Do NOT touch any other file. Do NOT create new files outside "$skill_dir/".
When finished, stop.

## Step 1 — Evaluate (do this silently, don't print the full report)

Load the skill-judge skill. Read "$rel_path" completely.
Score it across all 8 dimensions (D1-D8, 120 pts total).
Identify the weakest dimensions.

## Step 2 — Research the subject

Before rewriting, use web_search to find genuinely expert-level knowledge about
${skill_name:-this subject} that Claude would NOT already know:
- Non-obvious trade-offs, failure modes, and edge cases
- Decision heuristics practitioners learn only through experience
- Specific anti-patterns with concrete consequences
- Counterintuitive best practices
- Numbers, thresholds, and parameters that matter

This research is the most important step. The #1 problem with the current skill
is LOW KNOWLEDGE DELTA — it mostly restates things Claude already knows.

## Step 3 — Rewrite to A+

Rewrite "$rel_path" applying these specific fixes:

### D1 Knowledge Delta (target: 17+/20)
- DELETE every sentence Claude already knows (standard patterns, basic concepts,
  common best practices, textbook definitions)
- REPLACE with expert-only knowledge from your research
- Every paragraph must pass the test: "Would a 10-year practitioner say
  'yes, this took me years to learn'?"

### D2 Mindset + Procedures (target: 13+/15)
- Add "Before doing X, ask yourself..." thinking frameworks
- Include domain-specific procedures Claude wouldn't know (not generic
  file-open-edit-save steps)

### D3 Anti-Patterns (target: 12+/15)
- Every NEVER item must have: the wrong path, WHY it's seductive,
  the concrete consequence, and the correct alternative
- Format: "NEVER do X because [non-obvious reason]. Instead do Y."

### D4 Spec Compliance (target: 14+/15)
- Ensure frontmatter has name (lowercase, hyphens, ≤64 chars) and description
- Description must answer: WHAT does it do, WHEN to use it, plus trigger KEYWORDS
- Include explicit "Use when..." scenarios in the description

### D5 Progressive Disclosure (target: 12+/15)
- Keep SKILL.md under 200 lines (ideally 100-150)
- Move lengthy code examples to a references/ subdirectory
- Add MANDATORY loading triggers: "Before doing X, READ references/Y.md"
- Add "Do NOT load Z.md for this task" guidance

### D6 Freedom Calibration (target: 13+/15)
- Creative/design tasks → high freedom (principles, not rigid steps)
- Fragile/format tasks → low freedom (exact scripts, specific parameters)
- Match constraint level to consequence of mistakes

### D7 Pattern Recognition (target: 8+/10)
- Choose the right pattern: Mindset (~50 lines), Philosophy (~150),
  Navigation (~30), Process (~200), or Tool (~300)
- Don't force every skill into the same template

### D8 Practical Usability (target: 13+/15)
- Include decision trees for multi-path scenarios
- Add fallback strategies when primary approach fails
- Cover realistic edge cases

## Step 4 — Self-check

After rewriting, re-score the skill mentally. If any dimension is below its
target, iterate. Only stop when you're confident the total is 108+/120.

## Constraints
- ONLY modify files inside "$skill_dir/"
- Do NOT modify or read any other skill
- Do NOT create CHANGELOG, README, or meta files about the skill
- Commit nothing — the user will review and commit
- Exit when done
PROMPT
}

# ── Pre-filter into work queue ────────────────────────────────────────────────
skipping=true
[[ -z "$RESUME_FROM" ]] && skipping=false

skipped=0
QUEUE=()

for skill_path in "${SKILL_PATHS[@]}"; do
	rel_path="${skill_path#$REPO_ROOT/}"

	# ── Skip template ───────────────────────────────────────────────────────
	if $SKIP_TEMPLATE && [[ "$rel_path" == *"meta/skill-template"* ]]; then
		echo "[SKIP] $rel_path (template)"
		((skipped++)) || true
		continue
	fi

	# ── Resume support ──────────────────────────────────────────────────────
	if $skipping; then
		if [[ "$rel_path" == *"$RESUME_FROM"* ]]; then
			skipping=false
		else
			echo "[SKIP] $rel_path (before resume point)"
			((skipped++)) || true
			continue
		fi
	fi

	# ── Filter support ──────────────────────────────────────────────────────
	if [[ -n "$FILTER" ]] && [[ "$rel_path" != *"$FILTER"* ]]; then
		((skipped++)) || true
		continue
	fi

	QUEUE+=("$skill_path")
done

queue_total=${#QUEUE[@]}
echo ""
echo "Queued $queue_total skills (concurrency=$CONCURRENCY, model=${MODEL:-config-default}, reasoning=${REASONING_EFFORT:-config-default})."

# ── Per-job status tracking (subshells can't share counters) ──────────────────
STATUS_DIR="$(mktemp -d "$LOG_DIR/.status.XXXXXX")"
trap 'rm -rf "$STATUS_DIR"' EXIT

run_one() {
	local idx="$1"
	local skill_path="$2"
	local rel_path="${skill_path#$REPO_ROOT/}"
	local skill_dir
	skill_dir="$(dirname "$rel_path")"
	local prompt log_file
	local -a codex_cmd
	prompt="$(build_prompt "$skill_path")"
	log_file="$LOG_DIR/$(echo "$skill_dir" | tr '/' '_').log"

	# Build base command array
	build_cmd() {
		local model_override="$1"
		local -a cmd=(
			codex exec
			--full-auto
			--ephemeral
			--color never
			--skip-git-repo-check
			-C "$REPO_ROOT"
			-c 'web_search="live"'
		)
		if [[ -n "$model_override" ]]; then
			cmd+=(--model "$model_override")
		elif [[ -n "$MODEL" ]]; then
			cmd+=(--model "$MODEL")
		fi
		if [[ -n "$REASONING_EFFORT" ]]; then
			cmd+=(-c "model_reasoning_effort=\"$REASONING_EFFORT\"")
		fi
		cmd+=(-)
		printf '%s\n' "${cmd[@]}"
	}

	echo "[$idx/$queue_total] → $rel_path"

	if $DRY_RUN; then
		{
			echo "[DRY RUN] $rel_path (${#prompt} chars)"
			echo "--- Prompt preview (first 5 lines) ---"
			echo "$prompt" | head -5
			echo "..."
		} >"$log_file"
		: >"$STATUS_DIR/ok.$idx"
		return 0
	fi

	# First attempt with primary model
	local -a codex_cmd
	mapfile -t codex_cmd < <(build_cmd "")

	if printf '%s\n' "$prompt" | "${codex_cmd[@]}" >"$log_file" 2>&1; then
		echo "[$idx/$queue_total] ✓ $rel_path"
		: >"$STATUS_DIR/ok.$idx"
	else
		local rc=$?
		# Check if it's a context window error and we haven't already tried gpt-5.4
		if grep -q "out of room in the model's context window\|context window\|tokens used" "$log_file" 2>/dev/null &&
			[[ "${MODEL:-}" != "gpt-5.4" ]]; then
			echo "[$idx/$queue_total] ⚠ $rel_path (context limit, retrying with gpt-5.4)..."
			{

				echo ""

				echo "=== RETRY WITH GPT-5.4 ==="

				echo ""
			} >>"$log_file"

			local -a retry_cmd
			mapfile -t retry_cmd < <(build_cmd "gpt-5.4")

			if printf '%s\n' "$prompt" | "${retry_cmd[@]}" >>"$log_file" 2>&1; then
				echo "[$idx/$queue_total] ✓ $rel_path (retry succeeded)"
				: >"$STATUS_DIR/ok.$idx"
			else
				local retry_rc=$?
				echo "[$idx/$queue_total] ✗ $rel_path (retry failed, exit $retry_rc) — see $log_file"
				: >"$STATUS_DIR/fail.$idx"
			fi
		else
			echo "[$idx/$queue_total] ✗ $rel_path (exit $rc) — see $log_file"
			: >"$STATUS_DIR/fail.$idx"
		fi
	fi
}

# ── Dispatch with bounded concurrency ─────────────────────────────────────────
idx=0
for skill_path in "${QUEUE[@]}"; do
	((idx++)) || true

	# Block until a worker slot is free
	while (($(jobs -r -p | wc -l) >= CONCURRENCY)); do
		wait -n 2>/dev/null || true
	done

	run_one "$idx" "$skill_path" &

	# Small stagger so parallel launches don't all hit rate limits at once
	$DRY_RUN || sleep 1
done

# Drain remaining workers
wait

completed=$(find "$STATUS_DIR" -name 'ok.*' -printf '.' | wc -c)
failed=$(find "$STATUS_DIR" -name 'fail.*' -printf '.' | wc -c)

# ── Summary ───────────────────────────────────────────────────────────────────
echo ""
echo "══════════════════════════════════════════════════════════════════"
echo "COMPLETE"
echo "  Elevated: $completed"
echo "  Failed:   $failed"
echo "  Skipped:  $skipped"
echo "  Logs:     $LOG_DIR/"
echo "══════════════════════════════════════════════════════════════════"
