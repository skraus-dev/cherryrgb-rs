# Contributing to cherryrgb-rs

Welcome to cherryrgb-rs! We're excited to have you as a potential contributor.

Please read and follow these guidelines to ensure smooth collaboration and maintain the quality of our codebase.

Before you start contributing, check our issue tracker to see if there are any open issues that you would like to work on.
If you're proposing a new feature or change, please create a new issue first to discuss it with the community.

Starting off, you want to clone the project's repository and build it.

    git clone https://github.com/skraus-dev/cherryrgb-rs
    $ cd cherryrgb-rs
    $ cargo build --all


Patch the code. Write some tests. Ensure that your code is properly formatted
with `cargo fmt`. Ensure that everything builds and works as expected. Ensure
that you did not break anything. Check for common code anti-pattern via `cargo clippy`.

- For creating commit messages, please refer to [conventionalcommits](https://www.conventionalcommits.org/en/v1.0.0/)
- When opening a PR, please use the provided Pull request template

## Respect the copyright

As this is a project based on reverse-engineered work, I feel this needs to be mentioned.
Please do not upload possibly copyrighted material directly into the repository.

Rather reference the original binary URL and steps how to acquire the target file from
it (e.g. by extraction or local modification of the binary).

An example of that can be seen in [RESEARCH.md](./RESEARCH.md).