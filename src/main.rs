use clap::{CommandFactory, Parser};
use std::fs;
use std::io::{self, Read};
use std::process;
use std::thread;
use std::time::Duration;

use dorkforge::cli::{
    Cli, CliDiffFormat, CliOutputFormat, CliTxtMode, Commands, CoverageArgs, DiffArgs,
    GenerateArgs, GraphArgs, LintTemplatesArgs,
};
use dorkforge::combination::{
    is_combinations_json, parse_cli_combine_arg, parse_combinations_json, CombinationSpec,
};
use dorkforge::coverage::{
    format_coverage_json, format_coverage_txt, CoverageFilter, CoverageMatrix, SourceRegistry,
};
use dorkforge::diff::{diff_results, format_diff_txt, read_generation_result};
use dorkforge::error::{DorkForgeError, Result};
use dorkforge::generator::{DedupPolicy, DorkGenerator, GenerationResult, GeneratorOptions};
use dorkforge::identifiers::{IdentifierType, ParsedIdentifier};
use dorkforge::input::InputParser;
use dorkforge::output::{
    format_graph_dot, format_html, format_json, format_txt, format_urls, OutputFormat,
    OutputWriter, TxtMode,
};
use dorkforge::providers::Provider;
use dorkforge::quality::{lint_templates, LintSeverity};
use dorkforge::templates::TemplateRegistry;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Types => handle_types(),
        Commands::Providers => handle_providers(),
        Commands::Categories => handle_categories(),
        Commands::Coverage(args) => handle_coverage(args),
        Commands::LintTemplates(args) => handle_lint_templates(args),
        Commands::Generate(args) => handle_generate(*args),
        Commands::Completions { shell } => handle_completions(shell),
        Commands::Graph(args) => handle_graph(args),
        Commands::Diff(args) => handle_diff(args),
    }
}

fn handle_types() -> Result<()> {
    println!(
        "Supported Identifier Types in DorkForge v{}:\n",
        env!("CARGO_PKG_VERSION")
    );
    for id_type in IdentifierType::ALL {
        println!("  - {:<15} {}", id_type.as_str(), id_type.description());
        println!(
            "    Applicable categories: {}",
            id_type.applicable_categories().join(", ")
        );
        println!();
    }
    Ok(())
}

fn handle_providers() -> Result<()> {
    println!(
        "Supported Search Engine Providers in DorkForge v{}:\n",
        env!("CARGO_PKG_VERSION")
    );
    for p in Provider::ALL {
        println!("  - {:<12} ({})", p.id(), p.display_name());
        println!("    Operators:  {}", p.supported_operators().join(" "));
        println!("    Categories: {}", p.supported_categories().join(", "));
        println!("    Search URL: {}", p.search_base_url());
        println!();
    }
    Ok(())
}

fn handle_categories() -> Result<()> {
    let registry = TemplateRegistry::load_embedded_defaults()?;
    let cats = registry.all_categories();

    println!("Available OSINT Query Categories (from loaded templates):\n");
    for cat in &cats {
        println!("  - {cat}");
    }
    println!("\nCategory aliases accepted on the command line:");
    println!("  public_documents -> documents");
    println!("  profiles   -> public_profiles");
    println!("  code       -> code_repositories");
    println!("  historical_pages -> archives");
    println!("  company    -> company_records");
    println!("  security   -> security_reports");
    println!("  tech       -> technology_fingerprint");
    println!();
    Ok(())
}

fn handle_coverage(args: CoverageArgs) -> Result<()> {
    let registry = if let Some(ref config_dir) = args.config_dir {
        let mut reg = TemplateRegistry::new();
        reg.load_from_dir(config_dir)?;
        reg
    } else {
        TemplateRegistry::load_embedded_defaults()?
    };

    let sources = if let Some(ref path) = args.sources_file {
        let mut reg = SourceRegistry::new();
        reg.load_from_file(path)?;
        reg
    } else {
        SourceRegistry::load_embedded_defaults()?
    };

    let filter = CoverageFilter {
        provider: args.provider,
        category: args.category,
        r#type: args.r#type,
    };

    let matrix = CoverageMatrix::build(&registry, &sources, filter);

    match args.format {
        CliDiffFormat::Json => {
            println!("{}", format_coverage_json(&matrix, true)?);
        }
        CliDiffFormat::Txt => {
            print!("{}", format_coverage_txt(&matrix));
        }
    }

    Ok(())
}

