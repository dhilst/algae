//! Command-line interface (clap).

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "algae", version, about = "Algae v2 proof & specification language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to a vendored standard library directory (overrides the default).
    #[arg(long, global = true, value_name = "DIR")]
    stdlib: Option<PathBuf>,

    /// Path to algae.json or its directory (default: search upward from cwd).
    #[arg(short = 'p', long, global = true, value_name = "PATH")]
    project: Option<PathBuf>,

    /// Reduce output.
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Increase output.
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Tokenize and parse the targets, reporting syntax errors.
    Parse {
        targets: Vec<PathBuf>,
        /// Print the parsed AST.
        #[arg(long)]
        dump_ast: bool,
    },
    /// Parse and elaborate (kind/type/proof checking), reporting static errors.
    Typecheck { targets: Vec<PathBuf> },
    /// Elaborate, then run the proof checker over every obligation.
    Verify { targets: Vec<PathBuf> },
    /// Normalize operator glyphs (ASCII→Unicode by default), preserving whitespace.
    Fmt {
        targets: Vec<PathBuf>,
        /// Convert to ASCII forms instead of Unicode.
        #[arg(long)]
        ascii: bool,
        /// Print to stdout instead of editing in place.
        #[arg(long)]
        stdout: bool,
        /// Exit non-zero if reformatting would change a file; write nothing.
        #[arg(long)]
        check: bool,
    },
}

/// Options shared across subcommands, derived from global flags.
#[derive(Clone, Debug)]
pub struct GlobalOpts {
    pub stdlib: Option<PathBuf>,
    pub project: Option<PathBuf>,
    pub quiet: bool,
    pub verbose: bool,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let opts = GlobalOpts {
        stdlib: cli.stdlib,
        project: cli.project,
        quiet: cli.quiet,
        verbose: cli.verbose,
    };

    let result = match cli.command {
        Command::Parse { targets, dump_ast } => cmd_parse(&opts, &targets, dump_ast),
        Command::Typecheck { targets } => cmd_typecheck(&opts, &targets),
        Command::Verify { targets } => cmd_verify(&opts, &targets),
        Command::Fmt {
            targets,
            ascii,
            stdout,
            check,
        } => cmd_fmt(&opts, &targets, ascii, stdout, check),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

/// Collect `.alg` files from the given targets (files or directories,
/// recursively), in deterministic sorted order.
fn collect_alg_files(targets: &[PathBuf]) -> Result<Vec<PathBuf>, ()> {
    let mut files = Vec::new();
    for t in targets {
        if t.is_dir() {
            collect_dir(t, &mut files)?;
        } else if t.is_file() {
            files.push(t.clone());
        } else {
            eprintln!("error: no such file or directory: {}", t.display());
            return Err(());
        }
    }
    Ok(files)
}

fn collect_dir(dir: &PathBuf, out: &mut Vec<PathBuf>) -> Result<(), ()> {
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(e) => {
            eprintln!("error: reading {}: {e}", dir.display());
            return Err(());
        }
    };
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect_dir(&p, out)?;
        } else if p.extension().map(|e| e == "alg").unwrap_or(false) {
            out.push(p);
        }
    }
    Ok(())
}

fn read_source(path: &PathBuf) -> Result<String, ()> {
    std::fs::read_to_string(path).map_err(|e| {
        eprintln!("error: reading {}: {e}", path.display());
    })
}

fn cmd_parse(opts: &GlobalOpts, targets: &[PathBuf], dump_ast: bool) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    let mut ok = true;
    for f in &files {
        let src = read_source(f)?;
        match algae_kernel::parse::parse(&src) {
            Ok(m) => {
                if dump_ast {
                    println!("// {}", f.display());
                    println!("{m:#?}");
                } else if !opts.quiet {
                    println!("{}: ok ({} declarations)", f.display(), m.decls.len());
                }
            }
            Err(diags) => {
                ok = false;
                for d in &diags {
                    eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                }
            }
        }
    }
    if ok {
        Ok(())
    } else {
        Err(())
    }
}

fn module_name_of(path: &PathBuf) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main")
        .to_string()
}

