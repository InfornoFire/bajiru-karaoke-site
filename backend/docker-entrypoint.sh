#!/bin/sh
set -e

mkdir -p /data/storage
chown -R appuser:appuser /data/storage

exec gosu appuser "$@"
