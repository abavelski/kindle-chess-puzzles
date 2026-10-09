# Task 51 — Review games dialog bounds

**Status:** In progress — physical validation pending
**Working branch:** main
**Depends on:** Task 49

## Scope

Only the Game Review GAMES dialog: make it full display width, start directly
below the header, and swap CLOSE with PAGE > so CLOSE is at bottom right.
Keep the existing bottom edge, entries, pagination and all other dialogs unchanged.

## Acceptance

- Failing layout tests specify full width, header adjacency and button order.
- Existing review hit tests and intentionally inspected picker snapshot pass.
- Full host checks and pinned Kindle release build pass.

## Validation — 2026-10-09

Layout test failed on the old inset bounds before implementation. Updated
layout/hit tests and the inspected Gray8 picker snapshot pass; other snapshots
remain unchanged. Full host gates and pinned ARMv7/glibc-2.35 release build passed.
Staged/deployed with existing scripts to root@192.168.1.20:2222 after X exit.
Installed binary matches host; all 118 collection/state hashes are unchanged.
Physical dialog review remains pending.
