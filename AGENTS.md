## General Guidelines
Write succint, to the point code
Avoid unnecessary early outs (checking for empty collections before iteration, zeros before adding, ones before multipltying and similar situations)
Prefer simple clear code over micro-optimized code.
After a refactor, recursively follow up with cleanup to avoid code structure we would not create from scratch.
Callee come before callers in a file.
Test must be useful and test something meaningful. Tests content and names describe the current state of the code.
When asked to optimize performance, always measure before/after to make sure changes really speed things up.

### Committing
Split separable work into separate commits, with one step or concern per commit.
Each commit should pass: `cargo check`
Prefix refactoring commit messages with `refactor: `
