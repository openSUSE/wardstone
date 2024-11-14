# `wardstone`

![Continuous Integration](https://github.com/openSUSE/wardstone/actions/workflows/ci.yaml/badge.svg)

The `wardstone` project aims to create a library that can be used across different programming languages via a foreign function interface and a command line utility that users can run against their existing keys to detect conformance to varying cryptographic key standards and research publications.

The repository is organised as a series of the following Rust crates:

- [**`wardstone`**](./crates/cmd/). A command-line application that checks cryptographic keys for compliance.
- [**`wardstone_core`**](./crates/core/). A Rust library that curates compliance information for cryptographic keys from varying standards bodies and research groups.
- [**`wardstone_ffi`**](./crates/ffi/). A version of [`wardstone_core`](./crates/core/) that exports a foreign function interface for using the library from C and other languages that support it.

This is a [Google Summer of Code project](https://summerofcode.withgoogle.com/programs/2023/projects/QjOBHrdT) with [openSUSE](https://github.com/openSUSE/mentoring/issues/198).
