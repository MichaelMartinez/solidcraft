//! The instructions SolidCraft gives an agent, and the user's copy of them.
//!
//! Releases carry both texts compiled in. A file in `<data dir>/instructions/` replaces one
//! without rebuilding the app: `mcp.md` for the MCP `initialize` instructions, `assistant.md`
//! for the note the Assistant prepends. A file that is missing, unreadable, larger than
//! [`MAX_BYTES`] or blank keeps the compiled text, so a broken file never leaves an agent
//! without instructions.
//!
//! When a replacement is read is the caller's business: every call re-reads the file, but the
//! protocols differ. An MCP client gets the instructions once per connection, an ACP session
//! gets the note once per session, and an API endpoint gets it on every request. See
//! `docs/interaction.md`.

use std::path::{Path, PathBuf};

/// Largest prompt file accepted, in bytes. The compiled texts are about 1 KiB.
pub const MAX_BYTES: u64 = 64 * 1024;

const MCP_DEFAULT: &str = "SolidCraft is an independent parametric 3D CAD app, and these tools drive it: when its window is \
open the user watches the model appear, otherwise a headless session builds it. When asked to design or build a part, do it \
with these tools straight away. Units are millimetres and degrees in expressions, Z is up.\n\n\
Start: for anything beyond one primitive, call `skill_reference {\"skill\":\"solidcraft-modeling\"}` first (command shapes, \
ids, plane axes, worked examples and the known traps of this version; `file: \"SKILL.md\"` has the workflow). Then \
`inspect_design {\"measure\":false,\"sketches\":false}` to see what is already there.\n\n\
Plan: work the numbers out first (the parameters, the expected volume and bounding box after each stage, the world point you \
will use for each edge or face pick). Then build in a few `batch` calls (all-or-nothing; the reply names the failing step; \
`save`, `measure` and `set_parameter` can be steps). Use `verbose: false`. Name every feature. `list_commands {\"filter\": word}` \
gives ids and one-line parameter docs (never unfiltered: it also takes prefix, panel, compact: true and limit); `describe_command` gives one command's docs and schemas.\n\n\
Routes: a plate, block, boss, standoff or hole is cheapest as a primitive with expression fields (`solid.box`, `solid.cylinder`, \
`solid.hole`, `solid.fillet`, `solid.shell`, patterns and mirror), no sketch. Sketch when the profile is not a box or a circle. \
Fillet outside edges before shelling, build inside features after the shell, then the holes and cuts through walls. Give a blind \
hole or a hole through one wall of a hollow part its depth.\n\n\
Parametric: a size or position that must follow a parameter is an expression in a feature field (`\"L/2 - 10\"`), a sketch \
dimension to `\"origin\"` or an edge, or a pattern spacing; a plain number never follows a parameter. Every sketch must end at \
dof 0 (`sketch.solve` reports it). A pick (`edges: [[x,y,z]]`, `faces: [[x,y,z]]`) is a number that snaps to the nearest edge \
or face and follows it when parameters change.\n\n\
Verify: `measure {}` after each stage and compare volume and bbox with your numbers; `ok: true` alone proves nothing: a cut \
that leaves the volume unchanged has missed, a join that adds a body did not touch. Positions of holes and bosses come from \
`body_topology` (cylinder origin, axis and radius), not from the volume or the centre of mass of a symmetric part. To check \
that the design follows its parameters, `check_parameters {\"set\": {name: value}}` tries the values and puts the design \
back (volume, errors, cylinder axes), or change them in one `batch` with `coalesce_parameters: true` (one rebuild), `measure`, \
then set them back the same way before saving.\n\n\
Fix, don't rebuild: a wrong number in a feature is changed with `timeline.edit` or `parameters.change` on its input \
(`parameters.list {\"filter\":\"feature\"}` names them); `edit.checkpoint` and `edit.restore_checkpoint` go back exactly.\n\n\
A call whose error starts `still running:` was not cancelled: don't send it again; `pending` waits for its result.";

