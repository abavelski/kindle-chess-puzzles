# Task 23 — Add the analysis-browser state machine

**Status:** Ready  
**Depends on:** Task 22  
**Primary area:** `crates/chess-core/src/app.rs`  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Pure application-state tests; no device test


## Device/human-testing rule
This task must not request Kindle access. Preview/open/close/progress behavior is deterministic application state.

## Outcome
Add pure actions/state for opening analysis, selecting nodes, paging/focusing, and closing without changing live solving state.

## Required behavior
- Add explicit actions such as `OpenAnalysis`, `CloseAnalysis`, `SelectAnalysisNode`, and page navigation.
- Keep live board and grading cursor untouched while previewing.
- Derive displayed board from selected node FEN.
- Browsing never marks solved or returns `ProgressChanged`.
- Closing restores exact live solve board/selection state.
- Reset, puzzle navigation, collection switch, and mode changes clear preview deterministically.
- Define a tested disabled/no-op state for puzzles without rich analysis.

## Tests
Cover main/sideline selection, repeated selection, close/restore, correct/wrong solving state surviving preview, solved-puzzle preview, and navigation/reset interactions.

## Non-goals
No layout, hit testing, stylus/Linux events, or progress schema changes.

## Suggested commit
`core: add non-mutating analysis preview state`
