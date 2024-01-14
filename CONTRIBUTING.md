<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD033 -->

# Contributing

---

### Reporting a Bug

- Make sure the bug has not already been reported by searching on GitHub under [Issues](https://github.com/lune-org/lune/issues).
- If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/lune-org/lune/issues/new). Be sure to include a **title and description**, as much relevant information as possible, and if applicable, a **code sample** or a **test case** demonstrating the expected behavior.

---

### Contributing - Bug Fixes

1. Make sure an [issue](https://github.com/lune-org/lune/issues) has been created for the bug first, so that it can be tracked and searched for in the repository history. This is not mandatory for small fixes.
2. Open a new GitHub pull request for it. A pull request for a bug fix must include:
   - A clear and concise description of the bug it is fixing.
   - A new test file ensuring there are no regressions after the bug has been fixed.
   - A link to the relevant issue, or a `Fixes #issue` line, if an issue exists.

### Contributing - Features

1. Make sure an [issue](https://github.com/lune-org/lune/issues) has been created for the feature first, so that it can be tracked and searched for in the repository history. If you are making changes to an existing feature, and no issue exists, one should be created for the proposed changes.
2. Any API design or considerations should first be brought up and discussed in the relevant issue, to prevent long review times on pull requests and unnecessary work for maintainers.
3. Familiarize yourself with the codebase and the tools you will be using. Some important parts include:
   - The [mlua](https://crates.io/crates/mlua) library, which we use to interface with Luau.
   - Any [built-in libraries](https://github.com/lune-org/lune/tree/main/src/lune/builtins) that are relevant for your new feature. If you are making a new built-in library, refer to existing ones for structure and implementation details.
   - Our toolchain, notably [StyLua](https://github.com/JohnnyMorganz/StyLua), [rustfmt](https://github.com/rust-lang/rustfmt), and [clippy](https://github.com/rust-lang/rust-clippy). If you do not use these tools there is a decent chance CI will fail on your pull request, blocking it from getting approved.
4. Write some code!
5. Open a new GitHub pull request. A pull request for a feature must include:
   - A clear and concise description of the new feature or changes to the feature.
   - Test files for any added or changed functionality.
   - A link to the relevant issue, or a `Closes #issue` line.

### Contributing - Formatting & Cosmetic Changes

Changes that are purely cosmetic, and do not add to the stability, functionality, or testability of Lune, will generally not be accepted unless there has been previous discussion about the changes being made.

### Contributing - Documentation

#### Documentation Site

Check out the [docs](https://github.com/lune-org/docs) repository and its contribution guidelines.

#### Type Definitions

If type definitions for built-in libraries need improvements:

1. Check out the [types](https://github.com/lune-org/lune/tree/main/types) directory at the root of the repository.
2. Make the desired changes, and verify that they have the desired outcome.
3. Open a new GitHub pull request for your changes.

---

### Publishing a Release

The Lune release process is semi-automated, and takes care of most things for you. Here's how to create a new release:

1. Make sure the changelog is up to date and contains all of the changes since the last release.
2. Add the release date in the changelog + set a new version number in `Cargo.toml`.
3. Commit and push changes from step 2 to GitHub. This will automatically publish the Lune library to [crates.io](https://crates.io) when the version number changes.
4. Trigger the [release](https://github.com/lune-org/lune/actions/workflows/release.yaml) workflow on GitHub manually, and wait for it to finish. Find the new pending release in the [Releases](https://github.com/lune-org/lune/releases) section.
5. Add in changes from the changelog for the new pending release into the description, hit "accept" on creating a new version tag, and publish 🚀

---

If you have any questions, check out the `#lune` channel in the [Roblox OSS discord](https://discord.gg/H9WqmFAB5Y), where most of our realtime discussion takes place!

Thank you for contributing to Lune! 🌙
