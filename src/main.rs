use std::fs::File;
use std::io::BufWriter;

use std::path::PathBuf;

use argh::FromArgs;
use ddelta::{apply_chunked, generate_chunked, State};

use indicatif::{ProgressBar, ProgressStyle};

fn parse_size(mut text: &str) -> Result<usize, String> {
    text = text.trim();
    let (num, suffix) = text.split_at(
        text.find(|c: char| !c.is_digit(10) && c != '.')
            .unwrap_or(text.len()),
    );
    if num.trim().is_empty() && suffix.trim().is_empty() {
        return Ok(0);
    }
    let amt: f64 = num
        .parse()
        .map_err(|_| format!("Could not parse {} as a number", num))?;
    match suffix.trim() {
        "" | "B" => Ok(amt as usize),
        "K" | "KB" => Ok((amt * 1_000.) as usize),
        "M" | "MB" => Ok((amt * 1_000_000.) as usize),
        "G" | "GB" => Ok((amt * 1_000_000_000.) as usize),
        "T" | "TB" => Ok((amt * 1_000_000_000_000.) as usize),
        suffix => Err(format!("Suffix {} not understood", suffix)),
    }
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommand {
    Diff(DiffCmd),
    Patch(PatchCmd),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Generate and apply efficient binary patches to files.
struct Arguments {
    #[argh(subcommand)]
    nested: SubCommand,
}

/// Generate a patchfile from the difference between files.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "diff")]
struct DiffCmd {
    /// the old/original file
    #[argh(positional)]
    old: PathBuf,
    /// the new file
    #[argh(positional)]
    new: PathBuf,
    /// the patch file to generate
    #[argh(positional)]
    patch: PathBuf,
    /// an optional RAM limit. Defaults to no limit
    #[argh(option, short = 'r', from_str_fn(parse_size), default = "0")]
    ram_limit: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Apply a patchfile generated with 'diff'.
#[argh(subcommand, name = "patch")]
struct PatchCmd {
    /// the old/original file
    #[argh(positional)]
    old: PathBuf,
    /// the new file to write
    #[argh(positional)]
    new: PathBuf,
    /// the patch file
    #[argh(positional)]
    patch: PathBuf,
}

fn main() {
    let cmd: Arguments = argh::from_env();
    match cmd.nested {
        SubCommand::Diff(diff) => {
            let mut old = File::open(diff.old).unwrap();
            let mut new = File::open(diff.new).unwrap();
            let patch = File::create(diff.patch).unwrap();
            let chunk_sizes = match diff.ram_limit / 6 {
                0..=2 => None,
                0..=1024 => {
                    eprintln!(
                        "Warning: changing default RAM limit to {} as {} is too small",
                        1024 * 6,
                        diff.ram_limit
                    );
                    Some(1024)
                }
                other => Some(other),
            };
            let len = new.metadata().unwrap().len();
            let pb = ProgressBar::new(len);
            pb.set_style(ProgressStyle::default_bar().template("{spinner:.green} {msg}[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"));

            pb.set_message("Reading… ");
            generate_chunked(
                &mut old,
                &mut new,
                &mut BufWriter::new(patch),
                chunk_sizes,
                |v| match v {
                    State::Reading => {
                        pb.set_message("Reading… ");
                    }
                    State::Sorting => {
                        pb.set_message("Sorting… ");
                    }
                    State::Working(b) => {
                        pb.set_message("");
                        pb.set_position(b);
                    }
                },
            )
            .unwrap();
            pb.set_message("");
            pb.finish();
        }
        SubCommand::Patch(patch) => {
            let mut old = File::open(patch.old).unwrap();
            let mut new = File::create(patch.new).unwrap();
            let mut patch = File::open(patch.patch).unwrap();
            apply_chunked(&mut old, &mut new, &mut patch).unwrap();
        }
    }
}
