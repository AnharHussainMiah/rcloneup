# rcloneup

rcloneup is a lightweight, Rust-based CLI tool to help you easily configure and schedule backups from a local directory to an S3-compatible storage (like MinIO) using rclone. It sets up the rclone configuration, creates a backup script, and manages a cron job to run periodic backups automatically.
Inspired by the Rust toolchain’s rustup, rcloneup brings the same simplicity and safety to backing up data from your homelab or personal projects.

## 🚀 Features

Automatically generates and manages rclone config for MinIO or other S3-compatible storage
Creates an idempotent backup shell script
Installs/updates a user cron job to run backups on a configurable schedule (default: hourly)
Supports environment variable overrides and CLI arguments for flexible configuration
Verbose and dry-run modes for safe preview and troubleshooting
Written in Rust for strong type safety, clear error handling, and performance

## 🗸🗸 Prerequisites

rclone installed and available in your system PATH
Access to an S3-compatible storage service (e.g., MinIO) with credentials
Basic familiarity with the command line and cron jobs
Rust is not required to run the compiled binary, but needed if you want to build from source

## 📦 Installation

You can either:
Download a prebuilt binary [downloads](https://github.com/AnharHussainMiah/rcloneup/releases)

OR

Build from source using Rust:

```shell
git clone https://github.com/anharhussainmiah/rcloneup.git
cd rcloneup
cargo build --release
```

The compiled binary will be at `target/release/rcloneup`.

## 📖 Usage

Run rcloneup with the minimum required parameters for your setup:

```shell
./rcloneup \
 --source /path/to/local/data \
 --access-key your-minio-access-key \
 --secret-key your-minio-secret-key
```

## ❗Important arguments

| Argument     | Description                                       | Default                | Env Variable     |
| ------------ | ------------------------------------------------- | ---------------------- | ---------------- |
| --source     | Local directory to back up /path/to/backup/source | BACKUP_SOURCE          |                  |
| --remote     | Name of the rclone remote (S3 provider) minio     | RCLONE_REMOTE          |                  |
| --bucket     | Bucket/container name on the remote backup-bucket | REMOTE_BUCKET          |                  |
| --endpoint   | S3 server endpoint URL (MinIO or similar)         | http://mino.local:9000 | MINIO_ENDPOINT   |
| --access-key | MinIO access key (required)                       | none                   | MINIO_ACCESS_KEY |
| --secret-key | MinIO secret key (required)                       | none                   | MINIO_SECRET_KEY |
| --cron       | Cron schedule expression for backup job           | 0 \* \* \* \* (hourly) | CRON_SCHEDULE    |
| --verbose    | Print detailed logs                               | false                  |                  |
| --dry-run    | Show planned actions without making changes       | false                  |                  |

## 🧪 Example with environment variables

```shell
export BACKUP_SOURCE=/home/user/data
export MINIO_ACCESS_KEY=myaccesskey
export MINIO_SECRET_KEY=mysecretkey

./rcloneup --verbose
```

## 🤔 What happens when you run rcloneup?

Checks if rclone is installed in your system
Creates or updates the `~/.config/rclone/rclone.conf` file with your MinIO credentials
Creates or updates a shell script `~/rclone_backup.sh` that runs rclone sync to your bucket
Installs or updates a cron job that runs the backup script according to your schedule

## 🦺 Safety notes

The tool is idempotent — running it multiple times won’t overwrite existing configuration unnecessarily.
Credentials are saved in `~/.config/rclone/rclone.conf` — keep this file secure and avoid sharing
Use the `--dry-run` flag to preview changes before applying them
Review your cron jobs with `crontab -l` to confirm the backup schedule

## 🪛 Troubleshooting

If backups aren’t running as expected, check the log at `~/rclone_backup.log`
Make sure rclone is installed and accessible (rclone --version)
Verify your MinIO endpoint and credentials are correct
Use `--verbose` mode to see detailed output when running the tool

## 🔑 Contributing

Contributions, issues, and feature requests are welcome! Feel free to open an issue or pull request on GitHub.

## 🔑 License

This project is licensed under the MIT License.

## 👏 Acknowledgments

Inspired by the Rust ecosystem’s simplicity and the power of rclone.

### 🤖 AI Assistance Disclosure

Parts of this project, including code examples and initial implementations, were generated with assistance from an AI language model (OpenAI's ChatGPT). The generated code has been reviewed, tested, and heavily adapted to fit the project requirements.

Users should review and test the code carefully before using it in production environments.
