---
name: engineering-design-study
description: Run and document iterative engineering studies with source provenance, independent verification, retained failures, a study journal and a visual review report. Use for substantial design, integration or operational experiments and their continuation; scale down for routine edits.
---

# Engineering design study

Keep one evolving object or workflow and one journal connecting intent, source state,
experiments, observations, corrections and the next experiment. The report is a view
of that evidence. Assistant narration is a claim to verify, not verification itself.

## Establish the study

- Read the current task and continuity before choosing an experiment. Preserve the
  user's milestones, scope and review boundaries. This skill does not create new
  approval gates, authorize external actions, or turn a planning task into execution.
- Pin source revisions and authoritative references. Preserve original artifacts;
  make a linked study copy when experimentation would otherwise alter an original.
- Record units, coordinate frames, assumptions and acceptance tolerances where relevant.
  Separate source facts, measurements, selected defaults and unresolved dimensions.
- Use existing native capabilities first: keep native/editable state authoritative;
  exchange files and screenshots have narrower roles.

## Experiment and learn

Record the question and expected observable result before editing. Exercise the actual
workflow (including its local assistant when studying that assistant), then inspect the
result independently: input, tool results, native revision, measurements, annotated views.
Capture traces where available; never promise a replay that was not retained.

On failure, retain evidence of the failure and whether prior state survived; separate
observed facts from possible causes. Stop unproductive retries to inspect the actual input
and state. Repair only the demonstrated gap, keep a small regression when warranted, and
repeat the same experiment. Correct misleading journal claims explicitly; never silently
rewrite them into a successful history.

Verify the invariants the task calls for: dimensions/volume, counts, service behavior,
persistence, export or rollback. Use declared tolerances; byte equality suits unchanged
source snapshots, not regenerated geometry. Unsupported checks stay unsupported and block
only their acceptance. A screenshot or a healthy endpoint alone proves no workflow.

## Retain and review

- Maintain one authored journal: dated entries with intent, source revision,
  expected/observed results, evidence links, failures, fixes and the next experiment.
- Keep reproduction scripts in durable storage before closeout. Label archived scratch
  scripts as historical, with their failures and side effects; never expose them as an
  unattended replay. Automation must name an explicit target and check its revision
  before mutation.
- Produce a concise visual review: verified results, assumptions, unresolved questions
  and the exact remaining acceptance gates, with actual views and labeled diagrams.
  Keep source precision distinct from modeled/physical-fit confidence.
- Pause only at the user's review boundaries; otherwise continue authorized work.
  State milestone completion separately from overall acceptance.

For structured output, copy [assets/study.json](assets/study.json) and follow
[references/evidence-format.md](references/evidence-format.md). The helper and the domain
closeout notes are in [references/reporting.md](references/reporting.md).
