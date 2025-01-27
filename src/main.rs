// Copyright (C) 2024 SUSE LLC <petr.pavlu@suse.com>
// SPDX-License-Identifier: GPL-2.0-or-later

use ksymtypes::sym::SymCorpus;
use ksymtypes::{debug, init_debug_level};
use std::path::Path;
use std::time::Instant;
use std::{env, process};

/// A type to measure elapsed time for some operation.
///
/// The time is measured between when the object is instantiated and when it is dropped. A message
/// with the elapsed time is output when the object is dropped.
enum Timing {
    Active { desc: String, start: Instant },
    Inactive,
}

impl Timing {
    fn new(do_timing: bool, desc: &str) -> Self {
        if do_timing {
            Timing::Active {
                desc: desc.to_string(),
                start: Instant::now(),
            }
        } else {
            Timing::Inactive
        }
    }
}

impl Drop for Timing {
    fn drop(&mut self) {
        match self {
            Timing::Active { desc, start } => {
                eprintln!("{}: {:.3?}", desc, start.elapsed());
            }
            Timing::Inactive => {}
        }
    }
}

/// Prints the global usage message on `stdout`.
fn print_usage() {
    print!(concat!(
        "Usage: ksymtypes [OPTION...] COMMAND\n",
        "\n",
        "Options:\n",
        "  -d, --debug           enable debug output\n",
        "  -h, --help            display this help and exit\n",
        "  --version             output version information and exit\n",
        "\n",
        "Commands:\n",
        "  consolidate           consolidate symtypes into a single file\n",
        "  compare               show differences between two symtypes corpuses\n",
    ));
}

/// Prints the version information on `stdout`.
fn print_version() {
    println!("ksymtypes {}", env!("CARGO_PKG_VERSION"));
}

/// Prints the usage message for the `consolidate` command on `stdout`.
fn print_consolidate_usage() {
    print!(concat!(
        "Usage: ksymtypes consolidate [OPTION...] [PATH...]\n",
        "Consolidate symtypes into a single file.\n",
        "\n",
        "Options:\n",
        "  -h, --help            print this help\n",
        "  -j, --jobs=NUM        use NUM workers to perform the operation simultaneously\n",
        "  -o, --output=FILE     write the result in a specified file, instead of stdout\n",
    ));
}

/// Prints the usage message for the `compare` command on `stdout`.
fn print_compare_usage() {
    print!(concat!(
        "Usage: ksymtypes compare [OPTION...] PATH1 PATH2\n",
        "Show differences between two symtypes corpuses.\n",
        "\n",
        "Options:\n",
        "  -h, --help            print this help\n",
        "  -j, --jobs=NUM        use NUM workers to perform the operation simultaneously\n",
    ));
}

/// Handles an option with a mandatory value.
///
/// When the `arg` matches the `short` or `long` variant, the function returns [`Ok(Some(String))`]
/// with the option value. Otherwise, [`Ok(None)`] is returned when the `arg` doesn't match, or
/// [`Err`] in case of an error.
fn handle_value_option<I>(
    arg: &str,
    args: &mut I,
    short: &str,
    long: &str,
) -> Result<Option<String>, ()>
where
    I: Iterator<Item = String>,
{
    // Handle '-<short> <value>' and '--<long> <value>'.
    if arg == short || arg == long {
        match args.next() {
            Some(value) => return Ok(Some(value.to_string())),
            None => {
                eprintln!("Missing argument for '{}'", long);
                return Err(());
            }
        };
    }

    // Handle '-<short><value>'.
    if let Some(value) = arg.strip_prefix(short) {
        return Ok(Some(value.to_string()));
    }

    // Handle '--<long>=<value>'.
    if let Some(rem) = arg.strip_prefix(long) {
        if let Some(value) = rem.strip_prefix("=") {
            return Ok(Some(value.to_string()));
        }
    }

    Ok(None)
}

/// Handles the `-j`/`--jobs` option which specifies the number of workers to perform a given
/// operation simultaneously.
fn handle_jobs_option<I>(arg: &str, args: &mut I) -> Result<Option<i32>, ()>
where
    I: Iterator<Item = String>,
{
    if let Some(value) = handle_value_option(arg, args, "-j", "--jobs")? {
        match value.parse::<i32>() {
            Ok(jobs) => {
                if jobs < 1 {
                    eprintln!("Invalid value for '{}': must be positive", arg);
                    return Err(());
                }
                return Ok(Some(jobs));
            }
            Err(err) => {
                eprintln!("Invalid value for '{}': {}", arg, err);
                return Err(());
            }
        };
    }

    Ok(None)
}

/// Collects operands from the rest of `args` and checks that they are not an option, unless
/// `past_dash_dash` is `true`.
fn collect_operands<I>(args: I, past_dash_dash: bool, operands: &mut Vec<String>) -> Result<(), ()>
where
    I: Iterator<Item = String>,
{
    for arg in args {
        // Check it's not an option.
        if !past_dash_dash && arg.starts_with("-") {
            eprintln!("Option '{}' must precede operands", arg);
            return Err(());
        }
        operands.push(arg);
    }
    Ok(())
}

