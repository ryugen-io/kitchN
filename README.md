# Kitchn

**Strict Corporate Design Enforcement for your System.**

> "Single Source of Truth". One config change propagates to Shells, Scripts, Logs, GUIs, and TUI apps instantly.
> Now with C-API support for C++, Python, and more.

![Stack](https://img.shields.io/badge/Stack-Rust_2024-8be9fd?style=flat-square&logo=rust&logoColor=white&labelColor=282a36) ![Interface](https://img.shields.io/badge/Interface-FFI_%2F_C--ABI-ff79c6?style=flat-square&logo=c&logoColor=white&labelColor=282a36) ![Storage](https://img.shields.io/badge/Storage-FragmentsDB-ffb86c?style=flat-square&logo=sqlite&logoColor=white&labelColor=282a36) ![Engine](https://img.shields.io/badge/Engine-Tera-bd93f9?style=flat-square&logo=html5&logoColor=white&labelColor=282a36) ![License](https://img.shields.io/badge/License-MIT-50fa7b?style=flat-square&logo=open-source-initiative&logoColor=white&labelColor=282a36)

---

##  Mission

Kitchn unifies the theming and configuration of your entire ecosystem (e.g., Hyprland, Waybar, Alacritty). Instead of editing 10 different config files to change a color or font, you edit **one** central configuration. Kitchn then propagates these changes to all your installed applications ("Ingredients") via powerful templates.

With the new **C-ABI Compatible Core**, Kitchn is no longer just a CLI tool—it's a system-wide SDK that can be embedded into any application.

##  Installation

### Option A: Using Just (Recommended)
```bash
just install
```

### Option B: Manual
```bash
./install.sh
```

Both methods will:
1.  Create `~/.config/kitchn/` with default configurations.
2.  Build release binaries (`kitchn`, `kitchn-log`).
3.  Install them to `~/.local/bin/`.

> [!IMPORTANT]
> Ensure `~/.local/bin` is in your `$PATH`.

---

## Project Structure

```bash
.
├── crates/
│   ├── kitchn_lib/      # Core Logic (Rust 2024)
│   ├── kitchn_ffi/      # FFI Interface (Rust 2021, C-ABI)
│   ├── kitchn_cli/      # CLI wrapper (`kitchn`)
│   └── kitchn_log/      # Logging CLI (`kitchn-log`)
├── include/             # Generated C headers (kitchn.h)
├── assets/
│   ├── ingredients/     # Example .ing files
│   ├── examples/        # C++, Python, Rust integration examples
├── Cargo.toml           # Workspace config
└── justfile             # Command runner
```

### 🧠 Core Architecture

-   **Logic**: `kitchn_lib` (Rust 2024) handles all processing, rendering, and logic.
-   **Interface**: `kitchn_ffi` (Rust 2021) provides a stable C-ABI and auto-generates `kitchn.h` using `cbindgen`.
-   **Storage**: Ingredients are ingested into a high-performance **binary database** (`pastry.bin`) located in `~/.local/share/kitchn/`, ensuring instant access and clean storage.

### 🎨 The "Sweet Dracula" Standard

Kitchn enforces a strict, vibrant Dracula palette across your system:

| Color | Hex | Role | Usage |
|-------|-----|------|-------|
| ![#282a36](https://placehold.co/15x15/282a36/282a36.png) **Background** | `#282a36` | Canvas | Windows, Terminals, Editors |
| ![#44475a](https://placehold.co/15x15/44475a/44475a.png) **Current** | `#44475a` | Selection | Active lines, Hover states |
| ![#f8f8f2](https://placehold.co/15x15/f8f8f2/f8f8f2.png) **Foreground** | `#f8f8f2` | Text | Main content text |
| ![#bd93f9](https://placehold.co/15x15/bd93f9/bd93f9.png) **Purple** | `#bd93f9` | Primary | Accents, Borders, Keywords |
| ![#ff79c6](https://placehold.co/15x15/ff79c6/ff79c6.png) **Pink** | `#ff79c6` | Secondary | Highlights, Strings, Urgent |

---

##  Integration & FFI

`kitchn_lib` exposes a **C-ABI** compatible interface, allowing you to use Kitchn's configuration, logging, and packaging logic in other languages.

### C / C++
Include the header and link against the library:
```cpp
#include "kitchn.h"

KitchnContext* ctx = kitchn_context_new();
kitchn_context_set_app_name(ctx, "MyApp");
kitchn_log_preset(ctx, "boot_ok", NULL);
kitchn_context_free(ctx);
```

### Python
Use `ctypes` to load the shared library:
```python
import ctypes
lib = ctypes.CDLL("libkitchn_ffi.so")
ctx = lib.kitchn_context_new()
```

### Examples
Run the built-in examples to see it in action:
```bash
just examples
# OR specific ones:
just example-cpp
just example-python
just example-rust
```

---

## Commands

### Ingredient Management
```bash
# Install a single ingredient
kitchn stock ./assets/ingredients/waybar.ing

# Cook (Sync) all installed ingredients
kitchn cook

# Clean (Remove) all ingredients from pantry
kitchn pantry clean
```

### Logging
```bash
# Ad-hoc logging
kitchn-log error SYSTEM "Database connection failed"

# Using a preset
kitchn-log boot_ok
```

#### App-Scoped Logging
You can configure Kitchn to organize logs by application name in `layout.toml`:
```toml
path_structure = "{year}/{month}/{app}/{scope}"
app_name = "kitchn" # Default app name
```

Override the app name via CLI:
```bash
kitchn-log boot_ok --app MyApp
```

---

##  Debugging

Kitchn includes a powerful debug mode to diagnose failing hooks or configuration issues.

```bash
kitchn --debug
```

This will spawn a **separate terminal window** (prioritizing `rio`, `alacritty`, `kitty`) that streams verbose logs, including:
- Exact commands executed by hooks
- Stdout/Stderr from hooks (even if empty)
- Configuration files loaded
- Tera template context keys

You can also attach it to specific commands:
```bash
kitchn cook --debug
kitchn bake --debug
```

---

## 🛡️ Robustness

Kitchn enforces a **Single Instance Policy** using OS-level file locking (`flock`). This ensures that only one instance manages the pantry or system configuration at a time, preventing database corruption and conflicts.

-   **Automatic Cleanup**: If Kitchn crashes, the kernel releases the lock immediately.
-   **Non-Blocking**: A second instance will fail immediately with a clear error message instead of hanging.
-   **Debug Exception**: The debug viewer (`kitchn --debug`) is exempt and can run in parallel.

---

##  Ingredients (`.ing`)

An **Ingredient** is a single TOML file that teaches Kitchn how to theme a specific application. Ingredients are **ingested** into the `PastryDB` upon installation, meaning you don't need to keep the original files.

### Structure
```toml
[meta]
id = "waybar"

[[templates]]
target = "~/.config/waybar/style.css"
content = """
window#waybar {
    background-color: {{ colors.bg }};
    border-bottom: 2px solid {{ colors.primary }};
}
"""

[hooks]
reload = "pkill -SIGUSR2 waybar"
```

---

##  Configuration

Located in `~/.config/kitchn/`.

| File | Purpose |
|------|---------|
| `theme.toml` | Colors & Fonts |
| `icons.toml` | Icon abstractions (nerdfont/ascii) |
| `layout.toml` | Log structure & formatting |
| `cookbook.toml` | Pre-defined messages & dictionary |

You may split your configuration using `include = ["path/to/extra.toml"]`.

---

##  Uninstall

```bash
just uninstall
```
