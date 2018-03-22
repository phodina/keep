
pub struct Backup {

}

impl Backup {

	pub fn new() -> Backup {

		Backup{}
	}

	pub fn process_backup(&self) {}
	
	pub fn resolve_changes(&self) {}
   	
   	pub fn merge_files(&self) {}
    
    pub fn finish_backup(&self) {} 
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
