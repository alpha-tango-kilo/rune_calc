use std::{
    cmp::Ordering,
    fmt,
    fs::{File, OpenOptions},
    io,
    io::{Read, Write},
    ops::{Deref, DerefMut, SubAssign},
    path::{Path, PathBuf},
    process,
    slice::SliceIndex,
};

use anyhow::{anyhow, bail, Context};
use argh::FromArgs;
use comfy_table::Table;

const RUNE_NAMES: [&str; 13] = [
    "Dim Ergo Fragment",
    "Vivid Ergo Fragment",
    "Radiant Ergo Fragment",
    "Resplendent Ergo Fragment",
    "Dim Ergo Chunk",
    "Vivid Ergo Chunk",
    "Radiant Ergo Chunk",
    "Resplendent Ergo Chunk",
    "Dim Ergo Crystal",
    "Vivid Ergo Crystal",
    "Radiant Ergo Crystal",
    "Resplendent Ergo Crystal",
    "Ergo Crystal of the Eternal",
];
const RUNE_VALUES: [u32; 13] = [
    100, 300, 500, 700, 1000, 1500, 2000, 3000, 5000, 7000, 10000, 15000, 25000,
];

#[derive(FromArgs)]
/// Tells you the optimal Ergo items to use to reach your desired amount in
/// Lies of P
struct WhatDo {
    #[argh(subcommand)]
    subcommand: DoThis,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum DoThis {
    Init(Initialise),
    Calc(Calculation),
    Info(Information),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "calc")]
/// Perform an Ergo calculation
struct Calculation {
    /// how much Ergo you have
    #[argh(option, short = 'h', default = "0")]
    have: u32,
    /// how much Ergo you want to have
    #[argh(option, short = 'w')]
    want: u32,
    /// where to get the Ergo file from (defaults to ./lies_of_ergo)
    #[argh(option, default = "default_path()")]
    file: PathBuf,
    /// will provide extra information & statistics in output
    #[argh(switch, short = 'v')]
    verbose: bool,
    /// disable inventory look-up (prevents auto-discovery)
    #[argh(switch)]
    no_inv: bool,
}

impl Calculation {
    fn run(&self) -> anyhow::Result<()> {
        if self.have >= self.want {
            bail!("you already have all the Ergo you need");
        }
        println!(
            "You have {} Ergo, and you want {} Ergo, right?",
            self.have, self.want
        );
        let (outcome, inventory) = match self.no_inv {
            false => match File::open(&self.file) {
                Ok(mut handle) => {
                    let mut inventory = RuneCount::load(&mut handle)?;
                    (self.with_inventory(&mut inventory)?, Some(inventory))
                },
                Err(_) => (self.without_inventory(), None),
            },
            true => (self.without_inventory(), None),
        };

        println!("You need to use:\n{}", outcome.format_as_list(self.verbose));
        if self.verbose {
            let stats = VerboseStats::new(self.have, self.want, outcome);
            println!("{stats}");
        }

        // Update inventory file if desired
        if let Some(inventory) = inventory {
            if yay_nay_prompt("Do you want to update your inventory file?")
                .unwrap_or_default()
            {
                inventory.save(&self.file)?;
            }
        }
        Ok(())
    }

    fn solve(need: u32, inventory: Option<RuneCount>) -> RuneCount {
        Calculation::_solve(
            need,
            RuneCount::default(),
            RUNE_VALUES.len(),
            inventory,
        )
        .expect(
            "calculations should be checked before using Calculation::solve",
        )
    }

