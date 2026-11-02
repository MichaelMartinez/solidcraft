# Running the helper and closing out

The optional standard-library helper validates paths, hashes, cited artifact IDs and numerical
claims, and renders an HTML report and a journal **draft** without touching the native application:

```bash
python3 scripts/study_report.py check /path/to/study.json
python3 scripts/study_report.py render /path/to/study.json --html /path/to/review.html --journal /path/to/journal-draft.md
```

Commands are relative to this skill directory. Existing output files require `--replace`; never
use that on the authored journal. Reports and drafts live beside the manifest so relative evidence
links stay portable.

For app-specific collection and Faenad closeout read [faenad.md](faenad.md); for Chili3D also read
[chili3d.md](chili3d.md). Do not load the CAD reference for other work.

The command block above sat in `SKILL.md` until this copy was installed in SolidCraft, whose
`cargo xtask skills` caps a new skill at 3,600 characters; nothing was removed, only moved.
