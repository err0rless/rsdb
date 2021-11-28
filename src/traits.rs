use std::path::PathBuf;

pub trait Valid {
    fn valid(&self) -> bool;
    fn invalid(&self) -> bool { !self.valid() }
}

impl Valid for Option<PathBuf> {
    fn valid(&self) -> bool {
        if self.is_none() { return false }
        self.as_ref().unwrap().exists() && self.as_ref().unwrap().is_file()
    }
}
