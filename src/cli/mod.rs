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

    /// Number of worker threads (default: available parallelism).
    #[arg(short = 'j', long, global = true, value_name = "N")]
    jobs: Option<usize>,

    /// Ignore cached `.algo` artifacts and recompile.
    #[arg(long, global = true)]
    force: bool,

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
    /// Compile the targets to `.algo` bytecode.
    Compile { targets: Vec<PathBuf> },
    /// Compile if needed, then run the parallel proof checker over the bytecode.
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
    pub jobs: Option<usize>,
    pub force: bool,
    pub quiet: bool,
    pub verbose: bool,
}

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let opts = GlobalOpts {
        stdlib: cli.stdlib,
        project: cli.project,
        jobs: cli.jobs,
        force: cli.force,
        quiet: cli.quiet,
        verbose: cli.verbose,
    };

    let result = match cli.command {
        Command::Parse { targets, dump_ast } => cmd_parse(&opts, &targets, dump_ast),
        Command::Typecheck { targets } => cmd_typecheck(&opts, &targets),
        Command::Compile { targets } => cmd_compile(&opts, &targets),
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
        match crate::parse::parse(&src) {
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

fn jobs(opts: &GlobalOpts) -> usize {
    opts.jobs.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    })
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
        match crate::elaborate::proof::elaborate_unit(&src, &module, &resolver, false) {
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

/// The `.algo` artifact path for a source file.
fn algo_path(src: &PathBuf) -> PathBuf {
    src.with_extension("algo")
}

/// Hash a dependency's current source (returns None if unresolvable).
fn dep_current_hash(opts: &GlobalOpts, file: &PathBuf, module: &str) -> Option<u128> {
    use crate::elaborate::proof::SourceResolver;
    let resolver = resolver_for(opts, file);
    resolver
        .resolve(module)
        .ok()
        .map(|s| crate::bytecode::hash128(s.as_bytes()))
}

/// Whether a fresh `.algo` exists for `file` (own + dependency hashes match).
fn load_fresh_algo(opts: &GlobalOpts, file: &PathBuf, src: &str) -> Option<crate::bytecode::AlgoFile> {
    if opts.force {
        return None;
    }
    let bytes = std::fs::read(algo_path(file)).ok()?;
    let algo = crate::bytecode::decode(&bytes).ok()?;
    if algo.source_hash != crate::bytecode::hash128(src.as_bytes()) {
        return None;
    }
    for (module, stored) in &algo.deps {
        match dep_current_hash(opts, file, module) {
            Some(h) if h == *stored => {}
            _ => return None,
        }
    }
    Some(algo)
}

/// Atomically write `bytes` to `path` (temp file + rename).
fn write_atomic(path: &PathBuf, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("algo.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

/// Outcome of compiling one file: a status line (stdout) and rendered errors.
struct CompileOutcome {
    status: Option<String>,
    errors: Vec<String>,
}

/// Compile one file (owned inputs so it can run on a worker thread).
fn compile_file(opts: &GlobalOpts, file: &PathBuf) -> CompileOutcome {
    let src = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            return CompileOutcome {
                status: None,
                errors: vec![format!("error: reading {}: {e}", file.display())],
            }
        }
    };
    if !opts.force && load_fresh_algo(opts, file, &src).is_some() {
        return CompileOutcome {
            status: Some(format!("{}: up to date", file.display())),
            errors: Vec::new(),
        };
    }
    let module = module_name_of(file);
    let resolver = resolver_for(opts, file);
    match crate::elaborate::proof::elaborate_unit(&src, &module, &resolver, true) {
        Ok(unit) => {
            let source_hash = crate::bytecode::hash128(src.as_bytes());
            let bytes = crate::bytecode::encode(&unit, source_hash, &unit.deps);
            let mut errors = Vec::new();
            if let Err(e) = write_atomic(&algo_path(file), &bytes) {
                errors.push(format!("error: writing {}: {e}", algo_path(file).display()));
            }
            CompileOutcome {
                status: Some(format!("compiled {} -> {}", file.display(), algo_path(file).display())),
                errors,
            }
        }
        Err(diags) => CompileOutcome {
            status: None,
            errors: diags
                .iter()
                .map(|d| d.clone().with_file(file.clone()).render(Some(&src)))
                .collect(),
        },
    }
}

fn cmd_compile(opts: &GlobalOpts, targets: &[PathBuf]) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    // Files are independent units, so compile them in parallel. Results are
    // collected by input index and reported in deterministic order.
    let outcomes = parallel_map(files.clone(), jobs(opts), |f| compile_file(opts, f));
    let mut ok = true;
    for outcome in outcomes {
        if !outcome.errors.is_empty() {
            ok = false;
            for e in &outcome.errors {
                eprintln!("{e}");
            }
        }
        if let Some(s) = outcome.status {
            if !opts.quiet {
                println!("{s}");
            }
        }
    }
    if ok {
        Ok(())
    } else {
        Err(())
    }
}

/// Run `f` over `items` across `n_jobs` scoped worker threads, returning results
/// in input order (deterministic regardless of scheduling).
fn parallel_map<T, R>(items: Vec<T>, n_jobs: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
{
    let n = items.len();
    if n_jobs <= 1 || n <= 1 {
        return items.iter().map(|t| f(t)).collect();
    }
    let workers = n_jobs.min(n);
    let results: std::sync::Mutex<Vec<(usize, R)>> = std::sync::Mutex::new(Vec::with_capacity(n));
    std::thread::scope(|scope| {
        for w in 0..workers {
            let items = &items;
            let f = &f;
            let results = &results;
            scope.spawn(move || {
                let mut local = Vec::new();
                let mut i = w;
                while i < n {
                    local.push((i, f(&items[i])));
                    i += workers;
                }
                results.lock().unwrap().extend(local);
            });
        }
    });
    let mut v = results.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, r)| r).collect()
}