const ASSISTANT_DEFAULT: &str = "You are running inside SolidCraft, a parametric 3D CAD application, as the user's assistant. \
The open design is reachable through the `solidcraft-app` MCP server: use its tools (list_commands, batch, execute, inspect_design, \
measure, screenshot, set_parameter, undo, skill_reference) to read and change the design the user is looking at, and verify with \
measure or inspect_design. Before building anything beyond one primitive, read `skill_reference` for `solidcraft-modeling` (its \
SKILL.md has the workflow, its reference.md the command shapes and this version's traps), work the numbers out first, and after \
each stage compare `measure` with them. Prefer one `batch` over many small calls. Saving, opening or exporting files waits for the user's approval in \
the chat, and each message allows a limited number of changes. Don't use shell or file tools: work through the SolidCraft tools. \
Keep answers short.";

/// The compiled MCP instructions: what a release ships, and what the tool-name test checks.
pub fn default_instructions() -> &'static str {
    MCP_DEFAULT
}

/// The compiled Assistant note, untouched by any file on this machine.
pub fn default_assistant_note() -> &'static str {
    ASSISTANT_DEFAULT
}

/// The MCP instructions to send on this machine: the user's `mcp.md` when there is a usable one.
pub fn instructions() -> String {
    load(instructions_dir(), "mcp.md", MCP_DEFAULT)
}

/// The Assistant note to send on this machine: the user's `assistant.md` when there is a usable one.
pub fn assistant_note() -> String {
    load(instructions_dir(), "assistant.md", ASSISTANT_DEFAULT)
}

/// `<data dir>/instructions`, the folder a user's prompt files live in.
pub fn instructions_dir() -> Option<PathBuf> {
    crate::recovery::data_dir().map(|dir| dir.join("instructions"))
}

fn load(dir: Option<PathBuf>, file: &str, default: &'static str) -> String {
    dir.and_then(|dir| read(&dir.join(file))).unwrap_or_else(|| default.to_string())
}

/// A prompt file's text, or `None` when it is absent, not a regular file, too large, not UTF-8
/// or blank.
fn read(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    if text.trim().is_empty() { None } else { Some(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("solidcraft-guidance-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_file_replaces_the_default_and_a_missing_one_keeps_it() {
        let dir = scratch("file");
        std::fs::write(dir.join("mcp.md"), "Use the tools.").unwrap();
        assert_eq!(load(Some(dir.clone()), "mcp.md", MCP_DEFAULT), "Use the tools.");
        assert_eq!(load(Some(dir.clone()), "assistant.md", ASSISTANT_DEFAULT), ASSISTANT_DEFAULT);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_blank_oversized_or_binary_file_keeps_the_default() {
        let dir = scratch("unusable");
        std::fs::write(dir.join("mcp.md"), " \n\t").unwrap();
        assert_eq!(load(Some(dir.clone()), "mcp.md", MCP_DEFAULT), MCP_DEFAULT, "a blank file is not an instruction");
        std::fs::write(dir.join("mcp.md"), vec![b'x'; MAX_BYTES as usize + 1]).unwrap();
        assert_eq!(load(Some(dir.clone()), "mcp.md", MCP_DEFAULT), MCP_DEFAULT, "an oversized file is refused");
        std::fs::write(dir.join("mcp.md"), [0xff, 0xfe, 0x00]).unwrap();
        assert_eq!(load(Some(dir.clone()), "mcp.md", MCP_DEFAULT), MCP_DEFAULT, "a file that is not text is refused");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn with_no_folder_the_default_is_used() {
        assert_eq!(load(None, "mcp.md", MCP_DEFAULT), MCP_DEFAULT);
    }

    #[test]
    fn the_defaults_say_where_the_skills_are() {
        assert!(default_instructions().contains("skill_reference"));
        assert!(default_assistant_note().contains("skill_reference"));
    }
}
