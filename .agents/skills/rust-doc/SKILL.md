---
name: rust-doc
description: |
  Write documentation for this rust library. Use when I ask you to write, or
  update documentation in this rust library like
  "write documentation for the function `foo`", or
  "update the documentation for the struct `Bar`".
  Note that the granularity of the request is not always a single function or
  struct. For example, I might ask you to "write documentation for the module
  `baz`", which would require you to write documentation for multiple items
  in the `baz` module.
---

# Documentation Requirements
The output documentation should follow these requirements:

- The documentation should be written in markdown format. However, if
  the documentation is for a function, struct, or enum, it should be written in
  Rust doc comment format (i.e., using `///`).
- If you write an example in the documentation, it should be a complete, and
  runnable example that demonstrates how to use the item being documented.
- **DO NOT use `ignore` in the example code** unless the example is
  intended to be wrong code that demonstrates a common mistake or a pitfall.
- **DO NOT use `no_run` in the example code** unless the example is intended to be
  a code snippet that cannot be run in a test environment (e.g., it requires
  external dependencies or system resources that are not available in the test
  environment).
- The documentation should be clear, concise, and easy to understand. It should
  explain the purpose of the item being documented, how to use it, and any
  important details or caveats that users should be aware of.
- If the item being documented is a function, the documentation should include
  a description of the function's parameters, return value, and any errors that
  it may produce.
- If the item being documented is a struct or enum, the documentation should
  include a description of the struct or enum, its fields or variants, and any
  important details or caveats that users should be aware of.
- If the item being documented is a module, the documentation should include a
  description of the module, its purpose, and any important details or caveats
  that users should be aware of. It should also include a brief overview of the
  items contained in the module.

# Documentation Test
To test the examples in the documentation, use the following command in `rust`
directory (which contains the `Cargo.toml` file):

```bash
make test-doc
```

# Document Generation Command
To generate the documentation, use the following command in the same directory
as the `Cargo.toml` file:

```bash
make doc
```
