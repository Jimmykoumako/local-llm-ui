#!/usr/bin/env python3
"""Detect ISO 639-1 language code from stdin text using langdetect."""

import sys


def main() -> None:
    text = sys.stdin.read()
    if not text.strip():
        print("en")
        return
    try:
        from langdetect import detect

        print(detect(text))
    except Exception:
        print("en")


if __name__ == "__main__":
    main()
