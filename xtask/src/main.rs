#![forbid(missing_docs)]
/*!
# xtask: a DevOps tool for pre-organizaed running build scripts locally.
*/

use anyhow::Ok;
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use duct::cmd;
use std::fs::{create_dir_all, remove_dir_all};

// Constants.
const COVERAGE_DIRECTORY: &str = "coverage";
const COVERAGE_LCOV_FILE: &str = "coverage/lcov.info";

/// DevOps Script for local debugging also used in github workflows.
#[derive(Parser, Debug)]
#[command()]
struct Cli {
    /// The kind of action to perform
    #[command(subcommand)]
    command: Commands,

    #[command(flatten)]
    options: Options,
}

#[derive(clap::Args, Debug, Clone)]
struct Options {
    /// Whether to install dependencies lazily.
    ///
    /// This flag is especially helpful for github workflows, in which we don't want to
    /// install more dependencies than we need to for the specific workflow.
    #[clap(short, long)]
    lazy_install: bool,

    /// Min Coverage
    ///
    /// This only makes sense when the commands includes the step `verify-coverage-percent`,
    /// otherwise it's unused.
    #[clap(short, long, default_value = "50")]
    min_coverage: f32,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a workflow comprised of a sequence of steps.
    #[command(arg_required_else_help = true, short_flag = 'w')]
    Workflow { workflow: Workflow },

    /// Run a sequence of steps in the order that they're written.
    #[command(arg_required_else_help = true, short_flag = 's')]
    Steps {
        #[clap(num_args(1..))]
        steps: Vec<Step>,
    },

    /// Install all components and crates used by the xtask script.
    #[command(short_flag = 'i')]
    Install,

    /// Clean the directory of all build artifacts.
    #[command(short_flag = 'c')]
    Clean,
}

#[derive(Debug, Clone, ValueEnum)]
enum Step {
    CheckFormat,
    TestWithCoverage,
    GenerateCoverageLcov,
    GenerateCoverageHtml,
    WatchSelfWithDev,
    Release,
    Clean,
    InstallAll,
    VerifyCoveragePercent,
}

#[derive(Debug, Clone, ValueEnum)]
enum Workflow {
    Dev,
    DevWatch,
    PrValidation,
}

// The main entry point for the `xtask` CLI tool.
///
/// This function parses the CLI arguments and executes the appropriate steps.
///
/// # Returns
///
/// An `anyhow::Result` indicating whether the execution was successful.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Either break down the workflow into steps or take the raw steps from
    // the cli input and execute each step sequentially.
    let result = std::panic::catch_unwind(|| {
        match cli.command {
            Commands::Workflow { workflow } => workflow_steps(workflow),
            Commands::Steps { steps } => steps,
            Commands::Install => vec![Step::InstallAll],
            Commands::Clean => vec![Step::Clean],
        }
        .iter()
        .try_for_each(|step| perform_step(step, &cli.options))
        .expect("Run command steps")
    });

    // Print whether the output
    match result.is_ok() {
        true => println!("{}", "xtask Finished Successfully".bold().green()),
        false => println!("{}", "xtask Failed".bold().red()),
    }
    Ok(())
}

// Converts a workflow into its constituent steps.
fn workflow_steps(workflow: Workflow) -> Vec<Step> {
    match workflow {
        // The PrValidation steps should be updated in lock-step with the github
        // workflows that run when someone makes a pull request.
        Workflow::PrValidation => vec![
            Step::CheckFormat,
            Step::TestWithCoverage,
            Step::VerifyCoveragePercent,
            Step::Release,
        ],
        Workflow::Dev => vec![
            // Generate coverage first so that you still get coverage
            // results even if the formatting is bad wrong.
            Step::TestWithCoverage,
            Step::GenerateCoverageLcov,
            Step::GenerateCoverageHtml,
            Step::CheckFormat,
        ],
        Workflow::DevWatch => vec![
            // This is a little hacky, but ultimately it runs back to
            // the `Dev` workflow.
            Step::WatchSelfWithDev,
        ],
        // TODO:
        // Workflow::LocalDeploy => vec!(),
        // Workflow::DeployCrate => vec!(),
        // etc.
    }
}

/// Executes a CI step.
///
/// Takes in a step and options and passes in the options to a function that
/// is responsible for executing that step.
///
/// # Arguments
///
/// * `components`: The components to ensure are installed.
/// * `options`: The options for the command
///
/// # Returns
///
/// An `anyhow::Result` of `()` if the step ran successfully.
fn perform_step(step: &Step, options: &Options) -> anyhow::Result<()> {
    match step {
        Step::Clean => clean_step(options),
        Step::TestWithCoverage => test_with_coverage_step(options),
        Step::GenerateCoverageHtml => generate_coverage_html_step(options),
        Step::GenerateCoverageLcov => generate_coverage_lcov_step(options),
        Step::CheckFormat => check_format_step(options),
        Step::Release => release_step(),
        Step::InstallAll => install_all_step(),
        Step::WatchSelfWithDev => watch_self_with_dev_step(options),
        Step::VerifyCoveragePercent => verify_coverage_percent_step(options),
    }
}

