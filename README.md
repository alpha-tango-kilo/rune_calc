# lies_of_ergo

Tells you how many of which Ergo items you ought to use to get to your desired amount

## Installation

```shell
cargo install --git https://codeberg.org/alpha-tango-kilo/elden_runes --branch lies-of-p
```

(Add `--force` if updating)

## Usage

```
lies_of_ergo <command> [<args>]

Tells you the optimal Ergo items to use to reach your desired amount in Lies of P

Options:
  --help            display usage information

Commands:
  init              Initialise a new lies_of_ergo file (defaults to
                    ./lies_of_ergo)
  calc              Perform an Ergo calculation
  info              Tells you how many Ergo each item gives, in a neat table
```

### Calculate

```
lies_of_ergo calc [-h <have>] -w <want> [--file <file>] [-v] [--no-inv]

Perform an Ergo calculation

Options:
  -h, --have        how much Ergo you have
  -w, --want        how much Ergo you want to have
  --file            where to get the Ergo file from (defaults to ./lies_of_ergo)
  -v, --verbose     will provide extra information & statistics in output
  --no-inv          disable inventory look-up (prevents auto-discovery)
  --help            display usage information
```

### Initialise

```
lies_of_ergo init [<path>]

Initialise a new lies_of_ergo file (defaults to ./lies_of_ergo)

Positional Arguments:
  path

Options:
  --help            display usage information
```

### Information

```
lies_of_ergo info [<path>] [--with-inv]

Tells you how many Ergo each item gives, in a neat table

Positional Arguments:
  path

Options:
  --with-inv        show the quantities from your inventory alongside the table
                    (looks in ./elden_runes by default)
  --help            display usage information
```
