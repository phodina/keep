#[macro_use]
extern crate error_chain;
extern crate git2;

use std::path::Path;
use git2::{Commit, ObjectType, Oid, Repository, Signature, Sort};

// TEST: Have template dir & copy it

error_chain!{
    foreign_links{
        Git(::git2::Error);
    }
}

pub struct Backup {
    repo: Repository,
    last_commit: Option<Oid>,
    previous_commit: Option<Oid>,
    new: bool
}

impl Backup {
    pub fn new(path: &Path) -> Result<Backup> {
        let mut new = false;
        let repo = match Repository::open(path)
        {
            Ok(repo) => repo,
            Err(e) => {
                new = true;
                println!("Create new git repo!");
                Repository::init(path)?}
        };
        
        println!("Repo: {}", path.to_str().ok_or("Invalid repo path")?);

        Ok(Backup {
            repo: repo,
            last_commit: None,
            previous_commit: None,
            new: new
        })
    }

    pub fn process_backup(&mut self) -> Result<()> {

        if self.new {
            println!("Create first commit!");
            self.create_first_commit()?;
            self.new = false;
        }
        //let index = self.repo.index()?;

        match self.repo.state() {
            git2::RepositoryState::Clean => {
                println!("Repo clean");
            }
            _ => {
                println!("Repo dirty");
                self.create_commit("ASPC")?;
            }
        }

        Ok(())
    }

    pub fn resolve_changes(&mut self) -> Result<()> {
        let mut walker = self.repo.revwalk()?;
        let mut sort = Sort::empty();
        sort.set(Sort::TOPOLOGICAL, true);
        sort.set(Sort::TIME, true);
        walker.set_sorting(sort);
        walker.push_head()?;

        for oid in walker {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.message().ok_or("Invalid commit message")?;

            println!("ID: {} Msg: {}", oid, message);
            let tree_id = commit.tree_id();

            if self.last_commit.is_none() {
                println!("Last: {}", oid);
                self.last_commit = Some(tree_id);
                continue;
            }

            //let message = commit.message().ok_or("Invalid commit message")?;

            if message.starts_with("AGNC") {
                println!("Previous: {}", oid);
                self.previous_commit = Some(tree_id);
                break;
            }
        }

        // TODO: Return lists of modified files
        /*
        println!("Found last tree");
        let last_tree_ref = Some(&last_tree);

        if last_tree_ref.is_some() && previous_tree_ref.is_some() {
            let diffs = self.repo
                .diff_tree_to_tree(last_tree_ref, previous_tree_ref, None)?;

            for diff in diffs.deltas() {
                println!("Diff: {:?}", diff.new_file().path().unwrap());
                repo.find
            }
        } else {
            println!("Plant more trees");
        }*/
        Ok(())
    }

    pub fn merge_files(&self) -> Result<()> {
        let previous_tree = self.repo
            .find_tree(self.previous_commit.ok_or("Invalid previous commit")?)?;

        let last_tree = self.repo
            .find_tree(self.last_commit.ok_or("Invalid last commit")?)?;

        let mut index = self.repo
            .merge_trees(&previous_tree, &last_tree, &last_tree, None)?;

        if index.has_conflicts(){
            println!("Confilts during merge");
        }

        let oid = index.write_tree_to(&self.repo)?;
        println!("OID: {}", oid);
        Ok(())
    }

    pub fn finish_backup(&self) -> Result<()> {
        self.create_commit("AGNC")?;
        Ok(())
    }

    fn get_last_commit(&self) -> Result<Commit> {
        let obj = self.repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        Ok(obj.into_commit().unwrap())
    }

    fn create_first_commit(&self) -> Result<Oid> {
        // First use the config to initialize a commit signature for the user.
        let sig = self.repo.signature()?;

        // Now let's create an empty tree for this commit
        let tree_id = {
            let mut index = self.repo.index()?;

            // Outside of this example, you could call index.add_path()
            // here to put actual files into the index. For our purposes, we'll
            // leave it empty for now.

            index.write_tree()?
        };

        let tree = self.repo.find_tree(tree_id)?;

        // Ready to create the initial commit.
        //
        // Normally creating a commit would involve looking up the current HEAD
        // commit and making that be the parent of the initial commit, but here this
        // is the first commit so there will be no parent.
        Ok(self.repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?)
    }
    
    fn create_commit(&self, message: &str) -> Result<Oid> {
        let mut index = self.repo.index()?;
        let oid = index.write_tree()?;
        let signature = Signature::now("Petr Hodina", "hodinapetr46@gmail.com")?;
        
        let tree = self.repo.find_tree(oid)?;

        let reference = Some("HEAD");

        Ok(self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&self.get_last_commit()?],
        )?)
    }
}

#[cfg(test)]
mod tests {
    extern crate fs_extra;
    extern crate tempdir;
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn new_repository() {
        let git_dir = tempdir::TempDir::new("git_dirty").unwrap();
        let src = Path::new("samples");

        let dst = PathBuf::from(git_dir.path());

        let mut backup = Backup::new(&dst).unwrap();
        match backup.process_backup() {
            Ok(_) => (),
            Err(e) => println!("Process: {}", e),
        }

        match backup.resolve_changes() {
            Ok(_) => (),
            Err(e) => println!("Resolve: {}", e),
        }

        match backup.merge_files() {
            Ok(_) => (),
            Err(e) => println!("Merge: {}", e),
        }
    }
    
    #[test]
    fn no_conflicts() {
        let git_dir = tempdir::TempDir::new("git_dirty").unwrap();
        let options = fs_extra::dir::CopyOptions::new();
        let src = Path::new("samples");

        let mut dst = PathBuf::from(git_dir.path());
        dst.push(src);

        fs_extra::dir::copy(&src, &git_dir.path(), &options).unwrap();
            
        let mut backup = Backup::new(&dst).unwrap();
        match backup.process_backup() {
            Ok(_) => (),
            Err(e) => println!("Process: {}", e),
        }

        match backup.resolve_changes() {
            Ok(_) => (),
            Err(e) => println!("Resolve: {}", e),
        }

        match backup.merge_files() {
            Ok(_) => (),
            Err(e) => println!("Merge: {}", e),
        }
    }

    #[test]
    fn handle_conflicts() {
        let git_dir = tempdir::TempDir::new("git_dirty").unwrap();
        let options = fs_extra::dir::CopyOptions::new();
        let src = Path::new("samples");
        fs_extra::dir::copy(&src, &git_dir.path(), &options).unwrap();

        let mut changed = git_dir.path().to_str().unwrap().to_owned();
        changed.push_str("/main.rs");
        
        let mut dst = PathBuf::from(git_dir.path());
        dst.push(src);

        let mut backup = Backup::new(&dst).unwrap();
        match backup.process_backup() {
            Ok(_) => (),
            Err(e) => println!("Process: {}", e),
        }

        std::fs::copy("dirty/main.rs", changed).unwrap();
        match backup.resolve_changes() {
            Ok(_) => (),
            Err(e) => println!("Resolve: {}", e),
        }

        match backup.merge_files() {
            Ok(_) => (),
            Err(e) => println!("Merge: {}", e),
        }
    }
}
