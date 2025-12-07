#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Opaque context pointer (wraps HyprConfig with error storage)
 */
typedef struct HCoreContext HCoreContext;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

struct HCoreContext *hcore_context_new(void);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext` created by `hcore_context_new`.
 */
void hcore_context_free(struct HCoreContext *ctx);

/**
 * Copies the last error message into the provided buffer.
 * Returns the number of bytes written (excluding null terminator),
 * or -1 if the buffer was too small or no error exists.
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext` created by `hcore_context_new`.
 * * `buffer` must be a valid pointer to a writable memory region of at least `len` bytes.
 */
int hcore_get_last_error(struct HCoreContext *ctx, char *buffer, uintptr_t len);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext`.
 * * `level`, `scope`, and `msg` must be valid, null-terminated C strings.
 */
void hcore_log(struct HCoreContext *ctx, const char *level, const char *scope, const char *msg);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext`.
 * * `src_dir` and `out_file` must be valid, null-terminated C strings.
 */
int hcore_pack(struct HCoreContext *ctx, const char *src_dir, const char *out_file);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext`.
 * * `pkg_file` and `target_dir` must be valid, null-terminated C strings.
 */
int hcore_unpack(struct HCoreContext *ctx, const char *pkg_file, const char *target_dir);

/**
 * # Safety
 *
 * This function is unsafe because it dereferences raw pointers.
 * * `ctx` must be a valid pointer to `HCoreContext`.
 * * `path` must be a valid, null-terminated C string.
 */
int hcore_install(struct HCoreContext *ctx, const char *path);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
