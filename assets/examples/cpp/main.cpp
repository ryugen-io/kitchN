#include "kitchn.h"
#include <iostream>
#include <vector>

int main() {
  std::cout << "Initializing Kitchn Context..." << std::endl;

  // 1. Create Context (loads cookbook.toml)
  KitchnContext *ctx = kitchn_context_new();
  if (!ctx) {
    std::cerr << "Failed to create context!" << std::endl;
    return 1;
  }
  
  // Set App Name (New Feature)
  kitchn_context_set_app_name(ctx, "CppExample");

  // 2. Logging Example
  // This will use the kitchn logic from Rust (colors, file logging, etc.)
  kitchn_log(ctx, "info", "cpp_example", "Hello from C++ via FFI!");
  kitchn_log(ctx, "warn", "cpp_example", "This uses the shared library!");

  std::cout << "Testing Presets..." << std::endl;
  kitchn_log_preset(ctx, "test_pass", nullptr);
  kitchn_log_preset(ctx, "info", "Overridden preset message from C++!");

  // 3. Error Handling Example (simulated failure)
  std::cout << "\nAttempting to pack a non-existent directory..." << std::endl;
  int result = kitchn_pack(ctx, "/path/to/nothing", "output.pastry");

  if (result != 0) {
    // Retrieve the error message from Rust
    char error_buffer[1024];
    kitchn_get_last_error(ctx, error_buffer, 1024);
    std::cerr << "Caught expected error: " << error_buffer << std::endl;
  }

  // 4. Cleanup
  kitchn_context_free(ctx);
  std::cout << "\nContext freed. Exiting." << std::endl;

  return 0;
}
