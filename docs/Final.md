# Project Title

Team members:

- James Oh
- Matthew DellaNeve
- Renee Veit

## Summary Description

Reiterate the summary description of the overall goal of the project (updated as
necessary from the Proposal and/or Checkpoint documents).

## Project Execution Summary

Describe the work done for the project and lessons learned.

## Additional Details

- List any external Rust crates required for the project (i.e., what
  `[dependencies]` have been added to `Cargo.toml` files).
- Briefly describe the structure of the code (what are the main components, the
  module dependency structure). Why was the project modularized in this way?
- Choose (at least) one code excerpt that is a particularly good example of Rust
  features, idioms, and/or style and describe what makes it “Rusty”.
- Were any parts of the code particularly difficult to expres using Rust? What
  are the challenges in refining and/or refactoring this code to be a better
  example of idiomatic Rust?
- Describe any approaches attempted and then abandoned and the reasons why. What
  did you learn by undertaking this project?
- Review the final project grading rubric and discuss any relevant aspects of
  the project.



### Challenges (just notes as we worked on the project)
1. Left recursive parsing was very tricky
2. First made eval/exec consume AST, but changed to take immutable reference in order to make function work
3. Lifetime parameters were tricky (Function will have reference to block which lives in AST, so the lifetime parameters will basically represent the lifetime of AST tokens, had to expand the lifetime parameters to many structs since lot of them are related, however, it was crucial to not link the lifetime of environment with AST, the lifetime parameter is for LuaValue stored in env, but that doesn't mean env also needs to have equal lifetime as AST) - immutable ref was needed because of function call and loops (need to re-evaluate the expressions)
4. 