/// Handles the `consolidate` command which consolidates symtypes into a single file.
fn do_consolidate<I>(do_timing: bool, args: I) -> Result<(), ()>
where
    I: IntoIterator<Item = String>,
{
    // Parse specific command options.
    let mut args = args.into_iter();
    let mut output = "-".to_string();
    let mut num_workers = 1;
    let mut past_dash_dash = false;
    let mut maybe_path = None;

    loop {
        let arg = match args.next() {
            Some(arg) => arg,
            None => break,
        };

        if let Some(value) = handle_value_option(&arg, &mut args, "-o", "--output")? {
            output = value;
            continue;
        }
        if let Some(value) = handle_jobs_option(&arg, &mut args)? {
            num_workers = value;
            continue;
        }

        if arg == "-h" || arg == "--help" {
            print_consolidate_usage();
            return Ok(());
        }
        if arg == "--" {
            past_dash_dash = true;
            break;
        }
        if arg.starts_with("-") || arg.starts_with("--") {
            eprintln!("Unrecognized consolidate option '{}'", arg);
            return Err(());
        }
        maybe_path = Some(arg);
        break;
    }

    // Collect all paths on the command line.
    let mut paths = Vec::new();
    if let Some(path) = maybe_path {
        paths.push(path);
    }
    collect_operands(args, past_dash_dash, &mut paths)?;

    if paths.len() == 0 {
        eprintln!("The consolidate source is missing");
        return Err(());
    };

    // Do the consolidation.
    let mut syms = SymCorpus::new();

    {
        let _timing = Timing::new(do_timing, &format!("Reading symtypes from '{:?}'", paths));

        if let Err(err) = syms.load_multiple(&paths, num_workers) {
            if paths.len() == 1 {
                eprintln!(
                    "Failed to read symtypes from '{}': {}",
                    Path::new(&paths[0]).display(),
                    err
                );
            } else {
                eprintln!("Failed to read specified symtypes: {}", err);
            }
            return Err(());
        }
    }

    {
        let _timing = Timing::new(
            do_timing,
            &format!("Writing consolidated symtypes to '{}'", output),
        );

        if let Err(err) = syms.write_consolidated(Path::new(&output)) {
            eprintln!(
                "Failed to write consolidated symtypes to '{}': {}",
                output, err
            );
            return Err(());
        }
    }

    Ok(())
}

/// Handles the `compare` command which shows differences between two symtypes corpuses.
fn do_compare<I>(do_timing: bool, args: I) -> Result<(), ()>
where
    I: IntoIterator<Item = String>,
{
    // Parse specific command options.
    let mut args = args.into_iter();
    let mut num_workers = 1;
    let mut past_dash_dash = false;
    let mut maybe_path = None;

    loop {
        let arg = match args.next() {
            Some(arg) => arg,
            None => break,
        };

        if let Some(value) = handle_jobs_option(&arg, &mut args)? {
            num_workers = value;
            continue;
        }

        if arg == "-h" || arg == "--help" {
            print_compare_usage();
            return Ok(());
        }
        if arg == "--" {
            past_dash_dash = true;
            break;
        }
        if arg.starts_with("-") || arg.starts_with("--") {
            eprintln!("Unrecognized compare option '{}'", arg);
            return Err(());
        }
        maybe_path = Some(arg);
        break;
    }

    // Collect all paths on the command line.
    let mut paths = Vec::new();
    if let Some(path) = maybe_path {
        paths.push(path);
    }
    collect_operands(args, past_dash_dash, &mut paths)?;

    if paths.len() != 2 {
        eprintln!(
            "The compare command takes two sources, '{}' given",
            paths.len()
        );
    }

    // Do the comparison.
    debug!("Compare '{}' and '{}'", paths[0], paths[1]);

    let syms1 = {
        let _timing = Timing::new(do_timing, &format!("Reading symtypes from '{}'", paths[0]));

        let mut syms1 = SymCorpus::new();
        if let Err(err) = syms1.load(Path::new(&paths[0]), num_workers) {
            eprintln!("Failed to read symtypes from '{}': {}", paths[0], err);
            return Err(());
        }
        syms1
    };

    let syms2 = {
        let _timing = Timing::new(do_timing, &format!("Reading symtypes from '{}'", paths[1]));

        let mut syms2 = SymCorpus::new();
        if let Err(err) = syms2.load(Path::new(&paths[1]), num_workers) {
            eprintln!("Failed to read symtypes from '{}': {}", paths[1], err);
            return Err(());
        }
        syms2
    };

    {
        let _timing = Timing::new(do_timing, "Comparison");

        syms1.compare_with(&syms2, num_workers);
    }

    Ok(())
}

fn main() {
    let mut args = env::args();

    // Skip over the program name.
    match args.next() {
        Some(_) => {}
        None => {
            eprintln!("Unknown program name");
            process::exit(1);
        }
    };

    // Handle global options and stop at the command.
    let mut maybe_command = None;
    let mut do_timing = false;
    let mut debug_level = 0;
    loop {
        let arg = match args.next() {
            Some(arg) => arg,
            None => break,
        };

        if arg == "-d" || arg == "--debug" {
            debug_level += 1;
            continue;
        }
        if arg == "--timing" {
            do_timing = true;
            continue;
        }

        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        }
        if arg == "--version" {
            print_version();
            process::exit(0);
        }
        if arg.starts_with("-") || arg.starts_with("--") {
            eprintln!("Unrecognized global option '{}'", arg);
            process::exit(1);
        }
        maybe_command = Some(arg);
        break;
    }

    init_debug_level(debug_level);

    let command = match maybe_command {
        Some(command) => command,
        None => {
            eprintln!("No command specified");
            process::exit(1);
        }
    };

    // Process the specified command.
    match command.as_str() {
        "consolidate" => {
            if let Err(_) = do_consolidate(do_timing, args) {
                process::exit(1);
            }
        }
        "compare" => {
            if let Err(_) = do_compare(do_timing, args) {
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unrecognized command '{}'", command);
            process::exit(1);
        }
    }

    process::exit(0);
}
