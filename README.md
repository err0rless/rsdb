# rsdb
Linux debugger written in Rust

## Road to version 1.0.0
- [ ] Basic memory reading / writing
- [ ] ELF binary parsing
  - [ ] Entry point
  - [ ] Section
  - [ ] Symbol
  - [ ] Instructions
- [ ] Signal handling
- [ ] Support addtional platforms / environments
  - [ ] ARM64
  - [ ] Android
- [ ] Breakpoints
  - [ ] Conditional breakpoints
- [ ] Codepatching
  - [ ] easy patching `codepatch main+164 "ADD R0, 10"`
  - [ ] Managing patch-points
- [ ] Memory searching
  - [ ] keep tracks of specific memory address
  - [ ] Search strings

## Additionals
- [ ] Memory dumping
  - [ ] Save as file
- [ ] Variables
- [ ] Calculator
- [ ] Enhanced cli
  - [ ] autocomplete
  - [ ] save history
  - [ ] suggestion
- [ ] Scripting
- [ ] Documentation
- [ ] Unittest
- [ ] CI/CD
