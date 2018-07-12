#![no_std]
#![cfg_attr(feature = "dynamic", crate_type = "dylib")]

#[cfg(feature = "list")]
extern crate serde;
#[cfg(feature = "list")]
#[macro_use]
extern crate psl_codegen;

#[cfg(feature = "list")]
mod list;

use core::{str, fmt};

#[cfg(feature = "list")]
pub use list::List;

/// Type of suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Type {
    Icann,
    Private,
}

/// Information about the suffix
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Info {
    pub len: usize,
    pub typ: Option<Type>,
}

/// The suffix of a domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Suffix<'a> {
    str: &'a str,
    typ: Option<Type>,
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    str: &'a str,
    suf: Suffix<'a>,
}

/// A list of all public suffices
pub trait Psl {
    /// Finds the suffix of the given input labels
    ///
    /// # Assumptions
    ///
    /// *NB:* `domain` must be unicode and lowercase
    fn find(&self, domain: &str) -> Info;

    /// Get the public suffix of the domain
    /// 
    /// *NB:* `domain` must be unicode and lowercase
    #[inline]
    fn suffix<'a>(&self, domain: &'a str) -> Option<Suffix<'a>> {
        let Info { len, typ } = self.find(domain);
        if len == 0 {
            return None;
        }
        let offset = domain.len() - len;
        let bytes = domain.as_bytes();
        let str = str::from_utf8(&bytes[offset..]).ok()?;
        Some(Suffix { str, typ })
    }

    /// Get the registrable domain
    /// 
    /// *NB:* `domain` must be unicode and lowercase
    #[inline]
    fn domain<'a>(&self, domain: &'a str) -> Option<Domain<'a>> {
        let suf = self.suffix(domain)?;
        let mut labels = domain
            .trim_right_matches(suf.as_str())
            .split('.')
            .rev();
        // remove trailing dot
        labels.next()?;
        let root_label = labels.next()?;
        let registrable_len = root_label.len() + 1 + suf.as_str().len();
        let offset = domain.len() - registrable_len;
        let bytes = domain.as_bytes();
        let str = str::from_utf8(&bytes[offset..]).ok()?;
        Some(Domain { str, suf })
    }
}

impl<'a> Suffix<'a> {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.str
    }

    #[inline]
    pub fn typ(&self) -> Option<Type> {
        self.typ
    }

    #[inline]
    pub fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

impl<'a> Domain<'a> {
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.str
    }

    #[inline]
    pub fn suffix(&self) -> Suffix<'a> {
        self.suf
    }
}

impl<'a> fmt::Display for Suffix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl<'a> fmt::Display for Domain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Icann
    }
}