fn handle_lint_templates(args: LintTemplatesArgs) -> Result<()> {
    let registry = if let Some(ref config_dir) = args.config_dir {
        let mut reg = TemplateRegistry::new();
        reg.load_from_dir(config_dir)?;
        reg
    } else {
        TemplateRegistry::load_embedded_defaults()?
    };

    let report = lint_templates(registry.templates(), args.strict);

    match args.format {
        CliDiffFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| {
                    DorkForgeError::SerializationError(format!(
                        "Failed to serialize lint report: {e}"
                    ))
                })?
            );
        }
        CliDiffFormat::Txt => {
            println!(
                "================================================================================"
            );
            println!(
                "                     DORKFORGE TEMPLATE QUALITY LINT REPORT                     "
            );
            println!("================================================================================\n");
            println!(
                "Total templates evaluated: {} | Passed: {} | Issues found: {}\n",
                report.total_templates,
                report.passed_templates,
                report.issues.len()
            );

            if report.issues.is_empty() {
                println!("All templates passed validation quality checks successfully.");
            } else {
                for issue in &report.issues {
                    let field_str = issue
                        .field
                        .as_deref()
                        .map(|f| format!(" [{f}]"))
                        .unwrap_or_default();
                    let prefix = match issue.severity {
                        LintSeverity::Error => "[ERROR]",
                        LintSeverity::Warning => "[WARN] ",
                    };
                    println!(
                        "  {} {}{}: {}",
                        prefix, issue.template_id, field_str, issue.message
                    );
                }
                println!();
                println!(
                    "Summary: {} error(s), {} warning(s). Strict mode: {}.",
                    report.error_count(),
                    report.warning_count(),
                    args.strict
                );
            }
        }
    }

    if !report.is_success() {
        return Err(DorkForgeError::ValidationError(format!(
            "Catalog lint failed with {} error(s) and {} warning(s) in strict mode",
            report.error_count(),
            report.warning_count()
        )));
    }

    Ok(())
}

fn handle_completions(shell: clap_complete::Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}

fn handle_generate(args: GenerateArgs) -> Result<()> {
    if !args.watch {
        return handle_generate_once(&args);
    }
    if args.config_dir.is_none() && args.extra_templates.is_none() {
        return Err(DorkForgeError::ValidationError(
            "--watch requires --config-dir or --extra-templates".to_string(),
        ));
    }
    if args.value.as_deref() == Some("-") {
        return Err(DorkForgeError::ValidationError(
            "--watch cannot reread a one-shot stdin value; use --input instead".to_string(),
        ));
    }
    if args.watch_interval == 0 {
        return Err(DorkForgeError::ValidationError(
            "--watch-interval must be at least 1 second".to_string(),
        ));
    }

    let watched_dir = args
        .config_dir
        .as_deref()
        .or(args.extra_templates.as_deref())
        .expect("validated above");
    let mut snapshot = dorkforge::watch::template_snapshot(watched_dir)?;
    let mut first_run = true;
    loop {
        let mut run_args = args.clone();
        if !first_run {
            run_args.overwrite = true;
        }
        match handle_generate_once(&run_args) {
            Ok(()) => {}
            Err(error) if first_run => return Err(error),
            Err(error) => {
                eprintln!(
                    "[Warning] Reload failed; keeping the previous results until templates are fixed: {error}"
                );
            }
        }
        first_run = false;

        if !args.quiet {
            eprintln!(
                "[DorkForge] Watching '{}' for template changes (every {}s). Press Ctrl-C to stop.",
                watched_dir.display(),
                args.watch_interval
            );
        }
        loop {
            thread::sleep(Duration::from_secs(args.watch_interval));
            let next_snapshot = dorkforge::watch::template_snapshot(watched_dir)?;
            if next_snapshot != snapshot {
                snapshot = next_snapshot;
                if !args.quiet {
                    eprintln!("[DorkForge] Template change detected; regenerating.");
                }
                break;
            }
        }
    }
}

