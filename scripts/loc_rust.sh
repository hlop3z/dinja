#!/usr/bin/env bash

# Default minimum lines of code
MIN_LOC=${1:-500}

find . \
  -type d -name target -prune -o \
  -name "*.rs" -print0 |
  xargs -0 wc -l |
  awk -v min="$MIN_LOC" '$1 > min'