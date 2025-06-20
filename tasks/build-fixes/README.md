# Build Fixes Tasks

This folder contains tasks to fix build errors that occurred after enabling strict lint checking.

## Tasks Overview

These tasks can be worked on in parallel as they affect different files and functions:

1. **001-fix-unused-mut-markdown-widget.md** - Remove unused `mut` in `src/ui/markdown/widget.rs:76`
2. **002-fix-unused-function-format-messages.md** - Handle unused function in `src/ui/tui/ui.rs:629`  
3. **003-fix-unused-function-format-message.md** - Handle unused function in `src/ui/tui/ui.rs:874`
4. **004-fix-unused-field-viewport-height.md** - Handle unused field in `src/ui/markdown/widget.rs:210`

## Priority
All tasks are medium priority build errors that prevent compilation.

## Status
All tasks are ready to be worked on independently.