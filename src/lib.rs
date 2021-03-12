//! Common types for the public suffix implementation crates

#![no_std]
#![forbid(unsafe_code)]

/// A list of all public suffixes
pub trait List<'a> {
    /// Finds the suffix information of the given input labels
    fn find<T>(&self, labels: T) -> Info
    where
        T: Iterator<Item = &'a [u8]>;

    /// Get the public suffix of the domain
    ///
    /// *NB:* `name` must be a valid domain name in lowercase
    #[inline]
    fn suffix(&self, name: &'a [u8]) -> Option<Suffix<'a>> {
        let mut labels = name.rsplit(|x| *x == b'.');
        let fqdn = if name.ends_with(b".") {
            labels.next();
            true
        } else {
            false
        };
        let Info { mut len, typ } = self.find(labels);
        if fqdn {
            len += 1;
        }
        if len == 0 {
            return None;
        }
        let offset = name.len() - len;
        let bytes = name.get(offset..)?;
        Some(Suffix { bytes, fqdn, typ })
    }

    /// Get the registrable domain
    ///
    /// *NB:* `name` must be a valid domain name in lowercase
    #[inline]
    fn domain(&self, name: &'a [u8]) -> Option<Domain<'a>> {
        let suffix = self.suffix(name)?;
        let name_len = name.len();
        let suffix_len = suffix.bytes.len();
        if name_len < suffix_len + 2 {
            return None;
        }
        let offset = name_len - (1 + suffix_len);
        let subdomain = name.get(..offset)?;
        let root_label = subdomain.rsplitn(2, |x| *x == b'.').next()?;
        let registrable_len = root_label.len() + 1 + suffix_len;
        let offset = name_len - registrable_len;
        let bytes = name.get(offset..)?;
        Some(Domain { bytes, suffix })
    }
}

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
    bytes: &'a [u8],
    fqdn: bool,
    typ: Option<Type>,
}

impl Suffix<'_> {
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub const fn is_fqdn(&self) -> bool {
        self.fqdn
    }

    #[inline]
    pub const fn typ(&self) -> Option<Type> {
        self.typ
    }

    #[inline]
    pub const fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

impl PartialEq<&[u8]> for Suffix<'_> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.bytes == *other
    }
}

impl PartialEq<&str> for Suffix<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.bytes == other.as_bytes()
    }
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Domain<'a> {
    bytes: &'a [u8],
    suffix: Suffix<'a>,
}

impl Domain<'_> {
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub const fn suffix(&self) -> Suffix<'_> {
        self.suffix
    }
}

impl PartialEq<&[u8]> for Domain<'_> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.bytes == *other
    }
}

impl PartialEq<&str> for Domain<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.bytes == other.as_bytes()
    }
}