// Somewhat hacky step that calls this program again with the "dev" workflow under
// the command `cargo watch` so that the `dev` workflow runs every time the files update.
fn watch_self_with_dev_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-watch"])?;
    }
    // Watch for updates and run the dev workflow when an update is detected.
    create_dir_all(COVERAGE_DIRECTORY)?;
    cmd!(
        "cargo",
        "watch",
        "--ignore",
        COVERAGE_DIRECTORY,
        "-x",
        "xtask workflow dev"
    )
    .run()?;
    Ok(())
}

// Install all dependencies required by any step in this script.
fn install_all_step() -> anyhow::Result<()> {
    ensure_crates_are_installed(vec![
        "cargo-llvm-cov", // cargo-llvm-cov https://crates.io/crates/cargo-llvm-cov
        "cargo-watch",    // cargo-watch https://crates.io/crates/cargo-watch
    ])?;
    ensure_components_are_installed(vec!["clippy-preview", "rustfmt"])?;
    Ok(())
}

// run cargo build release.
fn release_step() -> anyhow::Result<()> {
    cmd!("cargo", "build", "--release").run()?;
    Ok(())
}

// run the tests with coverage analysis.
fn test_with_coverage_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-llvm-cov"])?;
    }
    create_dir_all(COVERAGE_DIRECTORY)?;
    cmd!("cargo", "llvm-cov", "test", "--workspace", "--no-report").run()?;
    Ok(())
}

// verify that the code coverage is above a certain percent.
fn verify_coverage_percent_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-llvm-cov"])?;
    }
    create_dir_all(COVERAGE_DIRECTORY)?;
    // This command has to generate an output
    cmd!(
        "cargo",
        "llvm-cov",
        "report",
        "--fail-under-lines",
        options.min_coverage.to_string()
    )
    .stdout_null()
    .run()
    .unwrap_or_else(|_| {
        panic!(
            "Verify code coverage is above the required percentage {}",
            options.min_coverage
        )
    });
    Ok(())
}

// generate coverage html website.
fn generate_coverage_html_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-llvm-cov"])?;
    }
    create_dir_all(COVERAGE_DIRECTORY)?;
    cmd!(
        "cargo",
        "llvm-cov",
        "report",
        "--html",
        "--output-dir",
        COVERAGE_DIRECTORY
    )
    .run()?;
    Ok(())
}

// generate coverage lcov file.
fn generate_coverage_lcov_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-llvm-cov"])?;
    }
    create_dir_all(COVERAGE_DIRECTORY)?;
    cmd!(
        "cargo",
        "llvm-cov",
        "report",
        "--lcov",
        "--output-path",
        COVERAGE_LCOV_FILE
    )
    .run()?;
    Ok(())
}

/// Cleans the workspace of build artifacts.
fn clean_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_crates_are_installed(vec!["cargo-llvm-cov"])?;
    }
    cmd!("cargo", "clean").run()?;
    cmd!("cargo", "llvm-cov", "clean").run()?;
    remove_dir_all(COVERAGE_DIRECTORY)?;
    Ok(())
}

/// Runs clippy on workwspace with some agressive linting - fails on warning.
fn check_format_step(options: &Options) -> anyhow::Result<()> {
    if options.lazy_install {
        ensure_components_are_installed(vec!["clippy-preview", "rustfmt"])?;
    }
    cmd!("cargo", "fmt", "--check", "--verbose").run()?;
    // cmd!("cargo", "fmt", "--all", "--", "--check", "--verbose").run()?;
    cmd!(
        "cargo",
        "clippy",
        "--",
        "-D",
        "warnings",
        "-W",
        "clippy::all",
        // TODO: Enable the warning flag below.
        // "-W",
        // "clippy::cargo"
    )
    .run()?;
    Ok(())
}

/// Ensures that the given crates are installed by attempting to install them.
fn ensure_crates_are_installed(crates: Vec<&str>) -> anyhow::Result<()> {
    for crate_ in crates {
        cmd!("cargo", "install", crate_).run()?;
    }
    Ok(())
}

/// Ensures that the given components are installed by attempting to install them.
fn ensure_components_are_installed(components: Vec<&str>) -> anyhow::Result<()> {
    for component in components {
        cmd!("rustup", "component", "add", component).run()?;
    }
    Ok(())
}
