# Using SolidCraft: which surface, when

SolidCraft can be driven from the window, from its own Assistant panel, from an agent running
outside the app, and from a script with no window at all. They are not four products: they are
four ways to reach the same engine commands. This page says which one to use for what, what each
one can and cannot do, and where the instructions an agent follows actually come from.

The per-surface references stay the place for details: [MCP](mcp.md) for the tool list and the
server's rules, [Agent chat](agent-chat.md) for the panel, [Control protocol](control-protocol.md)
for the JSON-line methods.

## One command surface, four transports

Every change an agent makes is an engine command — `solid.extrude`, `sketch.dimension`,
`model.fillet` — exactly the ones the toolbar and the palette run. There is no private geometry
API for agents, which is why a model built by an agent replays in the window, in a script and in
a regression test alike.

| Transport | Who holds the tool loop | What the user sees |
|---|---|---|
| Window | The person | The model |
| Assistant panel | An agent the app started (ACP program or API endpoint) | The model, changing as the agent works |
| MCP server, bridged | An agent you started elsewhere | The model in the app's window, if one is open |
| MCP server, headless | An agent you started elsewhere | Nothing; pictures come back as files |
| `solidcraft-cli run`/`eval`/`snapshot` | A script | Nothing; `snapshot` writes an image |
| Control channel | Any local program or test | Whatever the app does |

Transport choice is not a technical detail: it decides whether anything is reviewing the agent.

## The four ways people actually work

### The window alone

Untouched. Sketch, dimension, extrude, look at it. Still the only surface that is good at picking
one specific face on a busy body, or deciding whether a fillet looks right.

### The window plus the Assistant — one window

Type in the panel; the app starts an agent and hands it the `solidcraft-app` MCP server, which is
`solidcraft-cli mcp` bridged back into the running app. Changes appear in the same window as tool
cards, and everything changed while answering one message is **one undo step** named after the
message. This is the only surface with an approval gate: Allow / Always allow / Deny per command,
with file actions asking even when modelling does not.

What it cannot reach: the interface itself, the shell, files (its own file tools are turned off),
and the four bundled skills are the whole of its written guidance.

### An agent outside the app — two windows

You run the app (it listens on `127.0.0.1:27878` at start-up by default), your agent runs
`solidcraft-cli mcp`, and the server finds the port through `control/current.port` and reads the
launch's token. From your side there are two windows: the one you are talking in, and the
SolidCraft window. **The SolidCraft window is not the driver.** It is the viewport and the undo
stack; the conversation is where the work happens.

The same server runs headless when no app is listening (`--headless` forces that). Then there is
no window and the agent's only view of the model is `screenshot`, `measure` and `inspect_design`.

