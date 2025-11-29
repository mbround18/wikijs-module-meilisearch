#!/usr/bin/env sh
set -eu

NOW="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
HOSTNAME_VAL="${HOSTNAME:-unknown}"
SRC="${SOURCE:-/modules/meilisearch}"
DST="${DESTINATION:-/wiki/server/modules/meilisearch}"

VERSION_VAL="dev"
if [ -f "$SRC/VERSION" ]; then
	VERSION_VAL="$(cat "$SRC/VERSION" | tr -d '\n' || echo dev)"
fi

echo "=============================================="
echo "Wiki.js Meilisearch Module Copy Entrypoint"
echo "----------------------------------------------"
echo "Time:        $NOW"
echo "Container:   $HOSTNAME_VAL"
echo "Version:     $VERSION_VAL"
echo "Purpose:     Copy module assets into Wiki.js"
echo "Source:      $SRC"
echo "Destination: $DST"
echo "=============================================="

export RUST_LOG="${RUST_LOG:-info}"

exec /wiki_meilisearch "$@" 2>&1 | sed 's/^/[wiki-meili-copy] /' 
