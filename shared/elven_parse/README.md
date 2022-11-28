# elven_parse
A small elf parser used for research and embedded development

## Documentation
You can use cargo doc to check some of the documentation.
>The documentation in the cargo doc is incomplete, and modules do not have comprehensible documentation, but the example below should suffice. For more information regarding the data structures, you could check the source code or use `cargo doc`

## Example
An example of the elf parser

```rust
use elven_parse::Elf;

let file = std::fs::read("/path/to/the/elf/file").unwrap();
// Parse the header and populate the elf struct
let elf = Elf::new(file.as_slice()).parse().unwrap();
// Use the iterators to iterate over the program and section header
for program in elf.program_iter() {}
for section in elf.section_iter() {}

```
