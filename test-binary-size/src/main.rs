use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_project_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    path.pop();

    path
}

fn run(command: &mut Command) -> io::Result<()> {
    eprintln!("===> {command:?}");

    if command.status()?.success() {
        Ok(())
    } else {
        Err(ErrorKind::Other.into())
    }
}

fn check_size(
    project_dir: &Path,
    opt_level: &str,
    lto: &str,
    codegen_units: u8,
    extra_args: &[&str],
) -> io::Result<u64> {
    run(Command::new("cargo")
        .current_dir(project_dir)
        .args([
            "build",
            "--bin",
            "cargo-binstall",
            "--release",
            "--config",
            format!("profile.release.opt-level={opt_level}").as_str(),
            "--config",
            format!("profile.release.lto={lto:?}").as_str(),
            "--config",
            format!("profile.release.codegen-units={codegen_units}").as_str(),
        ])
        .args(extra_args))?;

    Ok(fs::metadata(project_dir.join("target/release/cargo-binstall"))?.len())
}

fn main() -> io::Result<()> {
    let project_dir = get_project_dir();

    println!("| opt-level | lto  | codegen-units | original size | optimized size | diff   |");
    println!("| --------- | ---- | ------------- | ------------- | -------------- | ------ |");

    for opt_level in ["3", "\"z\""] {
        for lto in ["off", "thin", "fat"] {
            for codegen_units in [1, 16] {
                let original_size = check_size(
                    &project_dir,
                    opt_level,
                    lto,
                    codegen_units,
                    &[
                        "--config",
                        "patch.crates-io.log.git=\"ssh://git@github.com/EFanZh/log.git\"",
                        "--config",
                        "patch.crates-io.log.rev=\"6042ec8a85f23d7e47df66074d6427b0c01ab90f\"",
                    ],
                )?;

                let optimized_size = check_size(
                    &project_dir,
                    opt_level,
                    lto,
                    codegen_units,
                    &[
                        "--config",
                        "patch.crates-io.log.git=\"ssh://git@github.com/EFanZh/log.git\"",
                        "--config",
                        "patch.crates-io.log.rev=\"13152ab0e2057f7a12400d7b0a9d21c93c86c311\"",
                    ],
                )?;

                let diff = optimized_size as i64 - original_size as i64;

                println!("| {opt_level:9} | {lto:4} | {codegen_units:13} | {original_size:13} | {optimized_size:14} | {diff:6} |");
            }
        }
    }

    Ok(())
}
