# Kitchn - Task Completion Checklist

## Before Committing

### Code Quality
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes (or run `cargo fmt`)
- [ ] No new warnings introduced

### Testing
- [ ] `cargo test --workspace` passes
- [ ] New functionality has tests
- [ ] CLI changes tested with `assert_cmd`

### Documentation
- [ ] Public items have doc comments
- [ ] README updated if user-facing changes
- [ ] CHANGELOG.md updated for releases

## Crate-Specific

### k-lib Changes
- [ ] Error types use `thiserror`
- [ ] Config structs derive `Serialize, Deserialize`
- [ ] Binary serialization tested if modified

### k-ffi Changes
- [ ] Functions are `#[no_mangle] extern "C"`
- [ ] Memory management is correct (new/free pairs)
- [ ] Header regenerated: `cbindgen --config cbindgen.toml --crate k-ffi -o include/kitchn.h`
- [ ] FFI examples still work: `just examples`

### CLI Changes
- [ ] Help text is clear (doc comments)
- [ ] Error messages are user-friendly
- [ ] Debug mode tested: `kitchn --debug`

## Release Checklist
- [ ] Version bumped in all Cargo.toml files
- [ ] CHANGELOG.md updated
- [ ] `just package` creates valid tarball
- [ ] Install script tested: `./install.sh`
