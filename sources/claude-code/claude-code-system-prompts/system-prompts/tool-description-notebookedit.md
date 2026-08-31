---
name: tool-description-notebookedit
description: "Tool description for editing Jupyter notebook cells"
type: concept
updated: 2026-05-30
tags: [claude-code, system-prompt, tool-description]
ccVersion: 2.0.14
source: https://github.com/Piebald-AI/claude-code-system-prompts/blob/main/system-prompts/tool-description-notebookedit.md
---
<!--
name: 'Tool Description: NotebookEdit'
description: Tool description for editing Jupyter notebook cells
ccVersion: 2.0.14
-->
Completely replaces the contents of a specific cell in a Jupyter notebook (.ipynb file) with new source. Jupyter notebooks are interactive documents that combine code, text, and visualizations, commonly used for data analysis and scientific computing. The notebook_path parameter must be an absolute path, not a relative path. The cell_number is 0-indexed. Use edit_mode=insert to add a new cell at the index specified by cell_number. Use edit_mode=delete to delete the cell at the index specified by cell_number.
