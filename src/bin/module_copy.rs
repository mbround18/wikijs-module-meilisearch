use clap::{Parser, Subcommand};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};
use walkdir::WalkDir;

const ENV_SOURCE: &str = "SOURCE";
const ENV_DESTINATION: &str = "DESTINATION";
const DEFAULT_SOURCE: &str = "/modules/meilisearch";
const DEFAULT_DESTINATION: &str = "/wiki/server/modules/meilisearch";

#[derive(Parser, Debug)]
#[command(
    name = "meili-module-util",
    about = "Utility for Wiki.js Meilisearch module delivery",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Copy module files from SOURCE to DESTINATION (defaults can come from env vars or constants)
    Copy {
        /// Source directory (overrides env if provided)
        source: Option<PathBuf>,
        /// Destination directory (overrides env if provided)
        destination: Option<PathBuf>,
        /// Replace existing destination contents before copy
        #[arg(long)]
        replace: bool,
        /// Perform a dry run: log actions without modifying files
        #[arg(long)]
        dry_run: bool,
    },
}

fn resolve_paths(source: Option<PathBuf>, destination: Option<PathBuf>) -> (PathBuf, PathBuf) {
    let src = source
        .or_else(|| env::var(ENV_SOURCE).ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOURCE));
    let dst = destination
        .or_else(|| env::var(ENV_DESTINATION).ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DESTINATION));
    (src, dst)
}

fn clear_destination(dst: &Path) -> io::Result<Vec<PathBuf>> {
    if dst.exists() {
        let mut removed = Vec::new();
        for entry in fs::read_dir(dst)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            removed.push(path);
        }
        Ok(removed)
    } else {
        Ok(Vec::new())
    }
}

fn list_destination_entries(dst: &Path) -> io::Result<Vec<PathBuf>> {
    if dst.exists() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(dst)? {
            let entry = entry?;
            entries.push(entry.path());
        }
        Ok(entries)
    } else {
        Ok(Vec::new())
    }
}

fn copy_recursive(src: &Path, dst: &Path) -> io::Result<Vec<PathBuf>> {
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Source must be directory",
        ));
    }
    fs::create_dir_all(dst)?;
    let mut copied = Vec::new();
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap();
        let dest_path = dst.join(rel);
        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            fs::copy(path, &dest_path)?;
            copied.push(dest_path);
        }
    }
    Ok(copied)
}

fn list_files_to_copy(src: &Path, dst: &Path) -> io::Result<Vec<PathBuf>> {
    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Source must be directory",
        ));
    }
    let mut copied = Vec::new();
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap();
        let dest_path = dst.join(rel);
        if path.is_file() {
            copied.push(dest_path);
        }
    }
    Ok(copied)
}

fn read_version_file(dst: &Path) -> Option<String> {
    // Try both root and nested pkg VERSION placements
    let candidates = [dst.join("VERSION"), dst.join("pkg/VERSION")];
    for c in candidates {
        if c.exists() {
            if let Ok(contents) = fs::read_to_string(&c) {
                return Some(contents.trim().to_string());
            }
        }
    }
    None
}

