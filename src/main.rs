use git2::{Error, Repository};
use std::path::PathBuf;

fn get_last_commit_for_file(
    repo: &Repository,
    file_path: &PathBuf,
) -> Result<(String, String, i64), Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;

    let mut last_commit: Option<(String, String, i64)> = None;

    for commit_id in revwalk {
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;

        // Ignore merge commits (2+ parents) because that's what 'git whatchanged' does.
        // Ignore commit with 0 parents (initial commit) because there's nothing to diff against
        if commit.parent_count() == 1 {
            let prev_commit = commit.parent(0)?;
            let tree = commit.tree()?;
            let prev_tree = prev_commit.tree()?;
            let diff = repo.diff_tree_to_tree(Some(&prev_tree), Some(&tree), None)?;
            for delta in diff.deltas() {
                let actual_file_path = delta.new_file().path().unwrap();
                if actual_file_path == file_path {
                    let file_mod_time = commit.time();
                    let unix_time = file_mod_time.seconds();
                    let sha = commit.id().to_string();
                    let description = commit.summary().unwrap_or_default();

                    // Update the last commit if the commit's mtime is newer than the existing one
                    if let Some((_, _, last_commit_time)) = last_commit {
                        if unix_time > last_commit_time {
                            last_commit = Some((sha, description.to_string(), unix_time));
                        }
                    } else {
                        last_commit = Some((sha, description.to_string(), unix_time));
                    }
                }
            }
        }
    }

    Ok(last_commit.unwrap_or_default())
}

fn main() -> Result<(), Error> {
    let repo = Repository::open(".")?;
    let file_path = PathBuf::from("src/main.rs");
    let (sha, description, _) = get_last_commit_for_file(&repo, &file_path)?;

    println!(
        "Last commit for {}: {} - {}",
        file_path.to_str().unwrap_or_default(),
        sha,
        description
    );

    Ok(())
}