    fn _solve(
        need: u32,
        partial_solution: RuneCount,
        big_index: usize,
        inventory: Option<RuneCount>,
    ) -> Option<RuneCount> {
        // Get the index of the smallest rune that is big enough to entirely
        // fulfill our need
        let closest_bigger_index = RUNE_VALUES[..big_index]
            .iter()
            .enumerate()
            .find(|(index, val)| {
                inventory.map(|inv| inv.has(*index)).unwrap_or(true)
                    && **val >= need
            })
            .map(|(index, _)| index);
        // Add this rune to a solution, which necessarily will have fewer runes
        // in it than the other approach
        let fewer_runes_solution = closest_bigger_index.map(|index| {
            let mut me = partial_solution;
            me[index] += 1;
            me
        });

        // Update big_index to ensure that the rune chosen is smaller than
        // closest_bigger_index
        // The above would be bad if it were made possible because then both
        // solutions could end up being the same, which is inefficient
        let big_index = closest_bigger_index.unwrap_or(big_index - 1);

        // This is similar to how we arrive at the fewer runes solution, but we
        // don't need and intermediary variable so we just smash it all
        // together with and_then directly after the find. and_then is used
        // instead of map as the recursive process can fail
        let more_runes_solution = RUNE_VALUES[..big_index]
            .iter()
            .enumerate()
            .rfind(|(index, val)| {
                inventory.map(|inv| inv.has(*index)).unwrap_or(true)
                    && **val < need
            })
            .and_then(|(index, val)| {
                // Get all our variables in order ready to recurse!
                let mut me = partial_solution;
                me[index] += 1;
                let new_inv = inventory.map(|inv| {
                    let mut new_inv = inv;
                    new_inv[index] -= 1;
                    new_inv
                });
                let need = need - *val;
                Calculation::_solve(need, me, big_index, new_inv)
            });

        match (fewer_runes_solution, more_runes_solution) {
            // In the case where we have two solutions, take the most efficient
            // one
            (Some(a), Some(b)) => {
                // Both solutions are guaranteed to give enough runes, so
                // whichever one gives less will be most efficient. Prefer a
                // when equal as a is the fewer runes solution
                use std::cmp::Ordering::*;
                match a.cmp(&b) {
                    Less | Equal => Some(a),
                    Greater => Some(b),
                }
            },
            // Otherwise, whichever works
            (a, b) => a.or(b),
        }
    }

    fn with_inventory(
        &self,
        inventory: &mut RuneCount,
    ) -> anyhow::Result<RuneCount> {
        let need = self.want - self.have;
        let inv_total = inventory.total();
        if inv_total < need {
            let short = need - inv_total;
            bail!(
                "you don't have enough Ergo items to reach your target, \
                 you'll be {short} Ergo short"
            );
        }
        let solution = Calculation::solve(need, Some(*inventory));
        *inventory -= solution;
        Ok(solution)
    }

    fn without_inventory(&self) -> RuneCount {
        Calculation::solve(self.want - self.have, None)
    }
}

impl Default for Calculation {
    fn default() -> Self {
        Calculation {
            have: 0,
            want: 0,
            file: default_path(),
            verbose: false,
            no_inv: false,
        }
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "init")]
/// Initialise a new lies_of_ergo file (defaults to ./lies_of_ergo)
struct Initialise {
    #[argh(positional, default = "default_path()")]
    path: PathBuf,
}

impl Initialise {
    const TEMPLATE: &'static str = include_str!("../lies_of_ergo_template");

    fn run(&self) -> anyhow::Result<()> {
        let mut handle = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.path)?;
        handle.write_all(Self::TEMPLATE.as_bytes())?;
        eprintln!("Successfully created {}", self.path.to_string_lossy());
        Ok(())
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "info")]
/// Tells you how many Ergo each item gives, in a neat table
struct Information {
    /// show the quantities from your inventory alongside the table (looks in
    /// ./lies_of_ergo by default)
    #[argh(switch)]
    with_inv: bool,
    #[argh(positional, default = "default_path()")]
    path: PathBuf,
}

impl Information {
    fn run(&self) -> anyhow::Result<()> {
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);
        let inv = if self.with_inv {
            match File::open(&self.path) {
                Ok(mut handle) => Some(RuneCount::load(&mut handle)?),
                Err(why) => {
                    eprintln!("Warning: failed to load inventory: {why}");
                    None
                },
            }
        } else {
            None
        };
        match inv {
            None => {
                table.set_header(["Item name", "Ergo value"]);
                RUNE_NAMES.into_iter().zip(RUNE_VALUES).for_each(
                    |(name, value)| {
                        table.add_row(&[String::from(name), value.to_string()]);
                    },
                );
            },
            Some(inv) => {
                table.set_header([
                    "Item name",
                    "Ergo value",
                    "You have",
                    "Total value",
                ]);
                RUNE_NAMES
                    .into_iter()
                    .zip(RUNE_VALUES)
                    .zip(inv.iter())
                    .for_each(|((name, value), count)| {
                        let total = value * *count;
                        table.add_row(&[
                            String::from(name),
                            value.to_string(),
                            count.to_string(),
                            total.to_string(),
                        ]);
                    });
                table.add_row(&[
                    String::new(),
                    String::new(),
                    String::from("Overall total:"),
                    inv.total().to_string(),
                ]);
            },
        }
        println!("{table}");
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
struct RuneCount([u32; 20]);

impl RuneCount {
    fn has(&self, index: usize) -> bool {
        self[index] > 0
    }

