
/// Single entry of the `Dtab`
#[derive(PartialEq, Eq, Debug)]
pub struct Dentry {
    pub key: String,
    pub val: String,
}

/// Delegate table.
#[derive(PartialEq, Eq, Debug)]
pub struct Dtab {
    pub entries: Vec<Dentry>,
}

impl Dentry {
    /// Create a new `Dentry` from the key-value pair.
    pub fn new(key: String, val: String) -> Dentry {
        Dentry {
            key: key,
            val: val,
        }
    }
}

impl Dtab {
    /// Create a new, empty `Dtab`.
    #[inline]
    pub fn new() -> Dtab {
        Dtab::from_entries(Vec::new())
    }

    /// Create a new `Dtab` containing the `Dentry`s.
    #[inline]
    pub fn from_entries(entries: Vec<Dentry>) -> Dtab {
        Dtab {
            entries: entries,
        }
    }

    /// Add an entry to this `Dtab`.
    #[inline]
    pub fn add_entry(&mut self, key: String, value: String) -> &Self {
        self.entries.push(Dentry::new(key, value));
        self
    }
}
