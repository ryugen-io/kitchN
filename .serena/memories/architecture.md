# Kitchn - Architecture

## Crate Dependency Graph

```
kitchn (CLI binary)
    └── k-lib (core logic)

k-log (logging binary)
    └── k-lib

k-ffi (C-ABI library)
    └── k-lib
```

## Module Structure

### k-lib (kitchn_lib)
Core library containing all business logic.

```
src/
├── lib.rs          # Public API exports
├── config.rs       # Configuration loading (Cookbook, Theme, Icons, Layout)
├── db.rs           # PastryDB (Sled-based ingredient storage)
├── factory.rs      # Object creation/initialization
├── ingredient.rs   # .ing file parsing and handling
├── logger.rs       # Themed logging (terminal + file)
├── packager.rs     # .bag packaging (ZIP)
├── processor.rs    # Tera template processing
└── defaults.toml   # Embedded default configuration
```

**Key Types**:
- `Cookbook` - Aggregated config (theme + icons + layout + dictionary)
- `ThemeConfig`, `IconsConfig`, `LayoutConfig`, `DictionaryConfig`
- `Ingredient` - Parsed .ing file
- `ConfigError` - Typed error enum

### kitchn_cli
CLI wrapper binary.

```
src/
├── main.rs         # Entry point, file locking
├── args.rs         # Clap argument definitions
├── logging.rs      # Tracing setup
└── commands/       # Command implementations
    ├── stock.rs
    ├── cook.rs
    ├── wrap.rs
    ├── pantry.rs
    └── bake.rs
```

### k-ffi (kitchn_ffi)
C-ABI compatible interface.

```
src/
└── lib.rs          # All FFI functions
```

**FFI API**:
- `kitchn_context_new()` / `kitchn_context_free()`
- `kitchn_context_set_app_name()`
- `kitchn_log()` / `kitchn_log_preset()`
- `kitchn_pack()` / `kitchn_unpack()` / `kitchn_store()`

### k-log (kitchn_log)
Standalone logging CLI.

```
src/
└── main.rs         # Simple clap CLI using k-lib::logger
```

## Data Flow

### Config Loading
```
TOML files → Cookbook::load() → Binary cache (bincode)
                                     ↓
                              Cookbook::load_with_cache()
```

### Ingredient Processing
```
.ing file → Ingredient::parse() → PastryDB::store()
                                       ↓
kitchn cook → PastryDB::get_all() → Tera::render() → Target files
                                                          ↓
                                                    Hooks::execute()
```

### FFI Flow
```
C/C++/Python → kitchn.h → k-ffi → k-lib
```

## Single Instance Policy
- Uses `flock()` on `~/.cache/kitchn/kitchn.lock`
- Prevents concurrent pantry/config modifications
- Debug viewer is exempt from locking
