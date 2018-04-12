#[macro_use]
extern crate error_chain;
extern crate git2;

use std::path::Path;
use git2::{Commit, ObjectType, Oid, Repository, RepositoryState, Signature};

error_chain!{
    foreign_links{
        Git(::git2::Error);
    }
}

pub struct Backup {
    repo: Repository,
}

impl Backup {
    pub fn new(path: &Path) -> Result<Backup> {
        let repo = Repository::open(path)?;
        println!("Repo: {}", path.to_str().unwrap());

        Ok(Backup { repo: repo })
    }

    pub fn process_backup(&self) {
        let index = self.repo.index().unwrap();

        match index.is_empty() {
            true => {
                println!("Repo clean");
            }
            false => {
                println!("Repo dirty");
                self.create_commit().unwrap();
            }
        }
    }

    pub fn resolve_changes(&self) {}

    pub fn merge_files(&self) {}

    pub fn finish_backup(&self) {}

    fn get_last_commit(&self) -> Result<Commit> {
        let obj = self.repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        Ok(obj.into_commit().unwrap())
    }

    fn create_commit(&self) -> Result<Oid> {
        let mut index = self.repo.index()?;
        let oid = index.write_tree()?;
        let signature = Signature::now("Petr Hodina", "hodinapetr46@gmail.com")?;
        let parent_commit = self.get_last_commit()?;
        let tree = self.repo.find_tree(oid)?;

        let message = "Message";
        Ok(self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let backup = Backup::new(Path::new("/tmp/backup")).unwrap();
        backup.process_backup();
    }
}
