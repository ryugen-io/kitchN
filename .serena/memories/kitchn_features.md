# Kitchn Features

## Ingredient Management
- **.ing Files**: The fundamental unit. A TOML file defining an ingredient (metadata + hooks).
- **.bag Files**: A zipped package containing multiple `.ing` files. Created via `kitchn wrap`.
- **Pantry**: A local database (`~/.local/share/kitchn/pantry.db`) storing managed ingredients.

## Commands
- `stock`: Adds ingredients/bags to the pantry.
- `wrap`: Packages a directory of ingredients into a portable `.bag`.
- `cook`: Applies all ingredients from the pantry to the system. This executes the hooks and renders templates.
- `pantry`: Lists all stocked ingredients.
- `bake`: Pre-compiles configuration files (theme, layout, icons) into a binary format for faster subsequent runs.

## Debug Mode (v0.2.0+)
- **Flag**: `--debug` (Global flag).
- **Behavior**: Always spawns a dedicated terminal window (prioritizing `rio`, then `alacritty`, `kitty`) to stream verbose debug logs.
- **Verbose Hook Logging**: Logs exact commands, execution time, exit codes, and full stdout/stderr (even if empty) for every hook.
- **Config Debugging**: Logs detailed file loading paths and Tera context keys.
- **Persistent**: Logs are written to `/tmp/kitchn-debug.log`.
