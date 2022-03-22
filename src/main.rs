use anyhow::{anyhow, bail, Context};
use argh::FromArgs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::process;

const RUNE_NAMES: [&str; 20] = [
    "Golden Rune [1]",
    "Golden Rune [2]",
    "Golden Rune [3]",
    "Golden Rune [4]",
    "Golden Rune [5]",
    "Golden Rune [6]",
    "Golden Rune [7]",
    "Golden Rune [8]",
    "Golden Rune [9]",
    "Golden Rune [10]",
    "Golden Rune [11]",
    "Golden Rune [12]",
    "Golden Rune [13]",
    "Numen's Rune",
    "Hero's Rune [1]",
    "Hero's Rune [2]",
    "Hero's Rune [3]",
    "Hero's Rune [4]",
    "Hero's Rune [5]",
    "Lord's Rune",
];
const RUNE_VALUES: [u32; 20] = [
    200, 400, 800, 1200, 1600, 2000, 2500, 3000, 3800, 5000, 6250, 7500, 10000,
    12500, 15000, 20000, 25000, 30000, 35000, 50000,
];

#[derive(FromArgs)]
/// Tells you the optimal rune items to use to reach your desired amount in Elden Ring
struct WhatDo {
    #[argh(subcommand)]
    subcommand: DoThis,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum DoThis {
    Init(Initialise),
    Calc(Calculation),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "calc")]
/// Perform a rune calculation
struct Calculation {
    /// how many runes you have
    #[argh(option, short = 'h', default = "0")]
    have: u32,
    /// how many runes you want to have
    #[argh(option, short = 'w')]
    want: u32,
    /// where to get the runes file from (defaults to ./elden_runes)
    #[argh(option, default = "default_path()")]
    file: PathBuf,
}

impl Calculation {
    fn run(&self) -> anyhow::Result<()> {
        if self.have >= self.want {
            bail!("you already have all the runes you need");
        }
        println!(
            "You have {} runes, and you want {} runes, right?",
            self.have, self.want
        );
        let outcome = match File::open(&self.file) {
            Ok(mut handle) => {
                let inventory = RuneCount::load(&mut handle)?;
                self.with_inventory(inventory)?
            }
            Err(_) => self.without_inventory(),
        };
        println!("You need to use:\n{}", outcome.format_as_list());
        Ok(())
    }

    fn with_inventory(
        &self,
        inventory: RuneCount,
    ) -> anyhow::Result<RuneCount> {
        let mut need = self.want - self.have;
        let inv_total = inventory.total();
        if inv_total < need {
            let short = need - inv_total;
            bail!("you don't have enough rune items to reach your target, you'll be {short} rune(s) short");
        }
        let mut counts = RuneCount([0u32; 20]);
        let mut last_index = 19;
        while need > 0 {
            // TODO: use multiple of one size at once if helpful
            let (index, val) = RUNE_VALUES[..=last_index]
                .iter()
                .enumerate()
                .filter(|(index, _)| inventory.has(*index))
                .rfind(|(_, val)| **val <= need)
                .unwrap_or((0, &200));
            last_index = index;
            counts[index] += 1;
            need = need.saturating_sub(*val);
        }
        // TODO: update inventory if loaded from file
        Ok(counts)
    }

    fn without_inventory(&self) -> RuneCount {
        let mut need = self.want - self.have;
        let mut counts = RuneCount::default();
        let mut last_index = 19;
        while need > 0 {
            let (index, val) = RUNE_VALUES[..=last_index]
                .iter()
                .enumerate()
                .rfind(|(_, val)| **val <= need)
                .unwrap_or((0, &200));
            last_index = index;
            counts[index] += 1;
            need = need.saturating_sub(*val);
        }
        counts
    }
}

impl Default for Calculation {
    fn default() -> Self {
        Calculation {
            have: 0,
            want: 0,
            file: default_path(),
        }
    }
}

#[derive(FromArgs)]
#[argh(subcommand, name = "init")]
/// Initialise a new elden_runes file
struct Initialise {
    #[argh(positional, default = "default_path()")]
    path: PathBuf,
}

impl Initialise {
    const TEMPLATE: &'static str = include_str!("../elden_runes_template");

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

#[derive(Copy, Clone, Default)]
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
struct RuneCount([u32; 20]);

impl RuneCount {
    fn has(&self, index: usize) -> bool {
        self[index] > 0
    }

    fn total(&self) -> u32 {
        self.into_iter()
            .zip(RUNE_VALUES)
            .fold(0, |acc, (count, val)| {
                acc.saturating_add(count.saturating_mul(val))
            })
    }

    fn format_as_list(&self) -> String {
        let mut buf = String::new();
        self.into_iter()
            .enumerate()
            .rev() // Give runes biggest to smallest
            .filter(|(_, count)| *count > 0)
            .for_each(|(index, count)| {
                buf.push_str("- ");
                buf.push_str(&count.to_string());
                buf.push_str("x ");
                buf.push_str(RUNE_NAMES[index]);
                buf.push('\n');
            });
        buf.pop(); // Removes final newline
        buf
    }

    fn load(read_handle: &mut File) -> anyhow::Result<Self> {
        let mut contents = String::new();
        read_handle.read_to_string(&mut contents)?;
        let mut counts = RuneCount([0; 20]);
        contents.lines()
            .enumerate()
            .try_for_each(|(line_no, line)| -> anyhow::Result<()> {
                let (count, name) = line
                    .split_once("x ")
                    .context(anyhow!("line {line_no}: missing delimiter between quantity and rune name"))?;
                let count = count
                    .parse::<u32>()
                    .context(anyhow!("line {line_no}: bad rune quantity"))?;
                let (index, _) = RUNE_NAMES
                    .iter()
                    .enumerate()
                    .find(|(_, rune)| **rune == name)
                    .context(anyhow!("line {line_no}: couldn't match rune name"))?;
                counts[index] += count;
                Ok(())
            })?;
        Ok(counts)
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

fn main() {
    use DoThis::*;
    let what_do: WhatDo = argh::from_env();
    let result = match what_do.subcommand {
        Init(init) => init.run(),
        Calc(calc) => calc.run(),
    };
    if let Err(why) = result {
        eprintln!("Error: {why}");
        process::exit(1);
    }
}

fn default_path() -> PathBuf {
    PathBuf::from("./elden_runes")
}

#[cfg(test)]
mod unit_tests {
    use crate::{Calculation, RuneCount};

    #[test]
    fn simple_calcs() {
        let calc = Calculation {
            want: 200,
            ..Default::default()
        };
        let mut expected = RuneCount::default();
        expected[0] = 1;
        assert_eq!(calc.without_inventory(), expected);

        let calc = Calculation {
            want: 420,
            ..Default::default()
        };
        let mut expected = RuneCount::default();
        expected[0] = 1;
        expected[1] = 1;
        assert_eq!(calc.without_inventory(), expected);

        let calc = Calculation {
            want: 1200,
            ..Default::default()
        };
        let mut expected = RuneCount::default();
        expected[3] = 1;
        assert_eq!(calc.without_inventory(), expected);

        let calc = Calculation {
            have: 200,
            want: 2200,
            ..Default::default()
        };
        let mut expected = RuneCount::default();
        expected[5] = 1;
        assert_eq!(calc.without_inventory(), expected);
    }
}
