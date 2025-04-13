#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchVec<T> {
    pub arch: Option<String>,
    pub values: Vec<T>,
}

impl<T> ArchVec<T> {
    pub fn enabled(&self, arch: &str) -> bool {
        match &self.arch {
            Some(a) => a == arch,
            None => true,
        }
    }

    pub fn from_vec<S: Into<String>>(arch: Option<S>, vec: Vec<T>) -> Self {
        Self {
            arch: arch.map(|s| s.into()),
            values: vec,
        }
    }
}