fn handle_generate_once(args: &GenerateArgs) -> Result<()> {
    let quiet = args.quiet;

    if args.max_variants == 0 {
        return Err(DorkForgeError::ValidationError(
            "--max-variants must be at least 1. Use 1 to disable multi-variant expansion."
                .to_string(),
        ));
    }

    // 1. Load templates
    let registry = if let Some(ref config_dir) = args.config_dir {
        let mut reg = TemplateRegistry::new();
        let loaded = reg.load_from_dir(config_dir)?;
        if loaded == 0 {
            return Err(DorkForgeError::ConfigError {
                path: config_dir.clone(),
                message: "No valid templates found in configuration directory".to_string(),
            });
        }
        reg
    } else if let Some(ref extra_dir) = args.extra_templates {
        TemplateRegistry::load_embedded_with_extra(extra_dir)?
    } else {
        TemplateRegistry::load_embedded_defaults()?
    };

    // 2. Parse providers
    let providers = match &args.providers {
        Some(list) => {
            let mut resolved = Vec::new();
            for item in list {
                let p = item.parse::<Provider>()?;
                if !resolved.contains(&p) {
                    resolved.push(p);
                }
            }
            if resolved.is_empty() {
                return Err(DorkForgeError::ValidationError(
                    "No providers specified".to_string(),
                ));
            }
            resolved
        }
        None => Provider::ALL.to_vec(),
    };

    // 3. Collect inputs: single identifiers & combinations
    let input_parser = InputParser::new(args.strict);
    let mut parsed_identifiers: Vec<ParsedIdentifier> = Vec::new();
    let mut parsed_combinations: Vec<CombinationSpec> = Vec::new();

    // 3a. Single identifier from --value or stdin
    if let Some(ref val) = args.value {
        let raw = if val == "-" {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| DorkForgeError::IoError {
                    path: std::path::PathBuf::from("<stdin>"),
                    source: e,
                })?;
            buf.trim().to_string()
        } else {
            val.clone()
        };

        if raw.is_empty() {
            return Err(DorkForgeError::EmptyInput);
        }

        let type_hint = match args.r#type {
            Some(ref t) => Some(t.parse::<IdentifierType>()?),
            None => None,
        };

        let parsed = input_parser.parse_single(
            &raw,
            type_hint,
            args.country.as_deref(),
            args.network.as_deref(),
        )?;
        parsed_identifiers.push(parsed);
    }

    // 3b. CLI --combine options
    if !args.combine.is_empty() {
        let mut combo_inputs = Vec::new();
        for item in &args.combine {
            combo_inputs.push(parse_cli_combine_arg(item)?);
        }
        parsed_combinations.push(CombinationSpec {
            id: "cli_combination".to_string(),
            identifiers: combo_inputs,
            categories: args.categories.clone(),
            providers: args.providers.clone(),
        });
    }

    // 3c. Input file (--input)
    if let Some(ref file_path) = args.input {
        let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if extension.eq_ignore_ascii_case("json") {
            let content = fs::read_to_string(file_path).map_err(|e| DorkForgeError::IoError {
                path: file_path.to_path_buf(),
                source: e,
            })?;
            if is_combinations_json(&content) {
                let combos = parse_combinations_json(&content)?;
                parsed_combinations.extend(combos);
            } else {
                let batch = input_parser.parse_json_str(&content)?;
                if !batch.errors.is_empty() {
                    for (item, err) in &batch.errors {
                        if !quiet {
                            eprintln!("[Warning] Skipped input '{item}': {err}");
                        }
                    }
                }
                parsed_identifiers.extend(batch.identifiers);
            }
        } else {
            let batch = input_parser.parse_file(file_path)?;
            if !batch.errors.is_empty() {
                for (item, err) in &batch.errors {
                    if !quiet {
                        eprintln!("[Warning] Skipped input '{item}': {err}");
                    }
                }
            }
            parsed_identifiers.extend(batch.identifiers);
        }
    }

    if parsed_identifiers.is_empty() && parsed_combinations.is_empty() {
        return Err(DorkForgeError::EmptyInput);
    }

    // 4. Validate category compatibility for single identifiers
    if let Some(ref cats) = args.categories {
        for id in &parsed_identifiers {
            let applicable = id.identifier_type().applicable_categories();
            for cat in cats {
                let cat_lower = cat.to_ascii_lowercase();
                let matches = applicable
                    .iter()
                    .any(|a| dorkforge::templates::category_matches(&cat_lower, a));
                if !matches && !quiet {
                    eprintln!(
                        "[Warning] Category '{}' has no templates for identifier type '{}' — will produce 0 results.",
                        cat,
                        id.identifier_type().as_str()
                    );
                }
            }
        }
    }

    // 5. Run generator
    let options = GeneratorOptions {
        providers,
        categories: args.categories.clone(),
        max_variants: args.max_variants,
        max_queries_per_identifier: args.max_queries_per_identifier,
        max_queries_per_provider: args.max_queries_per_provider,
        max_combination_variants: args.max_combination_variants,
        max_variant_combinations: args.max_variant_combinations,
        max_combination_queries: args.max_combination_queries,
        dedup_policy: DedupPolicy::Strict,
        country_hint: args.country.clone(),
        network_hint: args.network.clone(),
        strict: args.strict,
    };

    let generator = DorkGenerator::new(&registry, options);
    let result: GenerationResult =
        generator.generate_all(&parsed_identifiers, &parsed_combinations)?;

    // 6. Resolve output format
    let mut format = match args.format {
        CliOutputFormat::Txt => OutputFormat::Txt,
        CliOutputFormat::Json => OutputFormat::Json,
        CliOutputFormat::Html => OutputFormat::Html,
        CliOutputFormat::Both => OutputFormat::Both,
        CliOutputFormat::All => OutputFormat::All,
    };

    if matches!(format, OutputFormat::Both | OutputFormat::All) && args.output.is_none() {
        if !quiet {
            eprintln!(
                "[Warning] --format {} requires -o/--output to be specified. \
                 Falling back to --format txt. Use '-o <dir>' to save multiple formats.",
                if format == OutputFormat::Both {
                    "both"
                } else {
                    "all"
                }
            );
        }
        format = OutputFormat::Txt;
    }

    if args.urls && format == OutputFormat::Json {
        return Err(DorkForgeError::ValidationError(
            "--urls renders TXT links; use --format txt, both, or all (HTML is always clickable)"
                .to_string(),
        ));
    }

    let txt_mode = match args.mode {
        CliTxtMode::Clean => TxtMode::Clean,
        CliTxtMode::Readable => TxtMode::Readable,
    };

    // 7. Output
    if let Some(ref out_path) = args.output {
        let writer = OutputWriter::new(format, txt_mode, true, args.overwrite).with_urls(args.urls);
        let written = writer.write_to_destination(&result, Some(out_path))?;
        if !quiet {
            for file in written {
                eprintln!("[DorkForge] Saved: {}", file.display());
            }
        }
    } else {
        match format {
            OutputFormat::Txt => {
                if args.urls {
                    print!("{}", format_urls(&result));
                } else {
                    print!("{}", format_txt(&result, txt_mode));
                }
            }
            OutputFormat::Json => {
                println!("{}", format_json(&result, true)?);
            }
            OutputFormat::Html => {
                println!("{}", format_html(&result));
            }
            OutputFormat::Both | OutputFormat::All => unreachable!(),
        }
    }

    Ok(())
}

