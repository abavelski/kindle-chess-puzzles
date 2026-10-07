# Build, stage and deploy to Kindle Scribe

Task 08's automation targets the measured first-generation Scribe (Barolo,
firmware 5.19.6, ARMv7 hard-float, glibc 2.35). Checkpoint F's clean build,
deploy/update, library launch/exit, and uninstall/reinstall observations passed.
Library launch after a full reboot was explicitly deferred at phase-one close and remains
unverified. Existing idle/button sleep and wake were confirmed by the user on
2026-10-04; the new sleep indicator awaits physical visual review. A cross-build or
package contract test does not establish reboot or suspend reliability.

## Fresh checkout

Install Git, Python **3.12 or later**, make, and rustup. Python's tar extraction
filter is used to unpack the downloaded toolchain. On macOS install the Xcode
command-line tools; on Ubuntu install `build-essential python3`. Initialize:

```sh
git submodule update --init --recursive
rustup show
python3 -m venv .venv
. .venv/bin/activate
python3 -m pip install -r requirements-tools.txt
scripts/check.sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
```

`rust-toolchain.toml` selects Rust **1.85.1**, rustfmt, clippy and
`armv7-unknown-linux-gnueabihf`. Cargo uses the checked-in lockfile. All commands
work from another directory when invoked by their absolute path.

