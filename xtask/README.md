# Xtask - DevOps Script

## Getting started

To download all dependencies run `cargo xtask install`.

If you want to see the coverage locally on your editor then install an extension that can read a local `lcov.info` file and
turn that into realtime data in your IDE. `lcov` format is universal so all mainstream IDEs will have an extension to handle
this format.

- VSCode: [coverage-gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters)

## Summary

This xtask script simplifies the way we utilize github actions and other devops scripts;
it allows us to recreate what our CI does locally before we push to github.
The following are some useful local commands:

```
# Generates coverage files within `coverage/*` and checks formatting.
cargo xtask workflow dev
```
```
# Identical to `dev` workflow but will update coverage files on source change.
cargo xtask workflow dev-watch
```
```
# Passes iff the change would be accepted by github pr-validation.
cargo xtask workflow pr-validation
```

Additional details and commands can be found by running the `--help` flag on the cli.


## Troubleshooting

The xtask code should be easy to read, so any issues can be easily re-created by watching
what steps are running and then running the command that's within that step. Imagine this
is a bash script but in rust.

**Note:** There may be some weirdness around `cargo nightly`.
If you run into problems with that open an issue.

## What is the `cargo-xtask` pattern?
cargo-xtask is way to add free-form automation to a Rust project, a-la make, npm run or bespoke bash scripts.

The two distinguishing features of xtask are:
- It doesn't require any other binaries besides cargo and rustc, it fully bootstraps from them
- Unlike bash, it can more easily be cross platform, as it doesn't use the shell.


**Read more here: https://github.com/matklad/cargo-xtask**