    fn total(&self) -> u32 {
        self.slice_total(..)
    }

    fn slice_total<R>(&self, range: R) -> u32
    where
        R: SliceIndex<[u32], Output = [u32]>,
    {
        self.0[range]
            .iter()
            .zip(RUNE_VALUES)
            .fold(0, |acc, (count, val)| {
                acc.saturating_add(count.saturating_mul(val))
            })
    }

    fn format_as_list(&self, extras: bool) -> String {
        let mut buf = String::new();
        self.into_iter()
            .enumerate()
            .rev() // Give runes biggest to smallest
            .filter(|(_, count)| *count > 0)
            .for_each(|(index, count)| {
                let amount_given = RUNE_VALUES[index] * count;
                buf.push_str("- ");
                buf.push_str(&count.to_string());
                buf.push_str("x ");
                buf.push_str(RUNE_NAMES[index]);
                if extras {
                    buf.push_str(" (giving ");
                    buf.push_str(&amount_given.to_string());
                    buf.push(')');
                }
                buf.push('\n');
            });
        buf.pop(); // Removes final newline
        buf
    }

    fn load(read_handle: &mut File) -> anyhow::Result<Self> {
        let mut contents = String::new();
        read_handle.read_to_string(&mut contents)?;
        let mut counts = RuneCount([0; 20]);
        contents.lines().enumerate().try_for_each(
            |(line_no, line)| -> anyhow::Result<()> {
                let (count, name) = line.split_once("x ").context(anyhow!(
                    "line {line_no}: missing delimiter between quantity and \
                     item name"
                ))?;
                let count = count
                    .parse::<u32>()
                    .context(anyhow!("line {line_no}: bad item quantity"))?;
                let (index, _) = RUNE_NAMES
                    .iter()
                    .enumerate()
                    .find(|(_, rune)| **rune == name)
                    .context(anyhow!(
                        "line {line_no}: couldn't match item name"
                    ))?;
                counts[index] += count;
                Ok(())
            },
        )?;
        Ok(counts)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut handle =
            OpenOptions::new().write(true).truncate(true).open(path)?;
        self.into_iter().zip(RUNE_NAMES).try_for_each(
            |(count, name)| -> io::Result<()> {
                handle.write_all(count.to_string().as_bytes())?;
                handle.write_all(b"x ")?;
                handle.write_all(name.as_bytes())?;
                handle.write_all(b"\n")
            },
        )
    }

    #[cfg(test)]
    fn single(index: usize) -> Self {
        let mut rc = RuneCount::default();
        rc[index] = 1;
        rc
    }
}

impl Deref for RuneCount {
    type Target = [u32; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RuneCount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for RuneCount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuneCount {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_total = self.total();
        let other_total = other.total();
        my_total.cmp(&other_total)
    }
}

impl SubAssign for RuneCount {
    fn sub_assign(&mut self, rhs: Self) {
        self.iter_mut().zip(rhs.iter()).for_each(|(a, b)| *a -= *b);
    }
}

#[derive(Debug)]
struct VerboseStats {
    to_consume: u32,
    before_spending: u32,
    after_spending: u32,
}

impl VerboseStats {
    fn new(have: u32, want: u32, consuming: RuneCount) -> Self {
        let to_consume = consuming.total();
        let before_spending = have + to_consume;
        let after_spending = before_spending - want;
        VerboseStats {
            to_consume,
            before_spending,
            after_spending,
        }
    }
}

impl fmt::Display for VerboseStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "In total, you're consuming {} Ergo, which will result in you \
             having {} Ergo, leaving {} after spending",
            self.to_consume, self.before_spending, self.after_spending,
        )
    }
}

fn main() {
    use DoThis::*;
    let what_do: WhatDo = argh::from_env();
    let result = match what_do.subcommand {
        Init(init) => init.run(),
        Calc(calc) => calc.run(),
        Info(info) => info.run(),
    };
    if let Err(why) = result {
        eprintln!("Error: {why}");
        process::exit(1);
    }
}

fn default_path() -> PathBuf {
    PathBuf::from("./lies_of_ergo")
}

fn yay_nay_prompt(prompt: &str) -> io::Result<bool> {
    print!("{prompt} [Y/n] ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(!buf[..1].eq_ignore_ascii_case("n"))
}
