#!/bin/bash

# Extract all wikilinks from wiki files
grep -oP '\[\[([^\]|]+)' wiki/*.md | sed 's/.*\[\[//' | sort -u | while read link; do
  # Remove any anchor (#...)
  link=$(echo "$link" | sed 's/#.*//')
  
  # Check if file exists
  if [[ "$link" == wiki/* ]]; then
    # wiki link - check with and without .md
    if [[ ! -f "$link" && ! -f "$link.md" ]]; then
      echo "BROKEN: $link"
    fi
  elif [[ "$link" == sources/* ]]; then
    # sources link
    if [[ ! -f "$link" && ! -f "$link.md" ]]; then
      echo "BROKEN: $link"
    fi
  fi
done
