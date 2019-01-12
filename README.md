# `fart`: fitzgen's art

My personal tooling and framework for SVG-based generative art. Inspired
[Benjamin Kovach's write up on infrastructure and tooling for generative
art.](https://www.kovach.me/posts/2018-10-13-infrastructure-of-art.html)

## CLI

For a full listing of CLI commands and flags, run `fart help [subcommand]`.

### `fart new $name`

Create a new `fart` project.

### `fart watch [project]`

Watch a `fart` project, automatically build and run it on each change. Saves and
commits time-stamped SVGs for you, so you can see the evolution of your project.

## Library

Contains utilities for generating SVGs and working with numbers and RNGs. Mostly
just re-exports of other crates!

Read the docs:

```
cargo doc --open
```
