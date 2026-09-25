# Pomodoro timer: implementation kickoff

Reader: the owner, who pastes the block below as the first message of the
implementation session.

```text
Implement rust-timer (the `pomodoro` terminal timer) from docs/plan/. This message
authorises implementation within the pack's scope; it adds no requirements.

Read in this order: docs/plan/decision-record.md, docs/plan/behaviour-contract.md,
docs/plan/plan.md. The installed skills decide structure, process and evidence;
the pack decides behaviour. Where they conflict on structure, the skill wins and
you report the conflict.

Every slice:
1. Apply rust-quality-baseline at the start of the slice; iterate with
   `cargo xtask check --fast`; the full check runs before the slice is called done.
2. Add the behaviour, and consolidate any shape that now has three copies with one
   responsibility.
3. Evidence for the slice is a review of at most 40 lines, the commit hash and
   the CI run URL. Logs, tree dumps, hashes and captured output stay out of the
   repository.
4. Commit at the slice boundary on the feature branch feat/pomodoro.

After the first slice: push the feature branch and open the pull request, so hosted
CI is the proof from the second slice on.

After the second slice: stop. Run `cargo xtask gates` and paste its output
unedited, then run the plan's ownership-table checks O1 to O7 and report what each
one prints. Wait for review before slice three.

Ask when a decision blocks dependent work; continue independent slices meanwhile.
At each handoff report completed slices, acceptance ids covered, exact commands and
exit codes, open gates and the next action. Do not merge, publish or deploy.
```