fn resolver_for(opts: &GlobalOpts, file: &PathBuf) -> crate::project::DirResolver {
    let mut roots = Vec::new();
    if let Some(parent) = file.parent() {
        roots.push(parent.to_path_buf());
    }
    // Use an `algae.json` manifest if one is found (via --project or by walking
    // up from the file); otherwise fall back to the file's dir + stdlib.
    let manifest_start = opts.project.clone().unwrap_or_else(|| file.clone());
    if let Some((dir, manifest)) = crate::project::Manifest::find(&manifest_start) {
        roots.extend(manifest.roots(&dir, opts.stdlib.clone()));
    } else {
        roots.push(
            opts.stdlib
                .clone()
                .unwrap_or_else(crate::project::default_stdlib),
        );
    }
    crate::project::DirResolver::new(roots)
}

fn cmd_typecheck(opts: &GlobalOpts, targets: &[PathBuf]) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    let mut ok = true;
    for f in &files {
        let src = read_source(f)?;
        let module = module_name_of(f);
        let resolver = resolver_for(opts, f);
        match algae_kernel::elaborate::proof::elaborate_unit(&src, &module, &resolver, false) {
            Ok(_) => {
                if !opts.quiet {
                    println!("{}: ok", f.display());
                }
            }
            Err(diags) => {
                ok = false;
                for d in &diags {
                    eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                }
            }
        }
    }
    if ok {
        Ok(())
    } else {
        Err(())
    }
}

fn cmd_verify(opts: &GlobalOpts, targets: &[PathBuf]) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    let mut ok = true;
    for f in &files {
        let src = read_source(f)?;
        let module = module_name_of(f);
        let resolver = resolver_for(opts, f);
        let unit = match algae_kernel::elaborate::proof::elaborate_unit(&src, &module, &resolver, true)
        {
            Ok(unit) => unit,
            Err(diags) => {
                ok = false;
                for d in &diags {
                    eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                }
                continue;
            }
        };
        // Warnings are non-fatal: report them, but don't fail the run.
        for d in &unit.warnings {
            eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
        }
        let mut file_ok = true;
        let mut wip_count = 0usize;
        for ob in &unit.obligations {
            // Admitted (`by wip`) cases are skipped by the checker; the sound
            // parts are still checked.
            let errors = algae_kernel::core::check::check(&ob.root, &ob.label, &unit.rewrite);
            if !errors.is_empty() {
                file_ok = false;
                ok = false;
                for d in &errors {
                    eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                }
            }
            if ob.wip {
                wip_count += 1;
                // wip means an incomplete proof: report it and fail the run.
                ok = false;
                eprintln!("{}: {} is in progress (wip)", f.display(), ob.label);
            }
        }
        if file_ok && !opts.quiet {
            let wip = if wip_count > 0 {
                format!(", {wip_count} wip")
            } else {
                String::new()
            };
            println!(
                "{}: checked {} proof obligation(s){wip}",
                f.display(),
                unit.obligations.len()
            );
        }
    }
    if ok {
        Ok(())
    } else {
        Err(())
    }
}

fn cmd_fmt(
    opts: &GlobalOpts,
    targets: &[PathBuf],
    ascii: bool,
    stdout: bool,
    check: bool,
) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    let mut ok = true;
    let mut needs_format = false;
    for f in &files {
        let src = read_source(f)?;
        let formatted = match algae_kernel::fmt::format_source(&src, ascii) {
            Ok(s) => s,
            Err(diags) => {
                ok = false;
                for d in &diags {
                    eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                }
                continue;
            }
        };
        if check {
            if formatted != src {
                needs_format = true;
                if !opts.quiet {
                    eprintln!("{}: needs formatting", f.display());
                }
            }
        } else if stdout {
            print!("{formatted}");
        } else if formatted != src {
            if let Err(e) = std::fs::write(f, &formatted) {
                eprintln!("error: writing {}: {e}", f.display());
                ok = false;
            } else if !opts.quiet {
                println!("formatted {}", f.display());
            }
        }
    }
    if ok && !(check && needs_format) {
        Ok(())
    } else {
        Err(())
    }
}
