import ctypes
import os
import sys

# Load the shared library
# In a real scenario, this would be in /usr/lib or similar
lib_path = os.path.abspath("../../../target/release/libkitchn_ffi.so")
try:
    kitchn = ctypes.CDLL(lib_path)
except OSError as e:
    print(f"Failed to load library at {lib_path}: {e}")
    sys.exit(1)

# Define opaque pointer type for KitchnContext
class KitchnContext(ctypes.Structure):
    pass

KitchnContext_p = ctypes.POINTER(KitchnContext)

# Define function signatures
kitchn.kitchn_context_new.restype = KitchnContext_p
kitchn.kitchn_context_free.argtypes = [KitchnContext_p]

# New: Set App Name
kitchn.kitchn_context_set_app_name.argtypes = [KitchnContext_p, ctypes.c_char_p]
kitchn.kitchn_context_set_app_name.restype = None

kitchn.kitchn_log.argtypes = [KitchnContext_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
kitchn.kitchn_log.restype = None

kitchn.kitchn_log_preset.argtypes = [KitchnContext_p, ctypes.c_char_p, ctypes.c_char_p]
kitchn.kitchn_log_preset.restype = ctypes.c_int

kitchn.kitchn_pack.argtypes = [KitchnContext_p, ctypes.c_char_p, ctypes.c_char_p]
kitchn.kitchn_pack.restype = ctypes.c_int

kitchn.kitchn_get_last_error.argtypes = [KitchnContext_p, ctypes.c_char_p, ctypes.c_size_t]
kitchn.kitchn_get_last_error.restype = ctypes.c_int

def main():
    print("Initializing Kitchn Context (Python)...")
    
    # 1. Create Context
    ctx = kitchn.kitchn_context_new()
    if not ctx:
        print("Failed to create context!")
        sys.exit(1)

    try:
        # Set App Name
        kitchn.kitchn_context_set_app_name(ctx, b"PythonExample")

        # 2. Logging Example
        print("Sending logs from Python...")
        # strings must be bytes in ctypes
        kitchn.kitchn_log(ctx, b"info", b"python_example", b"Hello from Python via FFI!")
        kitchn.kitchn_log(ctx, b"success", b"python_example", b"Bindings are working!")
        
        print("Testing Presets...")
        kitchn.kitchn_log_preset(ctx, b"test_pass", None)
        kitchn.kitchn_log_preset(ctx, b"info", b"Python preset override!")

        # 3. Error Handling Example
        print("\nAttempting to pack a non-existent directory...")
        result = kitchn.kitchn_pack(ctx, b"/path/to/nothing", b"output.bag")
        
        if result != 0:
            error_buffer = ctypes.create_string_buffer(1024)
            kitchn.kitchn_get_last_error(ctx, error_buffer, 1024)
            print(f"Caught expected error: {error_buffer.value.decode('utf-8')}")

    finally:
        # 4. Cleanup
        kitchn.kitchn_context_free(ctx)
        print("\nContext freed. Exiting.")

if __name__ == "__main__":
    main()
