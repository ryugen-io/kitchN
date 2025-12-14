# CI Fix (2025-12-12)

Fixed CI compilation errors by switching to `nightly` toolchain.
- Problem: `cargo check` failed in CI because `stable` toolchain does not yet support `edition = "2024"`.
- Solution: Updated `.github/workflows/ci.yml` and `.github/workflows/release.yml` to use `dtolnay/rust-toolchain@nightly`.
