use anyhow::bail;
use std::{env, process};

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

fn main() {
    if let Err(why) = _main() {
        eprintln!("Error: {why}");
        process::exit(1);
    }
}

fn _main() -> anyhow::Result<()> {
    let (have, want) = process_args()?;
    if have >= want {
        bail!("you already have all the runes you need");
    }
    println!("You have {have} runes, and you want {want} runes, right?");
    let mut need = want - have;
    let mut counts = [0u32; 20];
    while need > 0 {
        // TODO: store index of last find and use that to slice RUNE_VALUES before searching
        let (index, val) = RUNE_VALUES
            .iter()
            .enumerate()
            .rfind(|(_, val)| **val < need)
            .unwrap_or((0, &200));
        counts[index] += 1;
        need = need.saturating_sub(*val);
    }
    println!("{}", format_output(&counts));
    Ok(())
}

fn process_args() -> anyhow::Result<(u32, u32)> {
    let args = env::args().skip(1).take(4).collect::<Vec<_>>();
    if args.len() != 4 {
        bail!("bad args\nUsage: use_runes have [number] want [number]");
    }
    let mut have = None;
    let mut want = None;
    for chunk in args.chunks_exact(2) {
        match chunk {
            [opcode, operand] => match opcode.as_str() {
                "have" => have = operand.parse().ok(),
                "want" => want = operand.parse().ok(),
                _ => {}
            },
            _ => unreachable!("chunks_exact was not exact"),
        }
    }
    match (have, want) {
        (Some(have), Some(want)) => Ok((have, want)),
        _ => {
            bail!("didn't specify haves and wants\nUsage: use_runes have [number] want [number]");
        }
    }
}

fn format_output(counts: &[u32; 20]) -> String {
    let mut buf = String::from("You will need:");
    counts
        .iter()
        .enumerate()
        .rev() // Give runes biggest to smallest
        .filter(|(_, count)| **count > 0)
        .for_each(|(index, count)| {
            buf.push_str("\n - ");
            buf.push_str(&count.to_string());
            buf.push_str("x ");
            buf.push_str(RUNE_NAMES[index]);
        });
    buf
}
