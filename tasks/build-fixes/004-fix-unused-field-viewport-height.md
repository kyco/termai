# Fix unused field viewport_height

## Issue
Field `viewport_height` is never read in `src/ui/markdown/widget.rs:210`

```
error: field `viewport_height` is never read
   --> src/ui/markdown/widget.rs:210:5
    |
206 | pub struct ScrollableMarkdown<'a> {
    |            ------------------ field in this struct
...
210 |     viewport_height: u16,
    |     ^^^^^^^^^^^^^^^
```

## Solution Options
1. Remove the field if it's truly unused
2. Add `#[allow(dead_code)]` if it's intended for future use
3. Find where it should be used and integrate it into the logic

## File Location
- `src/ui/markdown/widget.rs:210`

## Priority
Medium - Build error that prevents compilation