fn cmd_verify(opts: &GlobalOpts, targets: &[PathBuf]) -> Result<(), ()> {
    let files = collect_alg_files(targets)?;
    if files.is_empty() {
        eprintln!("error: no .alg files given");
        return Err(());
    }
    let mut ok = true;
    let n_jobs = jobs(opts);
    for f in &files {
        let src = read_source(f)?;
        // Use a fresh `.algo` if available (the fast second-run path),
        // otherwise elaborate and write one.
        let (obligations, rewrite, cached) = if let Some(algo) = load_fresh_algo(opts, f, &src) {
            (algo.obligations, algo.rewrite, true)
        } else {
            let module = module_name_of(f);
            let resolver = resolver_for(opts, f);
            match crate::elaborate::proof::elaborate_unit(&src, &module, &resolver, true) {
                Ok(unit) => {
                    let source_hash = crate::bytecode::hash128(src.as_bytes());
                    let bytes = crate::bytecode::encode(&unit, source_hash, &unit.deps);
                    let _ = write_atomic(&algo_path(f), &bytes);
                    (unit.obligations, unit.rewrite, false)
                }
                Err(diags) => {
                    ok = false;
                    for d in &diags {
                        eprintln!("{}", d.clone().with_file(f.clone()).render(Some(&src)));
                    }
                    continue;
                }
            }
        };
        let mut file_ok = true;
        let mut wip_count = 0usize;
        for ob in &obligations {
            // Admitted (`by wip`) cases are skipped by the checker; the sound
            // parts are still checked.
            let errors = crate::core::check::check(&ob.root, &ob.label, n_jobs, &rewrite);
            if !errors.is_empty() {
                file_ok = false;
                ok = false;
                for e in errors {
                    eprintln!("{}: {e}", f.display());
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
            let tag = if cached { " (cached)" } else { "" };
            let wip = if wip_count > 0 {
                format!(", {wip_count} wip")
            } else {
                String::new()
            };
            println!(
                "{}: checked {} proof obligation(s){wip}{tag}",
                f.display(),
                obligations.len()
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
        let formatted = match crate::fmt::format_source(&src, ascii) {
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
