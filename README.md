# elden_runes

Tells you how many of which rune items you ought to use to get to your desired amount.
Not very smart yet, but I'm working on it.

## Installation

```shell
cargo install --git https://codeberg.org/alpha-tango-kilo/elden_runes
```

(Add `--force` if updating)

## Usage

```
elden_runes <command> [<args>] 

Tells you the optimal rune items to use to reach your desired amount in Elden Ring

Options:
  --help            display usage information

Commands:
  init              Initialise a new elden_runes file
  calc              Perform a rune calculation
```

### Calculate

```
elden_runes calc [-h <have>] -w <want> [--file <file>] [-v] 

Perform a rune calculation

Options:
  -h, --have        how many runes you have
  -w, --want        how many runes you want to have
  --file            where to get the runes file from (defaults to ./elden_runes)
  -v, --verbose     will provide extra information & statistics in output
  --help            display usage information
```

### Initialise

```
elden_runes init [<path>]

Initialise a new elden_runes file (defaults to ./elden_runes)

Positional Arguments:
  path

Options:
  --help            display usage information
```
