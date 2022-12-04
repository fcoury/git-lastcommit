// Copyright 2021 Google, inc.
// SPDX-License-identifier: Apache-2.0

use git2::{Error, Repository};
use std::{cmp::max, collections::HashMap, path::PathBuf};

fn main() -> Result<(), Error> {
    let mut mtimes: HashMap<PathBuf, (i64, String)> = HashMap::new();
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
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
                let file_path = delta.new_file().path().unwrap();
                let file_mod_time = commit.time();
                let unix_time = file_mod_time.seconds();
                let sha = commit.id().to_string();
                mtimes
                    .entry(file_path.to_owned())
                    .and_modify(|(t, s)| {
                        *t = std::cmp::max(*t, unix_time);
                        *s = sha.to_owned();
                    })
                    .or_insert((unix_time, sha));
            }
        }
    }
    for (path, time) in mtimes.iter() {
        println!("{:?}: {:?}", path, time);
    }
    Ok(())
}