fn handle_graph(args: GraphArgs) -> Result<()> {
    let result = read_generation_result(&args.input)?;
    let dot = format_graph_dot(&result);
    if let Some(ref out_path) = args.output {
        if out_path.exists() && !args.overwrite {
            return Err(DorkForgeError::OutputConflict {
                path: out_path.clone(),
            });
        }
        if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| DorkForgeError::IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(out_path, dot).map_err(|e| DorkForgeError::IoError {
            path: out_path.clone(),
            source: e,
        })?;
    } else {
        print!("{dot}");
    }
    Ok(())
}

fn handle_diff(args: DiffArgs) -> Result<()> {
    let before = read_generation_result(&args.before)?;
    let after = read_generation_result(&args.after)?;
    let diff = diff_results(&before, &after);

    let rendered = match args.format {
        CliDiffFormat::Txt => format_diff_txt(&diff),
        CliDiffFormat::Json => serde_json::to_string_pretty(&diff).map_err(|e| {
            DorkForgeError::SerializationError(format!("Failed to serialize diff: {e}"))
        })?,
    };

    if let Some(ref out_path) = args.output {
        if out_path.exists() && !args.overwrite {
            return Err(DorkForgeError::OutputConflict {
                path: out_path.clone(),
            });
        }
        if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|e| DorkForgeError::IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(out_path, rendered).map_err(|e| DorkForgeError::IoError {
            path: out_path.clone(),
            source: e,
        })?;
    } else {
        println!("{rendered}");
    }

    Ok(())
}
