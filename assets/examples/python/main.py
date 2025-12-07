import ctypes
import os
import sys

# Load the shared library
# In a real scenario, this would be in /usr/lib or similar
lib_path = os.path.abspath("../../../target/release/libhcore_ffi.so")
try:
    hcore = ctypes.CDLL(lib_path)
except OSError as e:
    print(f"Failed to load library at {lib_path}: {e}")
    sys.exit(1)

# Define opaque pointer type for HCoreContext
class HCoreContext(ctypes.Structure):
    pass

HCoreContext_p = ctypes.POINTER(HCoreContext)

# Define function signatures
hcore.hcore_context_new.restype = HCoreContext_p
hcore.hcore_context_free.argtypes = [HCoreContext_p]

hcore.hcore_log.argtypes = [HCoreContext_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
hcore.hcore_log.restype = None

hcore.hcore_log_preset.argtypes = [HCoreContext_p, ctypes.c_char_p, ctypes.c_char_p]
hcore.hcore_log_preset.restype = ctypes.c_int

hcore.hcore_pack.argtypes = [HCoreContext_p, ctypes.c_char_p, ctypes.c_char_p]
hcore.hcore_pack.restype = ctypes.c_int

hcore.hcore_get_last_error.argtypes = [HCoreContext_p, ctypes.c_char_p, ctypes.c_size_t]
hcore.hcore_get_last_error.restype = ctypes.c_int

def main():
    print("Initializing Hyprcore Context (Python)...")
    
    # 1. Create Context
    ctx = hcore.hcore_context_new()
    if not ctx:
        print("Failed to create context!")
        sys.exit(1)

    try:
        # 2. Logging Example
        print("Sending logs from Python...")
        # strings must be bytes in ctypes
        hcore.hcore_log(ctx, b"info", b"python_example", b"Hello from Python via FFI!")
        hcore.hcore_log(ctx, b"success", b"python_example", b"Bindings are working!")
        
        print("Testing Presets...")
        hcore.hcore_log_preset(ctx, b"test_pass", None)
        hcore.hcore_log_preset(ctx, b"info", b"Python preset override!")

        # 3. Error Handling Example
        print("\nAttempting to pack a non-existent directory...")
        result = hcore.hcore_pack(ctx, b"/path/to/nothing", b"output.fpkg")
        
        if result != 0:
            error_buffer = ctypes.create_string_buffer(1024)
            hcore.hcore_get_last_error(ctx, error_buffer, 1024)
            print(f"Caught expected error: {error_buffer.value.decode('utf-8')}")

    finally:
        # 4. Cleanup
        hcore.hcore_context_free(ctx)
        print("\nContext freed. Exiting.")

if __name__ == "__main__":
    main()
