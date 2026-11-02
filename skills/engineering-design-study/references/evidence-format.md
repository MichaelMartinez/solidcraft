# Evidence manifest v1

The JSON manifest complements the authored journal and raw traces. It is not a replay
engine or proof that referenced files substantiate their claims; review that relationship.
Use the starter asset. Paths are relative to the manifest directory and must stay inside
it. URLs belong in source records, not artifact paths. The renderer escapes supplied text.

Required top-level fields: version=1, title, status (`in_progress`, `awaiting_review`,
`blocked`, `complete`), scope, source_revision, next_experiment, trace_limitations (list),
artifacts (list), experiments (list), checks (list), assumptions (list), blockers (list).
Optional review_gate states an existing user review boundary; do not invent one.

Artifacts: unique id, path, label; optional SHA-256. Hash immutable inputs/receipts/snapshots
when asserting exact identity. A file hash mismatch fails validation. Avoid hashing a
mutable authored journal and then treating every new entry as corruption.

Experiments: id, question, expected, observed, inference, failures, correction,
next_experiment, evidence (artifact IDs). Inference is explicitly labeled, including when
unconfirmed. Evidence links should identify the relevant JSON field or operation in prose.

Checks: name, status (`pass`, `fail`, `unsupported`, `pending`), evidence (artifact IDs),
optional detail and required_for_acceptance (default true). Pass/fail require evidence.
Numerical checks additionally have finite expected, measured, unit, and tolerance with
absolute and/or relative nonnegative values. Relative is a fraction: 0.01% = 0.0001.
The allowed difference is max(absolute, relative * abs(expected)); use an absolute tolerance
for an expected zero. Status must agree with the calculation. Unsupported/pending checks
must not carry invented measurements. Qualitative checks are assertions whose cited evidence
still requires independent review.

Complete requires at least one required check, every required check passing, and no blockers.
Historical failed experiments/checks remain retained but need not be required for current
acceptance. A milestone awaiting review may coexist with unsupported later-phase gates.
Don't equate an evidence bundle's completeness with the whole project being accepted.

The helper refuses overwrite by default. Render a draft for integration into the single
authored journal; do not regenerate over and erase prior experiment entries. Reports and
drafts must live beside the manifest so relative evidence links remain portable.
