use causis_core::run_leave_approval_demo;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 || args[0] != "demo" || args[1] != "leave-approval" {
        print_usage();
        return Ok(());
    }

    let fixture_dir = fixture_dir(args.get(2));
    let report = run_leave_approval_demo(&fixture_dir)?;
    let explanation = report.to_json_pretty();
    let output_path = default_output_path();
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, &explanation)?;

    println!("{}", report.summary());
    println!("explanation written to {}", output_path.display());
    println!("{explanation}");
    Ok(())
}

fn fixture_dir(arg: Option<&String>) -> PathBuf {
    arg.map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/leave-approval"))
}

fn print_usage() {
    println!("Causis CLI");
    println!();
    println!("Usage:");
    println!("  causis demo leave-approval [fixture-dir]");
}

fn default_output_path() -> PathBuf {
    PathBuf::from("target/causis/leave-approval/explanation.json")
}
