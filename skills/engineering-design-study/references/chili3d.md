# Chili3D study collection

Re-read nested checkout instructions before code changes. Use the in-app local assistant
and existing core/WASM operations. Direct reference geometry may use run_program;
editable design features should use the native parametric history. Do not add an external
agent bridge or assembly system merely to conduct a study.

Identify document and source revision explicitly. Verify a linked copy before mutating.
Do not select the first CAD browser tab for unattended work. For the managed browser,
follow docs/operations/runbooks/faenad-live-browser.md: attach-only CDP and acquire the
shared lock. Recheck current ports and runtime locations instead of hardcoding scratch paths.

Capture actual assistant tool calls and request diagnostics separately from its prose.
A successful native query can contradict the narrative. The Pi case retained an incorrect
rider diagnosis and appended an explicit correction with source evidence.

Measure native local geometry and visible world geometry separately; a transform can
change display bounds without editing a feature. Inspect mounting centers/edge geometry,
not just overall size. Measure the actual node shape independently of the assistant tool
response. Native validity/positive volume/shell closure do not prove every topology check.
Label unsupported boundary/manifold checks and keep export guards until exact conversion passes.

For the current Phase 2 roadmap use 0.001 mm dimensions and 0.01% relative volume, with
format-specific exceptions explicitly justified. Float values may differ after reopen;
compare tolerances, while unchanged source server snapshots should retain their exact hashes.
These thresholds are roadmap choices, not generic defaults for all studies.

Use server save receipts and fresh native reopen. A screenshot is supporting evidence:
check ancestor visibility before capture (the Pi study initially hid its root folder),
wait for rendering, inspect the actual image, and save camera/document provenance. Keeping
children visible does not make them visible under a hidden parent.

Official PDFs can be retained and rendered for the assistant's image input; do not call
that native PDF/CAD import. Recheck live import formats. Use simplified solids when allowed,
with sourced dimensions and provisional bounds explicitly distinguished.

The Pi study is paused after milestone 1, document `pi4-enclosure-study-20260926`.
Read its current journal and user review before advancing. Native reference solidity does
not close the full Phase 2 geometry gates. Scheduled-backup acceptance is a separate item.

Failure-derived tool lesson: run_program `id`/`name` belong beside `method`, outside `args`.
Edit operations consume input refs; query the result id. Malformed metadata now fails before
mutation (nested 6b8d69f). Repeated speculative kernel explanations are not diagnosis.

Failure-derived lessons from [F-0441](/mnt/vault4t/storage/chili3d/application-20261022/evidence/faenad-agent-restart-20261002/preservation.json):

- `plugin_faenad_map_frame`'s `insideLocalBounds` and `nearFaces` describe a bounding box, not material membership. The v3 Unistrut plate's mapped center points at local z=0/6 lie on a through-bore axis: independent native `solid.containsPoint` returns false for both, despite the bounding-box face labels. Query native material/faces in the declared local frame before asserting a point is on a solid or contact surface.
- Bind observer checks to the actual exposed tool contract. The native review tool is `check_design`; a synthetic `design_review` test fixture concealed a live observer mismatch. Exercise the real registered handler and live receipts for both failed native checks and requested review pauses.
- Separate agent failures from collection defects. An HTTP-origin UUID failure in the independent perturbation collector was repaired on exact saved copies, with source hashes and a successful editable-plate control; no agent attempt was repeated or replaced. Browser code cannot assume `crypto.randomUUID` exists on the operator HTTP origin.
