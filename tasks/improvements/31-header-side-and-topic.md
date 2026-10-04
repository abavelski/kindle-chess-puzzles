# Task 31 — Header side to move and puzzle topics

**Status:** Implemented
**Depends on:** completed Tasks 29–30
**Validation:** host automated tests and reviewed Gray8 snapshots; no physical checkpoint
**Working branch:** main (explicit user instruction)

Move the starting puzzle side-to-move label from the Solution status panel to the
center of the header, keeping puzzle identity/difficulty and existing controls.
Show optional JSON `topic` text by default in the Solution status panel, including
Cyrillic and up to three topic lines. NOTE shows only the description, replacing the
topic until hidden. Use larger text for the topic. Preserve description toggle/automatic reveal,
grading, analysis references, progress and uploaded collections. Verify embedded
font coverage; add a licensed deterministic fallback if Cyrillic is absent.

Acceptance: failing-first parser/font/render tests cover topic strings, absent/null
and legacy metadata compatibility, Cyrillic glyphs, white/black centered header
labels, no duplicated status label, description reveal/hide and solved reveal,
three-line topic with description, Free Board, long header metadata, and existing
analysis behavior. Review snapshots; run scripts/check.sh and the Kindle build.
Commit to main and deploy using the existing installer, recording installed binary
and unchanged collection/progress hashes. Physical readability awaits user review.

## Implementation and validation — 2026-10-04

Parser tests failed first because topic was not modeled; font coverage failed on
Cyrillic А; header/status tests failed on the absent centered label and duplicated
status text. Focused tests passed after implementation. Topics preserve authored
line breaks and share NOTE/automatic solved reveal. Non-string topic metadata
remains ignored to preserve previously accepted collections.

The header reserves a centered starting-side label and clips puzzle metadata to
its own left region, reducing its font size when necessary. Existing right-hand
controls and solved feedback remain intact. Noto Sans Regular/Bold provide missing
glyphs in both measurement and rendering; fallback fonts load only when needed.
Latin text retains Atkinson. Both fonts' OFL notices ship in runtime licenses.

Reviewed all changed Gray8 snapshots, including white/black side, three-line
Russian topics, topic-only, Free Board, long metadata, solving feedback, dialogs
and analysis pages. Existing analysis fixture pixels below the header were
identical to the previous reviewed images. Snapshot checksums were updated only
after review. Optional UI_SNAPSHOT_DIR exports review PGM files.

scripts/check.sh passed. The build gate encountered the previously observed
lifecycle readiness race once; its focused rerun and the full subsequent build
gate passed. scripts/build-kindle.sh produced the validated ARMv7/glibc-2.35
release with SHA-256:
`7bae259c2c4e5813e737ad85403638d66cc4ea4e4a20f674dee219163c60ec4e`.
Implementation commit `50ae2bf` is on main, per user instruction. Staged and
installed using the existing scripts at root@192.168.1.20:2222. The installed
binary hash matches the release above; all five collection hashes and the
progress hash were unchanged. Dynamic-loader dependency resolution succeeded.
The app is stopped and ready for library launch. Physical readability/user
acceptance is pending; deployment does not establish that observation. Details
are recorded in docs/device/ks1-barolo.md.

## Topic visibility correction — 2026-10-04

The user accepted the header and Cyrillic rendering, but clarified that topics
belong in the default solving status, replacing the former side-to-move status.
The original implementation incorrectly tied topic visibility to NOTE.

A regression test failed on absent default topic pixels, then passed after
separating default topic rendering from NOTE description rendering. Topic text
uses 40px instead of 30px at Scribe metrics. Tests cover three lines, NOTE-only
description, hide, reset, navigation, progress stability and solved-note return.
Reviewed changed snapshots before updating checksums; topic-free snapshots and
the accepted header remain unchanged.