fn run_copy(
    source: Option<PathBuf>,
    destination: Option<PathBuf>,
    replace: bool,
    dry_run: bool,
) -> io::Result<()> {
    let (src, dst) = resolve_paths(source, destination);
    info!(?src, ?dst, replace, dry_run, "Starting copy operation");

    if dry_run {
        if replace {
            let removed = list_destination_entries(&dst)?;
            if removed.is_empty() {
                debug!("DRY-RUN: No existing files to remove in destination");
            } else {
                for p in &removed {
                    debug!(would_remove=?p);
                }
                info!(count = removed.len(), "DRY-RUN: Destination would be cleared");
            }
        }

        let copied = list_files_to_copy(&src, &dst)?;
        if copied.is_empty() {
            warn!("DRY-RUN: No files would be copied; source may contain only directories");
        }
        for p in &copied {
            debug!(would_copy=?p);
        }
        info!(count = copied.len(), "DRY-RUN: Files that would be copied");

        if let Some(version) = read_version_file(&src) {
            info!(version, "DRY-RUN: Discovered VERSION in source");
        } else {
            debug!("DRY-RUN: No VERSION file found in source");
        }
        if let Some(version) = read_version_file(&dst) {
            info!(version, "DRY-RUN: Destination currently has VERSION");
        } else {
            warn!("DRY-RUN: Destination currently has no VERSION file");
        }
        return Ok(());
    }

    if replace {
        let removed = clear_destination(&dst)?;
        if removed.is_empty() {
            debug!("No existing files to remove in destination");
        } else {
            for p in &removed {
                debug!(removed=?p);
            }
            info!(count = removed.len(), "Destination cleared");
        }
    }

    let copied = copy_recursive(&src, &dst)?;
    if copied.is_empty() {
        warn!("No files copied; source may have contained only directories");
    }
    for p in &copied {
        debug!(copied=?p);
    }
    info!(count = copied.len(), "Files copied successfully");

    if let Some(version) = read_version_file(&src) {
        info!(version, "Discovered VERSION in source before copy");
    } else {
        debug!("No VERSION file found in source prior to copy");
    }
    if let Some(version) = read_version_file(&dst) {
        info!(version, "Discovered VERSION in destination after copy");
    } else {
        warn!("No VERSION file found in destination after copy");
    }
    Ok(())
}

fn main() {
    // Initialize tracing subscriber once
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_target(false)
        .with_env_filter(filter)
        .without_time()
        .with_ansi(false)
        .with_writer(std::io::stdout)
        .init();
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Copy {
            source,
            destination,
            replace,
            dry_run,
        } => run_copy(source, destination, replace, dry_run),
    };
    if let Err(e) = result {
        error!(error=%e, "Copy operation failed");
        let _ = writeln!(io::stderr(), "Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    fn write_file(p: &Path, content: &str) {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = File::create(p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn copies_directory_tree() {
        let src_dir = tempdir().unwrap();
        let dst_dir = tempdir().unwrap();
        write_file(&src_dir.path().join("a/b/c.txt"), "hello");
        write_file(&src_dir.path().join("root.txt"), "root");

        run_copy(
            Some(src_dir.path().to_path_buf()),
            Some(dst_dir.path().to_path_buf()),
            false,
            false,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(dst_dir.path().join("a/b/c.txt")).unwrap(),
            "hello"
        );
        assert_eq!(
            fs::read_to_string(dst_dir.path().join("root.txt")).unwrap(),
            "root"
        );
    }

    #[test]
    fn replace_clears_destination_first() {
        let src_dir = tempdir().unwrap();
        let dst_dir = tempdir().unwrap();
        // pre-populate destination with a file that shouldn't survive
        write_file(&dst_dir.path().join("to-be-removed.txt"), "old");
        // source has different files
        write_file(&src_dir.path().join("only-in-src.txt"), "new");

        run_copy(
            Some(src_dir.path().to_path_buf()),
            Some(dst_dir.path().to_path_buf()),
            true,
            false,
        )
        .unwrap();

        assert!(!dst_dir.path().join("to-be-removed.txt").exists());
        assert!(dst_dir.path().join("only-in-src.txt").exists());
    }

    #[test]
    fn dry_run_makes_no_changes() {
        let src_dir = tempdir().unwrap();
        let dst_dir = tempdir().unwrap();

        // Pre-populate both sides
        write_file(&src_dir.path().join("new/file.txt"), "content");
        write_file(&dst_dir.path().join("old/file.txt"), "old");

        // Run in dry-run with replace=true
        run_copy(
            Some(src_dir.path().to_path_buf()),
            Some(dst_dir.path().to_path_buf()),
            true,
            true,
        )
        .unwrap();

        // Destination should be unchanged
        assert!(dst_dir.path().join("old/file.txt").exists());
        assert!(!dst_dir.path().join("new/file.txt").exists());
    }
}
