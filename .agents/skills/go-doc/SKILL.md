---
name: go-doc
description: |
  Write documentation for this go directory (in directory and subdirectories
  in `go`). Use when I ask you to write, or update the documentation like
  "write documentation for the function `foo`", or "update the documentation
  for the struct `Bar`". Note that the granularity of the request is not
  always a single function or struct. For example, I might ask you to "write
  documentation for the module `baz`", which would require you to write
  documentation for multiple items in the `baz` module.
---

# Documentation Requirements
The output documentation should follow these requirements:

- Output the generated documentation in English.
- if the documentation is for function, struct, or enum, it should be
  written in Go doc comment format (i.e., using `//`).
- The documentation should be short, clear, concise, and easy to understand.
  It should explain the purpose of the item being documented, how to use it,
  and any important details or caveats that users should be aware of.
