#include "hcore.h"
#include <iostream>
#include <vector>

int main() {
  std::cout << "Initializing Hyprcore Context..." << std::endl;

  // 1. Create Context (loads hyprcore.toml)
  HCoreContext *ctx = hcore_context_new();
  if (!ctx) {
    std::cerr << "Failed to create context!" << std::endl;
    return 1;
  }

  // 2. Logging Example
  // This will use the corelog logic from Rust (colors, file logging, etc.)
  hcore_log(ctx, "info", "cpp_example", "Hello from C++ via FFI!");
  hcore_log(ctx, "warn", "cpp_example", "This uses the shared library!");

  std::cout << "Testing Presets..." << std::endl;
  hcore_log_preset(ctx, "test_pass", nullptr);
  hcore_log_preset(ctx, "info", "Overridden preset message from C++!");

  // 3. Error Handling Example (simulated failure)
  std::cout << "\nAttempting to pack a non-existent directory..." << std::endl;
  int result = hcore_pack(ctx, "/path/to/nothing", "output.fpkg");

  if (result != 0) {
    // Retrieve the error message from Rust
    char error_buffer[1024];
    hcore_get_last_error(ctx, error_buffer, 1024);
    std::cerr << "Caught expected error: " << error_buffer << std::endl;
  }

  // 4. Cleanup
  hcore_context_free(ctx);
  std::cout << "\nContext freed. Exiting." << std::endl;

  return 0;
}
