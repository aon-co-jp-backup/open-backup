#!/bin/sh
# open-backup 0.1.0
cd "$(dirname "$0")"
("../../open-backup" >/dev/null 2>&1 &)
open "http://127.0.0.1:4620/ui"
