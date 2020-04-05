use crate::integration_impl::{bindings::Parser, error::Error, tests::helpers::dirs::DATA_DIR};
use git2::{build::RepoBuilder, BranchType, MergeOptions, Repository, ResetType};
use std::path::PathBuf;
#[cfg(unix)]
use std::process::{Child, ChildStdin, Command, Stdio};
use std::{fs, path::Path};

#[cfg(unix)]
const HTML_HEADER: &[u8] = b"<!DOCTYPE html>\n<style>svg { width: 100%; }</style>\n\n";

#[cfg(windows)]
pub struct LogSession();

#[cfg(unix)]
pub struct LogSession(PathBuf, Option<Child>, Option<ChildStdin>);

#[cfg(windows)]
pub fn log_graphs(_parser: &mut Parser, _path: &str) -> std::io::Result<LogSession> {
    Ok(LogSession())
}

#[cfg(unix)]
pub fn log_graphs(parser: &mut Parser, path: &str) -> std::io::Result<LogSession> {
    use std::io::Write;

    let mut dot_file = std::fs::File::create(path)?;
    dot_file.write_all(HTML_HEADER)?;
    let mut dot_process = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(dot_file)
        .spawn()
        .expect("Failed to run Dot");
    let dot_stdin = dot_process
        .stdin
        .take()
        .expect("Failed to open stdin for Dot");
    parser.print_dot_graphs(&dot_stdin);
    Ok(LogSession(
        PathBuf::from(path),
        Some(dot_process),
        Some(dot_stdin),
    ))
}

#[cfg(unix)]
impl Drop for LogSession {
    fn drop(&mut self) {
        drop(self.2.take().unwrap());
        let output = self.1.take().unwrap().wait_with_output().unwrap();
        if output.status.success() {
            if cfg!(target_os = "macos")
                && fs::metadata(&self.0).unwrap().len() > HTML_HEADER.len() as u64
            {
                Command::new("open").arg(&self.0).output().unwrap();
            }
        } else {
            eprintln!(
                "Dot failed: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

pub fn clone_or_pull_repository(
    url: &str,
    branch: Option<&str>,
    path: &Path,
) -> Result<(), crate::integration_impl::error::Error> {
    let branch = branch.unwrap_or("master");
    if path.is_dir() {
        let repo = Repository::open(path)?;
        let mut remote_origin = repo.find_remote("origin")?;
        remote_origin.fetch(&[branch], None, None)?;

        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => match repo.find_branch(branch, BranchType::Local) {
                Ok(branch) => branch.into_reference(),
                Err(_) => {
                    let remote_branch = format!("origin/{}", branch);
                    let remote_commit = repo
                        .find_branch(&remote_branch, BranchType::Remote)?
                        .into_reference()
                        .peel_to_commit()?;
                    let mut branch = repo.branch(branch, &remote_commit, false)?;
                    branch.set_upstream(Some(&remote_branch))?;
                    branch.into_reference()
                }
            },
        };
        let head_commit = head.peel_to_commit()?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?.peel_to_commit()?;

        repo.merge_commits(&head_commit, &fetch_head, Some(&MergeOptions::default()))?;
        repo.reset(head_commit.as_object(), ResetType::Hard, None)?;
    } else {
        if path.exists() {
            fs::remove_file(&path)?;
        }

        RepoBuilder::new().branch(branch).clone(url, &path)?;
    }

    Ok(())
}

pub fn get_treesitter_repository() -> Result<PathBuf, Error> {
    use once_cell::sync::OnceCell;

    let repo_path = DATA_DIR.join("tree-sitter");

    static TREESITTER_REPO: OnceCell<()> = OnceCell::new();
    TREESITTER_REPO.get_or_try_init(|| {
        clone_or_pull_repository(
            "https://github.com/tree-sitter/tree-sitter.git",
            None,
            &repo_path,
        )
    })?;

    Ok(repo_path)
}
