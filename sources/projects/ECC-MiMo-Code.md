# Project memory
_Durable project-level knowledge. Persists across all sessions in this project. Edit only content under italic instructions._

## Project context
_What is this project? What's its goal? High-level identity._

MiMoCode (mi-code) — Xiaomi's AI coding assistant CLI tool, built on OpenCode. TypeScript project at `E:\Files\Projects\mi-code`. Uses ECC (Everything Claude Code) skills/rules/hooks for enhanced capabilities.

## Rules
_Hard constraints from user that every session must respect._

- **ECC mimocode.json path convention**: All paths in mimocode.json must be relative to `.mimocode/` directory root, not project root. Example: `"paths": ["./skills"]` (relative to .mimocode/). Agent prompts use `{file:.mimocode/prompts/agents/xxx.txt}` format. Plugins use `"./plugins/ecc-mimocode.ts"` format.
- **MiMo Code plugin export format**: Plugins must use `export default function` pattern, NOT `Plugin()` factory pattern.

## Architecture decisions
_Major design choices with rationale. The "why" matters more than the "what" for future sessions._

- **ECC hooks → MiMo Code adapter**: Instead of trying to make Claude Code's hooks.json work with MiMo Code, created a TypeScript plugin (`ecc-mimocode.ts`) that implements the same functionality using MiMo Code's native plugin API events. This is because MiMo Code uses different event names (tool.execute.before vs PreToolUse) and a different plugin resolution system (doesn't use CLAUDE_PLUGIN_ROOT).
- **MiMo Code directory structure**: MiMo Code uses `.mimocode/` directory, NOT `.opencode/`. Config, plugins, skills, commands, and prompts all go in `.mimocode/`. Created `mimocode.json` with explicit plugin reference and skills paths.
- **Plugin export format (VERIFIED WORKING)**: `export default { id: "xxx", server: async (input) => ... }` is the correct format. Named `server` export alone does NOT work. MiMo Code's `readV1Plugin` checks `mod.default` for object with `server` property. This format has been tested and confirmed working.
- **ECC plugin adapter confirmed functional**: All `tool.execute.before` hooks fire correctly. Dangerous command blocking and protected config file blocking both operational.
- **ECC plugin now implements all 25 MiMo Code hooks**: Full event coverage registered in `ecc-mimocode.ts`. Includes `event`, `config`, `tool`, `auth`, `provider`, `chat.message`, `chat.params`, `chat.headers`, `permission.ask`, `command.execute.before`, `tool.execute.before`, `shell.env`, `tool.execute.after`, `experimental.chat.messages.transform`, `experimental.chat.system.transform`, `experimental.session.compacting`, `experimental.compaction.autocontinue`, `experimental.text.complete`, `tool.definition`, `actor.preStop`, `actor.postStop`, `session.pre`, `session.post`, `session.userQuery.pre`, `session.userQuery.post`.

## Discovered durable knowledge
_Cross-task facts that survive across sessions. Promoted from session checkpoints' §7 when proven durable._

- **MiMoCode package cache bug** (Windows): `packages/opencode/src/npm/index.ts:44-47` — `sanitize()` strips `<>:"|?*` but not spaces. When a pkg spec with spaces hits `Npm.add()`, the cache dir `~/.cache/mimocode/packages/<pkg>` gets an invalid name. Fix: add space to the illegal chars set. **Bug report filed**: https://github.com/XiaomiMiMo/MiMo-Code/issues/1368
- **ECC skills manual install**: 21 ECC skills from `.claude/skills/ecc/` can be installed to `~/.local/share/mimocode/plugins/ecc-<name>/` with a `plugin.json` manifest — bypasses broken npm package installer.
- **MiMoCode source**: upstream `https://github.com/XiaomiMiMo/MiMo-Code.git`, local fork `E:\Files\GitHub\MiMo-Code`
- **ECC OpenCode installation**: The `ecc-universal` npm package (v2.0.0) contains a complete `.opencode/` directory at `C:\Users\<user>\AppData\Roaming\npm\node_modules\ecc-universal\.opencode\`. Copy to `.mimocode/` directory for MiMo Code support.
- **ECC repo vs npm package**: The GitHub repo `WorldFlowAI/everything-claude-code` has Claude Code configs only (no `.opencode`). OpenCode support lives exclusively in the `ecc-universal` npm package.
- **ECC installed to mi-code**: Full `.opencode/` directory copied to `E:\Files\Projects\mi-code\.mimocode\` — commands, plugins, prompts, skills (150+). Created `mimocode.json` with plugin reference and skills paths.
- **ECC mimocode.json path conventions**: All paths in mimocode.json should be relative to the `.mimocode/` directory root. Agent prompts use `{file:.mimocode/prompts/agents/xxx.txt}` format. Skills use `"paths": ["./skills"]` (relative to .mimocode/). Plugins use `"./plugins/ecc-mimocode.ts"` format.
- **ECC hooks.json architecture**: Uses `CLAUDE_PLUGIN_ROOT` env var bootstrap pattern scanning `~/.claude/plugins/ecc/` and other paths. May need adaptation for MiMo Code's plugin resolution.
- **MiMo Code uses `.mimocode` directory (NOT `.opencode`)**: Config loading in `config.ts:820-862` looks for `.mimocode` directory. Plugin auto-discovery scans `{plugin,plugins}/*.{ts,js}` under `.mimocode/`. Skills/commands/prompts also go in `.mimocode/`.
- **ECC plugin adapter confirmed working**: Created `ecc-mimocode.ts` in `.mimocode/plugins/` to bridge ECC hooks to MiMo Code's native plugin API. Uses `tool.execute.before` for dangerous command blocking, `tool.execute.after` for output warnings, `session.pre/post` for lifecycle logging. **All hooks verified functional after export format fix.**
- **MiMo Code plugin loader resolution order**: `readV1Plugin` checks `mod.default` first for object with `server` property. If that fails, `getLegacyPlugins` iterates `Object.values(mod)` looking for functions. Named `server` export alone is NOT sufficient. **Correct format**: `export default { id: "xxx", server: async (input) => ... }` (VERIFIED WORKING).
- **MiMo Code plugin confirmed loading**: After fixing export format, logs show `loading plugin` entry without errors. Plugin is fully operational.
- **Edit tool session requirement**: Edit tool requires the file to be Read in the current session before any Edit operation. Must always Read before Edit.
- **Windows PowerShell command differences**: `rm -rf /` is Linux syntax; Windows PowerShell uses `Remove-Item -Recurse -Force`. Hook patterns for dangerous command blocking need Windows-compatible variants.
- **MiMo Code plugin console.log invisible**: `console.log` inside plugin hooks does NOT appear in MiMo Code's session log files. The plugin system suppresses/redirects stdout from plugins. For hook verification, must use file-based logging (`appendFileSync` to a file in `.mimocode/`) or observe hook effects (blocked commands, injected env vars).
- **ECC hook file logging pattern**: Use `appendFileSync` to write to `.mimocode/ecc-hooks.log`. Include timestamp and hook-specific context. The log file persists across sessions and can be read after session end to verify all hooks fired.
- **shell.env verified working**: Can inject env vars into bash session. Confirmed `ECC_LOADED=1` appears in bash env after plugin loads.
- **MiMo Code plugin TypeScript compilation**: `tsc --noEmit` reports TS2591 errors on Node.js globals (`process`, `require`, `fs`, `path`) even though the plugin runs in MiMo Code's Node.js runtime. The plugin's transpiler handles these at runtime — TypeScript strict checks are a compilation concern, not a runtime concern for MiMo Code plugins. Use `(globalThis as any).require("fs")` and `(globalThis as any).process?.cwd?.()` to bypass.
- **MiMo Code plugin runtime lacks `require()`**: `globalThis.require("fs")` does NOT work in MiMo Code's ESM plugin runtime. Cannot use `require()` for file I/O in plugins. Must use in-memory data structures or custom tools for state persistence. The `_eccLog` array + `ecc-log` custom tool pattern works as a workaround.
- **Custom tool registration in MiMo Code plugins**: `hooks["tool"] = { "tool-name": { description, parameters, execute } }` successfully registers callable tools. The tool appears in MiMo Code's tool registry and can be called by the LLM. Useful for exposing internal plugin state.

## ECC hooks testing results (2026-06-26)

### Verified working hooks:
1. **tool.execute.before** — Blocks dangerous commands (`rm -rf /`, `mkfs`, `shutdown`, `reboot`, `format C:`, fork bomb) and protects config files (`.eslintrc`, `.prettierrc`, `tsconfig.json`, `biome.json`, `.gitignore`, `package-lock.json`). Returns `[ECC] Blocked: ...` or `[ECC] Protected: ...` messages.
2. **shell.env** — Injects `ECC_LOADED=1` and `NODE_ENV=development` into bash environment. Verified via `echo $ECC_LOADED` → `1`.
3. **session.pre** — Logs `[ECC] Session starting: {sessionID}, agent: {agentID}` at session start.
4. **session.post** — Logs `[ECC] Session ended: {sessionID}, outcome: {outcome}, duration: {time}s` at session end.
5. **actor.preStop** — Logs tool call counts (bash/write/edit) and total duration before actor stops.
6. **actor.postStop** — Returns `{ continue: false }` after actor stops.
7. **event** — Listens for `session.created` and other bus events.
8. **config** — Runs at startup for config modification.
9. **chat.message** — Fires on every user message.
10. **chat.params** — Fires on every LLM call, can modify temperature/topP.
11. **chat.headers** — Fires on every LLM call, can add custom headers.
12. **permission.ask** — Fires on permission requests, can auto-approve/deny.
13. **command.execute.before** — Blocks specific slash commands.
14. **tool.execute.after** — Detects `console.log` in write/edit output, adds metadata warning.
15. **experimental.chat.messages.transform** — Fires during message processing.
16. **experimental.chat.system.transform** — Fires during system prompt generation.
17. **experimental.session.compacting** — Injects context during session compaction.
18. **experimental.compaction.autocontinue** — Enables auto-continue after compaction.
19. **experimental.text.complete** — Fires on text completion.
20. **tool.definition** — Fires when tools are defined.
21. **session.userQuery.pre** — Fires before each LLM call, can cancel/modify queries.
22. **session.userQuery.post** — Fires after each LLM call, can check response quality.

### Hook verification method:
- Used in-memory `_eccLog` array + `ecc-log` custom tool pattern
- Plugin registers `ecc-log` tool via `hooks["tool"]` which returns accumulated log entries
- Direct `console.log` in plugins does NOT appear in MiMo Code logs (suppressed by plugin system)
- File-based logging (`appendFileSync`) also doesn't work due to missing `require()` in ESM runtime

### Key findings:
- All 25 hooks registered successfully in MiMo Code's plugin runtime
- Plugin loads without errors after export format fix (`export default { id, server }`)
- `ecc-log` tool appears in tool registry and is callable by LLM
- MiMo Code logs show `tool.registry status=started ecc-log` and `status=completed ecc-log` confirming registration
- Session lifecycle hooks (pre/post) fire automatically during normal operation
- Tool execution hooks fire on every tool call
- Experimental hooks fire during corresponding system events
