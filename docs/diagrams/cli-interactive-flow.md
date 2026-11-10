# CLI interactive flow

How the `ag` binary decides between its scriptable path and its interactive
interface, and how both reach the same executor (`RFC-0018`, roadmap Phase
4.7). Last reviewed: 2026-11-10. Source: `crates/ag-cli/src/`.

## Input resolution for `ag new`

```mermaid
graph TD
  args[Command line: ag new NAME --template T] --> complete{All inputs given?}
  complete -- yes --> request[NewProjectRequest]
  complete -- no --> session{Interactive session?}
  session -- "no: not a terminal, CI, AG_NO_TUI, TERM=dumb, --no-interactive" --> name{NAME given?}
  name -- no --> usage[Usage error, exit code 2]
  name -- yes --> default[Template defaults to rest] --> request
  session -- yes --> size{Terminal at least 80x24?}
  size -- yes --> wizard[Full-screen wizard]
  size -- no --> line[Sequential line prompts]
  wizard --> answered{Confirmed?}
  line --> answered
  answered -- no --> cancelled[Cancelled, nothing written]
  answered -- yes --> request
  request --> execute[execute_new: single executor]
  execute --> files[Project files]
  execute --> printed[Equivalent command printed after interactive flows]
```

The flag parser and the interactive front-ends produce the same
`NewProjectRequest` value. Only `execute_new` writes files.

## Screen architecture

```mermaid
graph LR
  key[Key press] --> map[msg_for: step and key to Msg]
  map --> update[Wizard::update: pure transition]
  update --> state[Wizard state]
  state --> view[view: reads state, draws frame]
  update -- "Effect::Submit(request) or Effect::Quit" --> loop[Event loop]
  loop --> guard[Terminal guard restores raw mode and alternate screen]
```

`Wizard::update` and `view` perform no I/O. The event loop in
`crates/ag-cli/src/tui/mod.rs` is the only code that touches the terminal,
and the guard in `tui/terminal.rs` restores it on return, on error, on
`Ctrl-C` and on panic.

## Wizard steps

```mermaid
stateDiagram-v2
  [*] --> Name
  Name --> Template: Enter (valid name)
  Name --> [*]: Esc (cancel)
  Template --> Review: Enter
  Template --> Name: Esc
  Review --> Template: Esc
  Review --> Name: Enter, but the directory now exists
  Review --> [*]: Enter (submit request)
  Name --> [*]: Ctrl-C
  Template --> [*]: q or Ctrl-C
  Review --> [*]: q or Ctrl-C
```
