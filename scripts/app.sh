#!/bin/sh
# Product-aware build/stage/deploy selector.
set -eu
if [ "$#" -lt 2 ]; then
    echo 'Usage: scripts/app.sh {build|stage|deploy} {puzzles|review|play} [options]' >&2
    exit 2
fi
command=$1
app=$2
shift 2
case "$command" in build|stage|deploy) ;; *)
    echo 'Expected build, stage or deploy' >&2
    exit 2 ;;
esac
case "$app" in puzzles|review|play) ;; *)
    echo 'Expected puzzles, review or play' >&2
    exit 2 ;;
esac
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
exec python3 tools/kindle_build.py "$command" --app "$app" "$@"
