# Fix unused function format_message

## Issue
Function `format_message` is never used in `src/ui/tui/ui.rs:874`

```
error: function `format_message` is never used
   --> src/ui/tui/ui.rs:874:4
    |
874 | fn format_message(message: &crate::session::model::message::Message) -> Text<'static> {
    |    ^^^^^^^^^^^^^^
```

## Solution Options
1. Remove the function if it's truly unused
2. Add `#[allow(dead_code)]` if it's intended for future use
3. Find where it should be used and integrate it

## File Location
- `src/ui/tui/ui.rs:874`

## Priority
Medium - Build error that prevents compilation