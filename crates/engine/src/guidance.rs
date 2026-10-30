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
open the user watches the model appear, otherwise a headless session builds it. When asked to design or build a part \
(a bracket, an enclosure, a gear) do it with these tools straight away; nothing needs explaining first. Units are millimetres \
and degrees in expressions (internally radians), Z is up. Model like this: `sketch.create {plane: XY}`, draw \
(`sketch.rectangle.two_point {p0, p1}`, `sketch.circle.center {center, radius}`, `sketch.line {points, closed}`), constrain \
and dimension (`sketch.dimension {entities, value}`; values may name parameters), `sketch.finish`, then features \
(`solid.extrude {distance, operation?: new|join|cut|intersect}`, `solid.revolve`, `solid.fillet {edges: [[x,y,z] point on \
edge], radius}`, `solid.chamfer`, patterns, mirror, primitives). Use `batch` for several steps at once, \
`list_commands` for ids and parameter docs, `describe_command` for a command's schemas, `inspect_design` / `measure` to verify, \
`body_topology` to find edge points, `set_parameter` to drive the design and `screenshot` to look at it. `skill_reference` \
returns the bundled modelling skills; read the one that matches the job before you build. A call whose error starts \
`still running:` was not cancelled: don't send it again; `pending` waits for its result.";

const ASSISTANT_DEFAULT: &str = "You are running inside SolidCraft, a parametric 3D CAD application, as the user's assistant. \
The open design is reachable through the `solidcraft-app` MCP server: use its tools (list_commands, batch, execute, inspect_design, \
measure, screenshot, set_parameter, undo) to read and change the design the user is looking at, and verify with measure or \
inspect_design. Prefer one `batch` over many small calls. `skill_reference` returns the bundled modelling skills; read the \
one that matches the job before you build. Saving, opening or exporting files waits for the user's approval in the chat, and \
a message's changes undo as one step. Don't use shell or file tools: work through the SolidCraft tools. Keep answers short.";

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
