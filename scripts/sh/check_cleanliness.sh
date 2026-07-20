#!/bin/bash
set -e

# Project Tidiness Enforcement Script
# Checks affected directories and files for clutter:
# 1. No directory affected by changes can exceed 9 items (files or subdirectories).
# 2. No source/text file affected by changes can exceed 500 lines.

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

VIOLATIONS=()
CHANGED_FILES=()

if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    UNCOMMITTED=$(git status --porcelain 2>/dev/null | awk '{print $2}' || true)
    LAST_COMMIT=$(git diff --name-only HEAD~1 HEAD 2>/dev/null || true)
    
    RAW_FILES=$(echo -e "$UNCOMMITTED\n$LAST_COMMIT" | sort -u | grep -v '^$' || true)
    for f in $RAW_FILES; do
        if [ -f "$f" ]; then
            CHANGED_FILES+=("$f")
        fi
    done
fi

if [ ${#CHANGED_FILES[@]} -eq 0 ]; then
    ALL_SRC=$(find src templates scripts -type f 2>/dev/null || true)
    for f in $ALL_SRC; do
        CHANGED_FILES+=("$f")
    done
fi

readarray -t UNIQUE_FILES < <(printf "%s\n" "${CHANGED_FILES[@]}" | sort -u | grep -v '^$')

AFFECTED_DIRS=()
for f in "${UNIQUE_FILES[@]}"; do
    if [ -f "$f" ]; then
        dir=$(dirname "$f")
        while [ "$dir" != "." ] && [ "$dir" != "/" ] && [ "$dir" != ".." ]; do
            AFFECTED_DIRS+=("$dir")
            dir=$(dirname "$dir")
        done
    fi
done

readarray -t UNIQUE_DIRS < <(printf "%s\n" "${AFFECTED_DIRS[@]}" | sort -u | grep -v '^$')

# 1. Check Directory Item Limit (max 9 direct children)
for dir in "${UNIQUE_DIRS[@]}"; do
    if [ ! -d "$dir" ]; then
        continue
    fi
    if [[ "$dir" == *".git"* || "$dir" == *"target"* || "$dir" == *"build_temp"* || "$dir" == *"torsocks_env"* ]]; then
        continue
    fi
    count=$(find "$dir" -maxdepth 1 -mindepth 1 | wc -l)
    if [ "$count" -gt 9 ]; then
        VIOLATIONS+=("[DIR OVER LIMIT] Directory '$dir' has $count items (max allowed: 9)")
    fi
done

# 2. Check File Line Count (max 500 lines)
for file in "${UNIQUE_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        continue
    fi
    if [[ "$file" == *".git"* || "$file" == *"target"* || "$file" == *"build_temp"* || "$file" == *"torsocks_env"* || "$file" == *"Cargo.lock"* || "$file" == *".ttf"* || "$file" == *".png"* || "$file" == *".jpg"* || "$file" == *".ico"* || "$file" == *".bmp"* || "$file" == *"bin/hnsd"* || "$file" == *"bin/arti"* || "$file" == *".so"* || "$file" == *".a"* ]]; then
        continue
    fi
    lines=$(wc -l < "$file" 2>/dev/null || echo 0)
    if [ "$lines" -gt 500 ]; then
        VIOLATIONS+=("[FILE OVER LIMIT] File '$file' has $lines lines (max allowed: 500)")
    fi
done

if [ ${#VIOLATIONS[@]} -ne 0 ]; then
    echo ""
    echo "================================================================================"
    echo "CLUTTER VIOLATION: You are an untidy bastard!"
    echo "================================================================================"
    echo "Offending cases found:"
    for v in "${VIOLATIONS[@]}"; do
        echo "  - $v"
    done
    echo ""
    echo "Please bear in mind untidy contibutor or ai agent we do not accept clutter around these places fix these files before continuing or ever thinking about pushing"
    echo "================================================================================"
    echo ""
    exit 1
fi

echo "[TIDINESS CHECK PASSED] All affected directories <= 9 items and files <= 500 lines."