An external agent brings its own shell, scripts and skills, so this is the only surface that can
follow a helper script, compare a STEP export against an oracle, or read a skill collection from
disk. It also has no approval gate: the folder allowlist in [MCP](mcp.md#which-files-the-agent-can-reach)
and the undo stack are the whole of the protection.

### Scripts — no window

`solidcraft-cli run design.json --out part.step`, `eval`, `snapshot`, `commands`. Deterministic,
fast, no window, no agent. For CI, corpus runs and regression oracles. Not interactive, and it
does not use the app's session, so it cannot see a design that is merely open.

## Choosing

| The job | Use | Why |
|---|---|---|
| Judge a shape, pick one face, nudge a dimension | Window | Agents are unreliable at "does this look right" |
| A small change while you are looking at the model | Assistant | Context is free: the view, the selection, the picker |
| Anything that needs approval before it lands | Assistant | The only gate there is |
| A multi-step build with verification | External agent, bridged | Shell, scripts, skills, measurement loops |
| The same build, twice, with no user | Scripts | Repeatable and reviewable |
| A regression after a code change | Scripts, or the test suite | Compares numbers, not pictures |
| Exploring what the command set can do | `list_commands`, `describe_command` | Both agents and people read the same catalog |

## What actually differs

|  | Window | Assistant | External agent | Script |
|---|---|---|---|---|
| Undo | Per action | One per message | Per command (a failed `batch` returns to its checkpoint) | Not kept |
| Approval gate | n/a | Yes | No | No |
| Shell and file tools | n/a | Off | The agent's own | n/a |
| Files it can reach | Everywhere you can | The MCP allowlist | The MCP allowlist | Paths you name |
| Written guidance | None | 4 bundled skills via `skill_reference` | Whatever the agent has, plus the 4 if it calls the tool | None |
| Sees the model | Always | Always | Only if bridged to a window | `snapshot` only |
| Failure recovery | Undo | Undo one step | `batch` rollback, or undo | Exit code |

Two asymmetries fall out of that table, and they are the important ones:

- **The surface with the approval gate is the one without a shell.** The Assistant is the safe
  place and the constrained one; the external agent is the capable one and the unguarded one.
- **The skills exist, but nothing points at them.** `skill_reference` serves the four bundled
  skills to any agent, in-app or external, and neither prompt mentions that the tool exists.
  `prompts/list` is empty, so there is no discovery path at all: an agent finds the skills only
  by reading the tool list closely.

## What is missing

Ordered by how much they cost a user per day.

1. **No session, so no handoff.** Intent does not survive a surface change. Start a design in the
   chat, finish it in the window, come back: the agent re-derives everything from
   `inspect_design`, because nothing carries "we were building a bracket for these holes".
   *Proposal:* a design-scoped session note both the chat and MCP read, and that the agent may
   update.
2. **No review surface outside the chat.** The Assistant shows tool cards and one undo step. An
   external agent leaves only the timeline and the undo stack: there is no "what did this turn
   change" to read, compare or reject. This is [#3382](https://github.com/tangency-labs/solidcraft/issues/3382)
   from the other side — the agent needs feature notices, the user needs the diff.
3. **No persistent guidance.** Two hard-coded prompts (below), four skills, and no place for the
   user's own rules. [#8220](https://github.com/tangency-labs/solidcraft/issues/8220) asks for
   user-authored modelling rules next to the bundled ones; they would also be the obvious home
   for a community skill collection.
4. **No way to see the instructions.** They can be replaced by a file, but nothing in the app
   shows the text in force. See [when a changed prompt takes effect](#when-a-changed-prompt-takes-effect).
5. **No provenance.** The design does not record what an agent did, under which agent, in which
   session. In an app whose whole selling point is that an agent can model, "who made this
   feature" is a question the file cannot answer.

## Where the instructions come from

There are exactly two prompts in the product. Each has a compiled default and a file that
replaces it:

| Prompt | Default | Replaced by | Sent to |
|---|---|---|---|
| MCP instructions | `crates/engine/src/guidance.rs` | `<data>/instructions/mcp.md` | Every MCP client, once per connection, as `initialize.result.instructions`, with the backend appended |
| Assistant note | the same file | `<data>/instructions/assistant.md` | The Assistant: `_meta.systemPrompt.append` once per ACP session, or a `system` message on every API request |

`<data>` is SolidCraft's per-user data folder, the parent of the plugins folder:
`~/Library/Application Support/SolidCraft` on macOS, `%LOCALAPPDATA%\SolidCraft` on Windows,
`$XDG_DATA_HOME/solidcraft` on Linux. So a custom MCP instruction lives at
`~/Library/Application Support/SolidCraft/instructions/mcp.md`, for example.

A replacement is used when it is a regular file of at most 64 KiB that is UTF-8 and not blank.
Anything else — missing, unreadable, oversized, blank — keeps the compiled default, so a broken
file never leaves an agent without instructions. Nothing validates a replacement: it is the
user's text, and it may name tools that no longer exist. `instructions_name_only_real_tools`
checks the compiled default only.

Everything else an agent reads is either tool documentation or skill text:

- `tools/list` descriptions, and `list_commands`/`describe_command` parameter docs, come from the
  command registry.
- `skill_reference` returns the four `skills/` documents, byte for byte
  (`crates/mcp/src/skill_reference.rs`).
- `resources/read` serves `solidcraft://design` and `solidcraft://commands`.
- `prompts/list` answers `{"prompts": []}` and the server does not advertise the `prompts`
  capability. MCP's prompt feature is unimplemented, not merely unused.

An agent's own system prompt — Claude Code's, Codex's, an endpoint's — is never read or changed
here. The Assistant note is appended to it, not substituted for it.

## When a changed prompt takes effect

Every call re-reads the file, but the protocols decide how often a call happens, so "hot" stops
short of "immediately everywhere":

| Surface | Takes effect |
|---|---|
| API-endpoint agent in the Assistant | Next request, so the next message |
| ACP agent in the Assistant (Claude Code and friends) | Next session; the note is appended at `session/new` |
| External MCP client | Next connection, or a restart of the MCP server; `initialize` is sent once |

## What is still missing here

- **No page shows the text in force.** There is no Preferences entry for the prompts, so a user
  has to know the folder exists. That page is the obvious next patch.
- **The skills are still compiled in.** `skill_reference` reads `skills/` at build time, so a
  community collection, or the user's own notes, still needs a rebuild. Prompts and skills should
  load through the same mechanism.

## See also

- [MCP](mcp.md): the tools, the folder rules, bridged versus headless.
- [Agent chat](agent-chat.md): the panel, permissions, what the Assistant can and cannot do.
- [Control protocol](control-protocol.md): the JSON-line surface the app itself is driven by.
- [Plugins](plugins.md): the designed extension point — sandboxed Wasm, not yet implemented.
