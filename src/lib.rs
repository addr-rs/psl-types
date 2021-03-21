//! Common types for the public suffix implementation crates

#![no_std]
#![forbid(unsafe_code)]

use core::cmp::Ordering;

/// A list of all public suffixes
pub trait List {
    /// Finds the suffix information of the given input labels
    ///
    /// *NB:* `labels` must be in reverse order
    fn find<'a, T>(&self, labels: T) -> Info
    where
        T: Iterator<Item = &'a [u8]>;

    /// Get the public suffix of the domain
    ///
    /// *NB:* `name` must be a valid domain name in lowercase
    #[inline]
    fn suffix<'a>(&self, name: &'a [u8]) -> Option<Suffix<'a>> {
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
    fn domain<'a>(&self, name: &'a [u8]) -> Option<Domain<'a>> {
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

impl<L: List> List for &'_ L {
    #[inline]
    fn find<'a, T>(&self, labels: T) -> Info
    where
        T: Iterator<Item = &'a [u8]>,
    {
        (*self).find(labels)
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
#[derive(Copy, Clone, Eq, Ord, Hash, Debug)]
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

    // Could be const but Isahc needs support for Rust v1.41
    #[inline]
    pub fn is_known(&self) -> bool {
        self.typ.is_some()
    }
}

impl PartialEq for Suffix<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.fqdn, other.bytes);
        this == other
    }
}

impl PartialEq<&[u8]> for Suffix<'_> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.fqdn, *other);
        this == other
    }
}

impl PartialEq<&str> for Suffix<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.fqdn, other.as_bytes());
        this == other
    }
}

impl PartialOrd for Suffix<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (this, other) = normalise_dot(self.bytes, self.fqdn, other.bytes);
        Some(this.cmp(other))
    }
}

/// A registrable domain name
#[derive(Copy, Clone, Eq, Ord, Hash, Debug)]
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

impl PartialEq for Domain<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.suffix.fqdn, other.bytes);
        this == other
    }
}

impl PartialEq<&[u8]> for Domain<'_> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.suffix.fqdn, *other);
        this == other
    }
}

impl PartialEq<&str> for Domain<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        let (this, other) = normalise_dot(self.bytes, self.suffix.fqdn, other.as_bytes());
        this == other
    }
}

impl PartialOrd for Domain<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (this, other) = normalise_dot(self.bytes, self.suffix.fqdn, other.bytes);
        Some(this.cmp(other))
    }
}

#[inline]
fn normalise_dot<'a>(
    mut this: &'a [u8],
    this_is_fqdn: bool,
    mut other: &'a [u8],
) -> (&'a [u8], &'a [u8]) {
    match (this_is_fqdn, other.ends_with(b".")) {
        (true, true) | (false, false) => {}
        (false, true) => {
            let other_len = other.len();
            if other_len > 0 {
                other = &other[..other_len - 1];
            }
        }
        (true, false) => {
            let this_len = this.len();
            this = &this[..this_len - 1];
        }
    }
    (this, other)
}

#[cfg(test)]
mod test {
    use super::{Info, List as Psl};

    struct List;

    impl Psl for List {
        fn find<'a, T>(&self, mut labels: T) -> Info
        where
            T: Iterator<Item = &'a [u8]>,
        {
            match labels.next() {
                Some(label) => Info {
                    len: label.len(),
                    typ: None,
                },
                None => Info { len: 0, typ: None },
            }
        }
    }

    #[test]
    fn www_example_com() {
        let domain = List.domain(b"www.example.com").expect("domain name");
        assert_eq!(domain, "example.com");
        assert_eq!(domain.suffix(), "com");
    }

    #[test]
    fn example_com() {
        let domain = List.domain(b"example.com").expect("domain name");
        assert_eq!(domain, "example.com");
        assert_eq!(domain.suffix(), "com");
    }

    #[test]
    fn example_com_() {
        let domain = List.domain(b"example.com.").expect("domain name");
        assert_eq!(domain, "example.com.");
        assert_eq!(domain.suffix(), "com.");
    }

    #[test]
    fn fqdn_comparisons() {
        let domain = List.domain(b"example.com.").expect("domain name");
        assert_eq!(domain, "example.com");
        assert_eq!(domain.suffix(), "com");
    }

    #[test]
    fn non_fqdn_comparisons() {
        let domain = List.domain(b"example.com").expect("domain name");
        assert_eq!(domain, "example.com.");
        assert_eq!(domain.suffix(), "com.");
    }

    #[test]
    fn self_comparisons() {
        let fqdn = List.domain(b"example.com.").expect("domain name");
        let non_fqdn = List.domain(b"example.com").expect("domain name");
        assert_eq!(fqdn, non_fqdn);
        assert_eq!(fqdn.suffix(), non_fqdn.suffix());
    }

    #[test]
    fn com() {
        let domain = List.domain(b"com");
        assert_eq!(domain, None);

        let suffix = List.suffix(b"com").expect("public suffix");
        assert_eq!(suffix, "com");
    }

    #[test]
    fn root() {
        let domain = List.domain(b".");
        assert_eq!(domain, None);

        let suffix = List.suffix(b".").expect("public suffix");
        assert_eq!(suffix, ".");
    }

    #[test]
    fn empty_string() {
        let domain = List.domain(b"");
        assert_eq!(domain, None);

        let suffix = List.suffix(b"");
        assert_eq!(suffix, None);
    }
}
