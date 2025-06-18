# Fix unused function format_messages_without_selection

## Issue
Function `format_messages_without_selection` is never used in `src/ui/tui/ui.rs:629`

```
error: function `format_messages_without_selection` is never used
   --> src/ui/tui/ui.rs:629:4
    |
629 | fn format_messages_without_selection(messages: &[&crate::session::model::message::Message]) -> Text<'static> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Solution Options
1. Remove the function if it's truly unused
2. Add `#[allow(dead_code)]` if it's intended for future use
3. Find where it should be used and integrate it

## File Location
- `src/ui/tui/ui.rs:629`

## Priority
Medium - Build error that prevents compilation