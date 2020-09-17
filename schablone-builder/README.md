# schablone
schablone is a german word, meaning template.
It's a command-line utility that creates projects from a schablone (template).

Schablone is currently still in early stages and heavy development, so a lot will change.

# Tera
The templating engine that is used by schablone is [tera](https://tera.netlify.app/).

# Installation
```sh
cargo install schablone
```

# Usage
```sh
Build a schablone from template

USAGE:
    schablone [FLAGS] [OPTIONS] <name> <target>

FLAGS:
    -d               Do not actually create the folders and files at the destination. Useful for testing templates and
                     parameters
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p <parameters>             Parameters to render as KEY=VALUE pairs separated by a comma. These take precedence over
                                the 'parameters_file'
    -f <parameters_file>        Parameters to render, given the path to a JSON file

ARGS:
    <name>      Name of the schablone
    <target>    Where to write the results
```
