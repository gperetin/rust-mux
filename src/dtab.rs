
#[derive(PartialEq, Eq, Debug)]
pub struct Dentry {
    pub key: String,
    pub val: String,
}


#[derive(PartialEq, Eq, Debug)]
pub struct Dtab {
    pub entries: Vec<Dentry>,
}


impl Dentry {
    pub fn new(key: String, val: String) -> Dentry {
        Dentry {
            key: key,
            val: val,
        }
    }
}

impl Dtab {
    #[inline]
    pub fn new() -> Dtab {
        Dtab::from_entries(Vec::new())
    }

    #[inline]
    pub fn from_entries(entries: Vec<Dentry>) -> Dtab {
        Dtab {
            entries: entries,
        }
    }

    #[inline]
    pub fn add_entry(&mut self, key: String, value: String) -> &Self {
        self.entries.push(Dentry::new(key, value));
        self
    }
}
