# Kitchn Log Architecture

## Philosophy
- "Sweet Dracula" theme is the standard, enforced by `kitchn_log`.
- Uses XML-like tags for rich text formatting.
- Ensures consistent styling across the entire Hyprcore ecosystem by being the centralized logging utility.

## Interaction
- `kitchn` CLI calls `kitchn-log` as a subprocess for user-facing, pretty messages.
    - Example: `log_msg("cook_start", "simmering ...")`
- This ensures that even if `kitchn` logic changes, the visual output remains consistent with the Hyprcore brand.

## Cook Summary
- `kitchn cook` reports a concise summary.
- If hooks fail, the summary explicitly states: `cooked X ingredients successfully but Y hooks failed`.

## Verbose/Debug Logging (Internal)
- Separated from the visual `kitchn_log`.
- Uses Rust's standard `log` facade.
- Directed to `/tmp/kitchn-debug.log`.
- Displayed in a **separate terminal window** spawned by `kitchn --debug`.
- Content-based coloring:
    - **ERROR/FAIL**: Red & Bold
    - **WARN**: Yellow
    - **INFO/SUCCESS/STDOUT**: Green/Cyan
    - **DEBUG**: Dimmed Blue
