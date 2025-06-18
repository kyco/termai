# Fix unused mut in markdown widget

## Issue
Variable does not need to be mutable in `src/ui/markdown/widget.rs:76`

```
error: variable does not need to be mutable
  --> src/ui/markdown/widget.rs:76:15
   |
76 |     fn render(mut self, area: Rect, buf: &mut Buffer) {
   |               ----^^^^
   |               |
   |               help: remove this `mut`
```

## Solution
Remove the `mut` keyword from the `self` parameter in the render function.

## File Location
- `src/ui/markdown/widget.rs:76`

## Priority
Medium - Build error that prevents compilation