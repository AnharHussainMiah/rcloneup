use anyhow::{bail, Context, Result};
use clap::Parser;
use std::{
    env,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Local directory to back up
    #[arg(long, default_value = "/path/to/backup/source", env = "BACKUP_SOURCE")]
    source: String,

    /// Rclone remote name
    #[arg(long, default_value = "minio", env = "RCLONE_REMOTE")]
    remote: String,

    /// Remote bucket/container name
    #[arg(long, default_value = "backup-bucket", env = "REMOTE_BUCKET")]
    bucket: String,

    /// MinIO server endpoint URL
    #[arg(long, default_value = "http://minio.local:9000", env = "MINIO_ENDPOINT")]
    endpoint: String,

    /// MinIO access key (required)
    #[arg(long, env = "MINIO_ACCESS_KEY")]
    access_key: String,

    /// MinIO secret key (required)
    #[arg(long, env = "MINIO_SECRET_KEY")]
    secret_key: String,

    /// Cron schedule expression (default hourly)
    #[arg(long, default_value = "0 * * * *", env = "CRON_SCHEDULE")]
    cron: String,

    /// Enable verbose logging
    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    /// Dry-run mode (show actions without making changes)
    #[arg(long, short, default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate inputs
    validate_args(&args)?;

    if args.verbose {
        println!("Args: {:#?}", args);
    }

    if !is_rclone_installed()? {
        println!("Warning: 'rclone' not found in PATH. Please install it before proceeding.");
    } else if args.verbose {
        println!("Found 'rclone' in PATH.");
    }

    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let rclone_config_dir = home_dir.join(".config").join("rclone");
    let rclone_config_file = rclone_config_dir.join("rclone.conf");
    let backup_script = home_dir.join("rclone_backup.sh");

    if args.verbose {
        println!("Rclone config dir: {}", rclone_config_dir.display());
        println!("Rclone config file: {}", rclone_config_file.display());
        println!("Backup script: {}", backup_script.display());
    }

    if !args.dry_run {
        fs::create_dir_all(&rclone_config_dir)
            .with_context(|| format!("Failed to create rclone config directory {:?}", rclone_config_dir))?;
    } else if args.verbose {
        println!("(dry-run) Would create directory: {}", rclone_config_dir.display());
    }

    let config_content = format!(
        r#"[{remote}]
type = s3
provider = Minio
env_auth = false
access_key_id = {access_key}
secret_access_key = {secret_key}
endpoint = {endpoint}
"#,
        remote = args.remote,
        access_key = args.access_key,
        secret_key = args.secret_key,
        endpoint = args.endpoint
    );

    if args.dry_run {
        println!("(dry-run) Would write rclone config file to: {}", rclone_config_file.display());
        if args.verbose {
            println!("--- rclone.conf content ---\n{}", config_content);
        }
    } else {
        write_if_changed(&rclone_config_file, config_content.as_bytes(), 0o600, args.verbose)?;
    }

    let script_content = format!(
        r#"#!/bin/bash
rclone sync "{source}" "{remote}:{bucket}" --log-file="$HOME/rclone_backup.log" --log-level INFO --delete-during
"#,
        source = args.source,
        remote = args.remote,
        bucket = args.bucket
    );

    if args.dry_run {
        println!("(dry-run) Would write backup script to: {}", backup_script.display());
        if args.verbose {
            println!("--- backup script content ---\n{}", script_content);
        }
    } else {
        write_if_changed(&backup_script, script_content.as_bytes(), 0o755, args.verbose)?;
    }

    if args.dry_run {
        println!("(dry-run) Would update crontab to run backup script with schedule: '{}'", args.cron);
    } else {
        update_cron_job(&backup_script, &args.cron, args.verbose)?;
    }

    println!("Setup complete!");
    if args.dry_run {
        println!("(dry-run mode - no changes were made)");
    }
    println!("Remember to keep your access keys secure.");

    Ok(())
}

fn validate_args(args: &Args) -> Result<()> {
    if args.access_key.trim().is_empty() {
        bail!("MinIO access key must not be empty");
    }
    if args.secret_key.trim().is_empty() {
        bail!("MinIO secret key must not be empty");
    }
    if !PathBuf::from(&args.source).exists() {
        bail!("Backup source directory does not exist: {}", args.source);
    }
    // Basic cron schedule check (very basic)
    if args.cron.trim().split_whitespace().count() != 5 {
        bail!("Cron schedule must have exactly 5 fields, got '{}'", args.cron);
    }
    Ok(())
}

fn is_rclone_installed() -> Result<bool> {
    Ok(which::which("rclone").is_ok())
}

fn write_if_changed(path: &PathBuf, content: &[u8], perms: u32, verbose: bool) -> Result<()> {
    let need_write = if path.exists() {
        let existing = fs::read(path)?;
        existing != content
    } else {
        true
    };

    if need_write {
        if verbose {
            println!("Writing file: {}", path.display());
        }
        fs::write(path, content)?;
        let mut perms_obj = fs::metadata(path)?.permissions();
        perms_obj.set_mode(perms);
        fs::set_permissions(path, perms_obj)?;
    } else if verbose {
        println!("File up to date: {}", path.display());
    }
    Ok(())
}

fn update_cron_job(script_path: &PathBuf, cron_schedule: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("Updating crontab...");
    }

    let output = Command::new("crontab").arg("-l").output();

    let mut lines = Vec::new();

    match output {
        Ok(out) if out.status.success() => {
            let reader = BufReader::new(&out.stdout[..]);
            for line in reader.lines() {
                let line = line?;
                if !line.contains(script_path.to_str().unwrap()) {
                    lines.push(line);
                } else if verbose {
                    println!("Removing existing cron job line: {}", line);
                }
            }
        }
        _ => {
            if verbose {
                println!("No existing crontab found or error reading it, starting fresh.");
            }
        }
    }

    lines.push(format!("{} {}", cron_schedule, script_path.display()));

    if verbose {
        println!("New crontab lines:");
        for line in &lines {
            println!("  {}", line);
        }
    }

    let mut crontab_process = Command::new("crontab")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn crontab command")?;

    {
        let stdin = crontab_process.stdin.as_mut().context("Failed to open stdin")?;
        for line in &lines {
            writeln!(stdin, "{}", line)?;
        }
    }

    let status = crontab_process.wait()?;
    if !status.success() {
        bail!("Failed to install new crontab");
    }

    if verbose {
        println!("Crontab updated successfully.");
    }
    Ok(())
}