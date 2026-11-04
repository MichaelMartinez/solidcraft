//! Asset provenance gate. ATTRIBUTION.md is the single inventory of collected files.
use std::collections::BTreeSet;
use std::path::Path;

const ASSET_EXT: &[&str] = &[
    "png",
    "jpg",
    "jpeg",
    "svg",
    "ico",
    "icns",
    "car",
    "vectorcraft",
    "ai",
    "psd",
    "afdesign",
    "fig",
    "sketch",
    "ttf",
    "otf",
    "woff",
    "woff2",
    "gif",
    "bmp",
    "webp",
    "wav",
    "mp3",
    "mp4",
    "stl",
    "step",
    "stp",
    "obj",
    "fcstd",
    "3dm",
];

fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) -> Result<(), String> {
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().into_owned();
        // vendored code keeps its own licences: the patched crates (vendor/README.md) and the
        // kernel (solidcore/); their images, STEP files and fonts are not SolidCraft's assets.
        if name.starts_with('.')
            || name.starts_with("target")
            || matches!(name.as_str(), "plan" | "dist" | "node_modules" | "corpus" | "solidcore" | "vendor")
        {
            continue;
        }
        let p = e.path();
        let kind = e.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            if p.starts_with(root.join("assets")) {
                return Err(format!("resource symlink: {}", p.display()));
            }
        } else if kind.is_dir() {
            walk(&p, root, out)?;
        } else if kind.is_file() {
            out.push(p.strip_prefix(root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

/// The paths git's ignore rules cover. They are not part of the repository, so a local
/// scratch directory (`screenshots/`, a side checkout named in `.git/info/exclude`) cannot
/// fail the gate on files that never ship. `None` when git is unavailable or this is not a
/// checkout: the walker's own skips are then all there is.
fn git_ignored(root: &Path, files: &[String]) -> Result<Option<BTreeSet<String>>, String> {
    use std::io::Write as _;
    use std::process::Stdio;
    let spawned = std::process::Command::new("git")
        .current_dir(root)
        .args(["check-ignore", "--stdin", "-z"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let Ok(mut child) = spawned else { return Ok(None) };
    let mut input = Vec::with_capacity(files.iter().map(|file| file.len() + 1).sum());
    for file in files {
        input.extend_from_slice(file.as_bytes());
        input.push(0);
    }
    // Feed stdin from a thread: git answers while it still reads, and with many ignored
    // names a synchronous write would deadlock against a full stdout pipe. Joining after
    // wait_with_output is safe, because git's exit closes the pipe.
    let writer = child.stdin.take().map(|mut stdin| {
        std::thread::spawn(move || {
            // A path git rejects closes the pipe under us.
            let _ = stdin.write_all(&input);
        })
    });
    let out = child.wait_with_output().map_err(|e| format!("git check-ignore: {e}"))?;
    if let Some(writer) = writer {
        let _ = writer.join();
    }
    match out.status.code() {
        Some(0) => Ok(Some(String::from_utf8_lossy(&out.stdout).split('\0').filter(|path| !path.is_empty()).map(str::to_owned).collect())),
        // 1: nothing is ignored. Any other code (not a checkout) is no answer.
        Some(1) => Ok(Some(BTreeSet::new())),
        _ => Ok(None),
    }
}

/// Every file the gate considers, in a stable order: the walk minus what git ignores.
fn attributable(root: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort();
    if let Some(ignored) = git_ignored(root, &files)? {
        files.retain(|file| !ignored.contains(file));
    }
    Ok(files)
}

fn needs_attribution(path: &str) -> bool {
    // Include code masters, native work, catalogs, metadata and licence texts in the collection.
    if path.starts_with("assets/locales/pending/") {
        return false;
    }
    if path.starts_with("assets/") {
        return !path.ends_with(".md");
    }
    Path::new(path).extension().is_some_and(|ext| ASSET_EXT.contains(&ext.to_string_lossy().to_ascii_lowercase().as_str()))
}

fn missing<'a>(files: &'a [String], attribution: &str) -> Vec<&'a str> {
    let rows: BTreeSet<_> =
        attribution.lines().filter(|line| line.starts_with('|')).filter_map(|line| line.split('|').nth(1)).map(str::trim).collect();
    files.iter().map(String::as_str).filter(|path| needs_attribution(path) && !rows.contains(path)).collect()
}

pub fn run(root: &Path) -> Result<(), String> {
    let mut attribution = std::fs::read_to_string(root.join("ATTRIBUTION.md")).map_err(|e| format!("ATTRIBUTION.md: {e}"))?;
    // Rows still pending (attribution/pending/*.md) count: they are folded on main.
    for (_, row) in crate::pending::attribution_rows(root)? {
        attribution.push('\n');
        attribution.push_str(&row);
    }
    let files = attributable(root)?;
    let missing = missing(&files, &attribution);
    if !missing.is_empty() {
        return Err(format!("assets missing attribution rows:\n{}", missing.iter().map(|path| format!("  {path}")).collect::<Vec<_>>().join("\n")));
    }
    crate::render_assets::run(root, &files, &attribution)?;
    println!("assets: {} files, all attributed", files.iter().filter(|path| needs_attribution(path)).count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collected_code_native_work_and_metadata_require_rows() {
        for path in
            ["assets/icons/source/icon.rs", "assets/work/project.ai", "assets/fonts/OFL.txt", "assets/locales/en.json", "assets/macos/Assets.car"]
        {
            assert!(needs_attribution(path), "{path}");
        }
        assert!(needs_attribution("docs/screenshot.png"));
        assert!(needs_attribution("design/project.vectorcraft"));
        assert!(!needs_attribution("assets/README.md"));
        assert!(!needs_attribution("crates/ui/src/widget.rs"));
    }

    /// A scratch file git ignores never ships, so it must not be collected or need a row.
    #[test]
    fn ignored_files_are_not_collected() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!("xtask-assets-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("scratch")).unwrap();
        std::fs::write(root.join(".gitignore"), "scratch/\n").unwrap();
        std::fs::write(root.join("scratch").join("left-over.png"), b"").unwrap();
        std::fs::write(root.join("kept.png"), b"").unwrap();
        let init = std::process::Command::new("git").args(["init", "--quiet"]).arg(&root).output();
        if !init.is_ok_and(|out| out.status.success()) {
            // No git here: nothing to compare against, and the walker still runs.
            let _ = std::fs::remove_dir_all(&root);
            return;
        }
        let files = attributable(&root).unwrap();
        assert_eq!(files, ["kept.png"]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn vendored_code_is_not_an_asset() {
        let root = std::env::temp_dir().join(format!("solidcraft-xtask-assets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("solidcore/assets")).unwrap();
        std::fs::create_dir_all(root.join("vendor/patched")).unwrap();
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("solidcore/assets/logo.svg"), "x").unwrap();
        std::fs::write(root.join("vendor/patched/upstream.svg"), "x").unwrap();
        std::fs::write(root.join("assets/ours.svg"), "x").unwrap();
        let mut files = Vec::new();
        walk(&root, &root, &mut files).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(files, ["assets/ours.svg"]);
    }

    #[test]
    fn incidental_mentions_do_not_replace_attribution_rows() {
        let files = ["assets/source.svg".to_string(), "assets/README.md".to_string()];
        assert_eq!(missing(&files, "Mention assets/source.svg"), ["assets/source.svg"]);
        assert!(missing(&files, "| assets/source.svg | author | original | MIT |\n").is_empty());
    }
}
