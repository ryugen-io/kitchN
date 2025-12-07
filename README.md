# Hyprcore

**Strict Corporate Design Enforcement for Hyprland.**

> "Single Source of Truth". One config change propagates to Shells, Scripts, Logs, GUIs, and TUI apps instantly.
> Now with C-API support for C++, Python, and more.

![Stack](https://img.shields.io/badge/Stack-Rust_2024-8be9fd?style=flat-square&logo=rust&logoColor=white&labelColor=282a36) ![Interface](https://img.shields.io/badge/Interface-FFI_%2F_C--ABI-ff79c6?style=flat-square&logo=c&logoColor=white&labelColor=282a36) ![Storage](https://img.shields.io/badge/Storage-FragmentsDB-ffb86c?style=flat-square&logo=sqlite&logoColor=white&labelColor=282a36) ![Engine](https://img.shields.io/badge/Engine-Tera-bd93f9?style=flat-square&logo=html5&logoColor=white&labelColor=282a36) ![License](https://img.shields.io/badge/License-MIT-50fa7b?style=flat-square&logo=open-source-initiative&logoColor=white&labelColor=282a36)

---

##  Mission

Hyprcore unifies the theming and configuration of your entire **Hyprland** ecosystem. Instead of editing 10 different config files to change a color or font, you edit **one** central configuration. Hyprcore then propagates these changes to all your installed applications ("Fragments") via powerful templates.

With the new **C-ABI Compatible Core**, Hyprcore is no longer just a CLI tool—it's a system-wide SDK that can be embedded into any application.

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
1.  Create `~/.config/hyprcore/` with default configurations.
2.  Build release binaries (`hyprcore`, `corelog`) and shared libraries (`libhcore_lib.so`).
3.  Install them to `~/.local/bin/` and `~/.local/lib/`.

> [!IMPORTANT]
> Ensure `~/.local/bin` is in your `$PATH` and `LD_LIBRARY_PATH` includes user lib directories if needed.

---

## Project Structure

```bash
.
├── crates/
│   ├── hcore_lib/       # Core Logic (Rust 2024)
│   ├── hcore_ffi/       # FFI Interface (Rust 2021, C-ABI)
│   ├── hcore_cli/       # CLI wrapper (`hyprcore`)
│   └── hcore_log/       # Logging CLI (`corelog`)
├── include/             # Generated C headers (hcore.h)
├── assets/
│   ├── fragments/       # Example .frag files
│   ├── examples/        # C++, Python, Rust integration examples
├── Cargo.toml           # Workspace config
└── justfile            # Command runner
```

### 🧠 Core Architecture

-   **Logic**: `hcore_lib` (Rust 2024) handles all processing, rendering, and logic.
-   **Interface**: `hcore_ffi` (Rust 2021) provides a stable C-ABI and auto-generates `hcore.h` using `cbindgen`.
-   **Storage**: Fragments are ingested into a high-performance **binary database** (`fragments.bin`) located in `~/.local/share/hyprcore/`, ensuring instant access and clean storage.

### 🎨 The "Sweet Dracula" Standard

Hyprcore enforces a strict, vibrant Dracula palette across your system:

| Color | Hex | Role | Usage |
|-------|-----|------|-------|
| ![#282a36](https://placehold.co/15x15/282a36/282a36.png) **Background** | `#282a36` | Canvas | Windows, Terminals, Editors |
| ![#44475a](https://placehold.co/15x15/44475a/44475a.png) **Current** | `#44475a` | Selection | Active lines, Hover states |
| ![#f8f8f2](https://placehold.co/15x15/f8f8f2/f8f8f2.png) **Foreground** | `#f8f8f2` | Text | Main content text |
| ![#bd93f9](https://placehold.co/15x15/bd93f9/bd93f9.png) **Purple** | `#bd93f9` | Primary | Accents, Borders, Keywords |
| ![#ff79c6](https://placehold.co/15x15/ff79c6/ff79c6.png) **Pink** | `#ff79c6` | Secondary | Highlights, Strings, Urgent |
| ![#ffb86c](https://placehold.co/15x15/ffb86c/ffb86c.png) **Orange** | `#ffb86c` | Functions | Methods, Parameters |
| ![#f1fa8c](https://placehold.co/15x15/f1fa8c/f1fa8c.png) **Yellow** | `#f1fa8c` | Classes | Types, Structs |
| ![#8be9fd](https://placehold.co/15x15/8be9fd/8be9fd.png) **Cyan** | `#8be9fd` | Constants | Literals, Macros |
| ![#50fa7b](https://placehold.co/15x15/50fa7b/50fa7b.png) **Green** | `#50fa7b` | Success | Strings, Validations |
| ![#ff5555](https://placehold.co/15x15/ff5555/ff5555.png) **Red** | `#ff5555` | Error | Deletions, Failures |


---

##  Integration & FFI

`hcore_lib` exposes a **C-ABI** compatible interface, allowing you to use Hyprcore's configuration, logging, and packaging logic in other languages.

### C / C++
Include the header and link against the library:
```cpp
#include "hcore.h"

HCoreContext* ctx = hcore_context_new();
hcore_log(ctx, "info", "cpp_app", "Connected to Hyprcore!");
hcore_context_free(ctx);
```

### Python
Use `ctypes` to load the shared library:
```python
import ctypes
lib = ctypes.CDLL("libhcore_lib.so")
ctx = lib.hcore_context_new()
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

## Workflow

```mermaid
%%{
  init: {
    'theme': 'base',
    'themeVariables': {
      'primaryColor': '#282a36',
      'primaryTextColor': '#f8f8f2',
      'primaryBorderColor': '#bd93f9',
      'lineColor': '#6272a4',
      'secondaryColor': '#44475a',
      'tertiaryColor': '#44475a'
    }
  }
}%%
graph TD
    subgraph Configuration [User Configuration]
        Theme[theme.toml]
        Icons[icons.toml]
        Layout[layout.toml]
        Dictionary[dictionary.toml]
    end

    subgraph Core [hcore_lib SDK]
        ConfigEngine[Config Engine]
        Renderer[Tera Renderer]
        FFI[C-ABI Interface]
    end

    subgraph Tools [Official Tools]
        CLI[hyprcore CLI]
        Log[corelog CLI]
    end

    subgraph External [3rd Party Apps]
        PyApp[Python Scripts]
        CppApp[Native C++ Apps]
    end

    Configuration --> ConfigEngine
    Dictionary --> Log
    ConfigEngine --> Renderer
    ConfigEngine --> FFI
    
    FFI --> External
    FFI --> Tools
    
    Renderer -->|Generate| ConfigFiles[System Configs]
```

---

## Commands

### Fragment Management
```bash
# Install a single fragment
hyprcore install ./assets/fragments/waybar.frag

# Install a fragment package (.fpkg)
hyprcore install my-theme.fpkg

# Pack fragments into a package
hyprcore pack ./my-fragments -o my-theme.fpkg

# Sync all installed fragments
hyprcore sync
```

### Logging
```bash
# Ad-hoc logging
corelog error SYSTEM "Database connection failed"

# Using a preset
corelog boot_ok
```

---

##  Fragments (`.frag`)

A **Fragment** is a single TOML file that teaches Hyprcore how to theme a specific application. Fragments are **ingested** into the `FragmentsDB` upon installation, meaning you don't need to keep the original `.frag` files.

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

Located in `~/.config/hyprcore/`.

| File | Purpose |
|------|---------|
| `theme.toml` | Colors & Fonts |
| `icons.toml` | Icon abstractions (nerdfont/ascii) |
| `layout.toml` | Log structure & formatting |
| `dictionary.toml` | Pre-defined messages |

You may split your configuration using `include = ["path/to/extra.toml"]`.

---

##  Uninstall

```bash
just uninstall
```