The build first runs formatting, clippy, workspace tests (including render
snapshots), generated-asset verification, and Python/C/lifecycle/package
contracts. A failing check prevents the cross-build and removes its receipt.
It then regenerates the Sashité layers, obtains the checksum-pinned official
**Zig 0.13.0** archive for macOS/Linux (ARM64 or x86-64), and creates readable
`target/toolchains/bin/kindle-{cc,ar,ranlib}` wrappers. The C compiler/linker
command is `zig cc -target arm-linux-gnueabihf.2.35`. Zig provides its own
headers/libc targeting that version; no global ARM compiler, FBInk library, or
Kindle sysroot is required. Toolchain URLs and SHA-256 hashes are stored in
`tools/kindle_build.py`, sourced from [Zig's download index](https://ziglang.org/download/index.json).
The download cache is under ignored `target/toolchains/`.

FBInk must be clean at **92e127008145b2a22fba7c59815d810d716310dd**.
The source-content fingerprint also verifies all recursively pinned FBInk
submodules; extracted corresponding source can be verified without Git metadata. Cargo's
FBInk boundary invokes make with `KINDLE=1 MINIMAL=1 IMAGE=1 staticlib`, compiles
the C bridges, and statically links their archives and FBInk into the Rust
release binary. C objects are rebuilt when compiler settings change. glibc
and the ARM hard-float loader remain dynamic device dependencies. Build-time
ELF validation rejects incorrect ARM/ELF/ABI headers, a different loader, or
GLIBC version references newer than 2.35. Loader/runtime confirmation is still
required on the physical Scribe.

`check-kindle.sh` remains the earlier GNU/override smoke-build command. Use
`build-kindle.sh` for the pinned release workflow and staged artifacts.

## Inspectable artifacts and data layout

Outputs:

```text
target/kindle/kindle-chess                  optimized, validated release
target/kindle/build.json                    successful-check/source/binary receipt
target/kindle/stage/                        inspectable install directory
target/kindle/kindle-chess.tar              deterministic scp/install archive
target/kindle/kindle_chess_0.1.0_kindlehf.kpkg  KPM manifest-v2 gzip/tar package
```

Staging requires a successful receipt matching the exact source tree and
binary; changing code, tests, packaging, or build scripts requires rebuilding.
SHA256SUMS covers every payload file. Tar file order, ownership and timestamps
are normalized; gzip timestamps are normalized too. This makes repeat packaging
of the same build deterministic, without promising bit-identical binaries
across different checkout paths or operating systems.

Installed layout:

```text
/mnt/us/kindle-chess/runtime/       executable, launchers, receipt, notices, source.tar
/mnt/us/kindle-chess/puzzles/       immutable user collections; created by the app
/mnt/us/kindle-chess/games/         immutable user review collections
/mnt/us/kindle-chess/state/         progress and settings; created by the app
/mnt/us/kindle-chess/logs/          launch.log and lifecycle.log
/mnt/us/documents/kindle-chess.sh   library scriptlet
```

The package contains no puzzle/game/state/log directories to copy over user data.
The binary already embeds its Task 05 default collection and Task 06 installs
it only when no matching collection exists. User progress and invalid/future
records retain Task 06 protections.

The source archive includes project sources, original SVGs, pinned FBInk,
locked Rust dependency sources and their license files, plus a vendored Cargo
configuration. Ignored FBInk build products are excluded; host contracts verify
the extracted dependency tree resolves with `cargo metadata --offline --locked`.
From extracted source, `scripts/build-kindle.sh` uses the same workflow (its
first Zig download still requires network access). Runtime notices record FBInk GPL-3.0-or-later, Sashité CC0, and
fixture provenance. The repository does not yet declare a license for its own
code; these are local-install artifacts. A public release requires the owner's
GPL-compatible license decision and corresponding-source compliance.

## Deploy and launch

Use the previously verified scp-over-SSH transport. The target address is an
argument, never a baked-in device identifier:

```sh
scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222
```

SSH authentication is interactive; the script does not store credentials or
relax host-key verification. It uses scp's legacy `-O` transport for the measured
KOReader SSH server, validates the entire stage before network access, uploads
to a unique app-owned temporary path, and runs the installer. Remote incoming
files are cleaned on success/failure. It never launches the app automatically.

Exit the app before updating. The installer checks the app supervisor lock and
process, holds an installation lock, verifies payload hashes, prepares a new
runtime directory, and renames the previous runtime to a backup before swapping
in the new one. Signal/error cleanup restores the backup if needed. It uses
ordinary directories/renames rather than requiring symlinks on `/mnt/us`.
Only runtime files and this app's matching scriptlet are replaced. A different
file at `documents/kindle-chess.sh` causes an explicit refusal. Reinstall/update
never writes puzzles, state, or logs.

With Scriptlets/SH_Integration installed, open **Kindle Chess Puzzles** in the
Kindle library. [KindleModding's scriptlet documentation](https://kindlemodding.org/kindle-dev/scriptlets.html)
specifies `.sh` files under `documents` and the `Name`/`Author` header format.
The scriptlet includes `# DontUseFBInk`, as used by the installed KOReader
scriptlet, so SH_Integration does not become another framebuffer writer.
Library launch, stable board visibility, interaction and X exit were verified
on 2026-10-03 after the paired-process display ownership fix. For the
verified direct-shell route:

```sh
ssh -tt -p 2222 root@192.168.1.20
sh /mnt/us/kindle-chess/runtime/launch.sh
```

The package launcher validates `uname -m` and the hard-float loader before
starting the app, logs failures, then calls the Task 07 lifecycle
supervisor. Display ownership is handled only in that supervisor: it verifies and pauses
awesome and Xorg before chess, resumes the same processes on cleanup, then requests native
repaint. No rotation, framebuffer-mode or power-policy logic is duplicated. Exit through the app's top-right X control or terminate the supervised
shell as described in [lifecycle notes](EINK_LIFECYCLE.md). Keep recovery SSH
available for the first library-launch test.

## KPM packaged form

The `.kpkg`, verified with KPM 0.2.2 on this Scribe, follows the current [KPM package conventions](https://kindlemodding.org/kindle-dev/kpm/creating-a-package.html)
and [example manifest](https://github.com/KindleModding/example_kpm_package/blob/main/manifest.json):
manifest version 2, `kindlehf` platform, gzip/tar payload and install/uninstall/
launch hooks at package root. Read-only inspection confirmed KPM 0.2.2 (`kindlehf`) at
`/var/local/kmc/bin/kpm` on this Scribe. Use the scp/scriptlet route first.
For a local KPM installation, copy the `.kpkg` to `/mnt/us/`, then use
`/var/local/kmc/bin/kpm install file:///mnt/us/kindle_chess_0.1.0_kindlehf.kpkg`
and `/var/local/kmc/bin/kpm launch kindle_chess`. Installation, uninstall and reinstall with that local-package command were
verified on the Scribe without changing puzzle/progress hashes. No repository
publication is configured. The uninstall hook recognizes KPM's `upgrade`
argument and preserves the working runtime until the replacement installs.
The library scriptlet also works with the direct installation.

## Uninstall, recovery and reinstall

Exit the app, then run on the Kindle:

```sh
sh /mnt/us/kindle-chess/runtime/uninstall.sh
```

This removes runtime and the exact matching scriptlet; it preserves puzzles,
progress and logs. Re-deploy reinstalls the runtime and scriptlet and restores
existing progress through normal app startup. Data deletion is a separate manual
choice. KPM owns removal of its package files.

An interrupted update may leave `.install-lock`, `runtime.old`, or `runtime.new`.
Inspect processes before removing an abandoned installation lock. If runtime
is absent but runtime.old exists, rename runtime.old back to runtime, then
re-run the installer. Never remove puzzles/state to recover an installation.
For app/supervisor crash recovery use the Task 07 procedure.

## HUMAN CHECKPOINT F

Record the clean checkout commit, artifact SHA-256, target/firmware, exact
commands and per-item observations in `docs/device/ks1-barolo.md`: clean check,
build, stage/deploy, library launch/exit, second-build update with unchanged
puzzle/progress content, reboot/relaunch, and uninstall/reinstall preservation.
All items except post-reboot library launch were physically verified. At phase-one close
on 2026-10-03 the reboot observation was explicitly deferred, so Task 08 is Implemented
with that limitation recorded rather than with a fabricated pass. Task 07's non-sleep
checks are accepted; its natural suspend/resume check remains separately deferred and
must not be inferred from checkpoint F. Host tests do not substitute for either missing
observation.


The app grabs finger input while open, so native swipe menus are unavailable.
Use the **X** in the top-right header to exit; it remains accessible during
promotion and collection selection. Exit retries pending progress persistence,
returns normally to release input/display resources, and lets the supervisor
request the stock screen repaint. SSH recovery remains documented in
[EINK_LIFECYCLE.md](EINK_LIFECYCLE.md). The earlier library white-overpaint regression
was resolved by the verified paired awesome/Xorg display handoff; ordinary library
launch/interaction/X-exit and native repaint now pass. Post-reboot launch remains the
explicitly deferred observation described above.

## Import and update review games

With the host virtual environment active, convert ordinary or annotated PGNs:

```sh
python3 tools/pgn_review_converter.py my-games.pgn -o games-my-games.json --title "My games"
```

Exit the app, create `/mnt/us/kindle-chess/games/` if needed, and copy the JSON
there using USB or your established SSH transport. Only `games.json` and
`games-<nonempty>.json` are discovered; `KINDLE_CHESS_GAME_DIR` overrides the
directory. Relaunch and tap the workspace icon beside Refresh, then GAMES.
PREV/NEXT traverse the main line; bold moves preview stored positions. FREE
provides reversible scratch exploration. Switching back preserves puzzle state
and progress. Review selection has no durable resume yet.

For updates, back up the JSON and rerun the converter with `-o` targeting the
same filename. Validation precedes atomic host replacement; annotation-only
updates retain generated game IDs and duplicate IDs fail. See
[Game Review import/update](GAME_REVIEW_IMPORT.md) for identity rules, rollback,
and safe copying. Host tests and the cross-build do not establish physical
review-mode acceptance; that remains Task 49.
