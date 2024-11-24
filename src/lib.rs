//! A macro for mapping integers to Rust `enum`s with integer-numbered variants, plus
//! a catch-all `Unknown` variant.

/// A macro that implements useful functionality on integer-based `enum`s.
/// ```rust
/// yikes_intenum::yikes_intenum! {
///     /// IP datagram encapsulated protocol.
///     pub enum Protocol(u8) {
///         HopByHop  = 0x00,
///         Icmp      = 0x01,
///         Igmp      = 0x02,
///         Tcp       = 0x06,
///         Udp       = 0x11,
///         Ipv6Route = 0x2b,
///         Ipv6Frag  = 0x2c,
///         IpSecEsp  = 0x32,
///         IpSecAh   = 0x33,
///         Icmpv6    = 0x3a,
///         Ipv6NoNxt = 0x3b,
///         Ipv6Opts  = 0x3c
///     }
/// }
/// ```
#[macro_export]
macro_rules! yikes_intenum {
    (
        $( #[$enum_attr:meta] )*
        pub enum $name:ident($ty:ty) {
            $(
              $( #[$variant_attr:meta] )*
              $variant:ident = $value:expr
            ),+ $(,)?
        }
    ) => {
        paste::paste! {
            mod [< _ $name:snake _private >] {
                #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
                pub struct Sealed;
            }

            // #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
            #[derive(Eq, Clone, Copy)]
            // #[cfg_attr(feature = "defmt", derive(defmt::Format))]
            $( #[$enum_attr] )*
            #[repr($ty)]
            pub enum $name {
                $(
                $( #[$variant_attr] )*
                $variant
                ),*,
                Unknown {
                    value: $ty,
                    _private: [< _ $name:snake _private >]::Sealed
                }
            }

            // Debug
            impl ::core::fmt::Debug for $name {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        $( $name::$variant => ::core::fmt::Formatter::write_str(f, stringify!($variant)) ),*,
                        $name::Unknown{value: other, ..} => {
                            write!(f, "Unknown({})", other)
                        }
                    }
                }
            }

            // PartialEq (Eq is derived automatically)
            // impl ::core::marker::StructuralPartialEq for $name {}
            impl ::core::cmp::PartialEq for $name {
                #[inline]
                fn eq(&self, other: &$name) -> bool {
                    $ty::from(self).eq(&$ty::from(other))
                }
            }

            // PartialOrd, Ord
            impl ::core::cmp::PartialOrd for $name {
                #[inline]
                fn partial_cmp(&self, other: &$name) -> ::core::option::Option<::core::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }

            impl ::core::cmp::Ord for $name {
                #[inline]
                fn cmp(&self, other: &$name) -> ::core::cmp::Ordering {
                    $ty::from(self).cmp(&$ty::from(other))
                }
            }

            // Hash
            impl ::core::hash::Hash for $name {
                #[inline]
                fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
                    $ty::from(self).hash(state)
                }
            }

            impl ::core::convert::From<$ty> for $name {
                fn from(value: $ty) -> Self {
                    match value {
                        $( $value => $name::$variant ),*,
                        other => $name::Unknown{value: other, _private: [< _ $name:snake _private >]::Sealed}
                    }
                }
            }

            impl ::core::convert::From<&$name> for $ty {
                fn from(value: &$name) -> Self {
                    match value {
                        $( &$name::$variant => $value ),*,
                        &$name::Unknown{value: other, ..} => other
                    }
                }
            }

            impl ::core::convert::From<$name> for $ty {
                fn from(value: $name) -> Self {
                    (&value).into()
                }
            }
        } // paste::paste!
    }
} // macro_rules! yikes_intenum

// pub[(crate)] use yikes_intenum; // if not using `#[macro_export]`

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use core::hash::{BuildHasher, Hash, Hasher};

    use super::*;

    yikes_intenum! {
        pub enum TestIpProtocol(u8) {
            Icmp = 0x01_u8,
            Tcp = 0x06_u8,
        }
    }

    #[test]
    fn test_ipprotocol_roundtrip() {
        for i in 0..=u8::MAX {
            let a: TestIpProtocol = i.into();
            let b: u8 = a.into();
            let c: TestIpProtocol = b.into();
            let d: u8 = c.into();
            assert_eq!(a, c);
            assert_eq!(c, a);
            assert_eq!(b, d);
            assert_eq!(d, b);
        }
    }

    #[test]
    fn test_ipprotocol_debug() {
        for i in 0..=u8::MAX {
            let a: TestIpProtocol = i.into();
            match &a {
                TestIpProtocol::Icmp => {
                    assert_eq!("Icmp", format!("{a:?}"));
                }
                TestIpProtocol::Tcp => {
                    assert_eq!("Tcp", format!("{a:?}"));
                }
                TestIpProtocol::Unknown { value, .. } => {
                    assert_eq!(format!("Unknown({value})"), format!("{a:?}"));
                }
            }
        }
    }

    #[test]
    fn test_ipprotocol_eq() {
        for i in 0..=u8::MAX {
            let a: TestIpProtocol = i.into();
            let b = TestIpProtocol::Unknown {
                value: i,
                _private: _test_ip_protocol_private::Sealed,
            };
            assert!(
                a.eq(&b),
                "a {a:?} ({}) != b {b:?} ({})",
                u8::from(&a),
                u8::from(&b)
            );
            assert!(
                b.eq(&a),
                "b {b:?} ({}) != a {a:?} ({})",
                u8::from(&b),
                u8::from(&a)
            );
        }
    }

    #[test]
    fn test_ipprotocol_cmp() {
        #![allow(clippy::similar_names)]

        for i in 0..u8::MAX {
            for a in [
                i.into(),
                TestIpProtocol::Unknown {
                    value: i,
                    _private: _test_ip_protocol_private::Sealed,
                },
            ] {
                for j in (i + 1)..=u8::MAX {
                    for b in [
                        j.into(),
                        TestIpProtocol::Unknown {
                            value: j,
                            _private: _test_ip_protocol_private::Sealed,
                        },
                    ] {
                        let a_cmp_b = a.cmp(&b);
                        let a_pcmp_b = a.partial_cmp(&b);
                        let b_cmp_a = b.cmp(&a);
                        let b_pcmp_a = b.partial_cmp(&a);
                        let a_int = u8::from(&a);
                        let b_int = u8::from(&b);

                        assert_eq!(
                            a_cmp_b,
                            Ordering::Less,
                            "[cmp] a {a:?} ({a_int}) !< b {b:?} ({b_int})"
                        );
                        assert_eq!(
                            a_pcmp_b,
                            Some(Ordering::Less),
                            "[pcmp] a {a:?} ({a_int}) !< b {b:?} ({b_int})"
                        );
                        assert_eq!(
                            b_cmp_a,
                            Ordering::Greater,
                            "[cmp] b {b:?} ({b_int}) !> a {a:?} ({a_int})"
                        );
                        assert_eq!(
                            b_pcmp_a,
                            Some(Ordering::Greater),
                            "[pcmp] b {b:?} ({b_int}) !> a {a:?} ({a_int})"
                        );

                        // extra checks specifically for invariants that should hold
                        // for correct implementations of `Ord` and `PartialOrd`.
                        // (just in case the above is ever modified carelessly).

                        // check that `cmp` and `partial_cmp` agree.
                        assert_eq!(a_pcmp_b, Some(a_cmp_b), "a cmp b != a pcmp b");
                        assert_eq!(b_pcmp_a, Some(b_cmp_a), "b cmp a != b pcmp a!");

                        // check reversing args also reverses the `cmp` result.
                        assert_eq!(a_cmp_b, b_cmp_a.reverse());
                        assert_eq!(b_cmp_a, a_cmp_b.reverse());
                    }
                }
            }
        }
    }

    #[test]
    fn test_ipprotocol_hash() {
        for i in 0..u8::MAX {
            let a: TestIpProtocol = i.into();
            let b = TestIpProtocol::Unknown {
                value: i,
                _private: _test_ip_protocol_private::Sealed,
            };

            #[allow(unused_qualifications, clippy::type_complexity)]
            let hashers: Vec<(
                &str,
                Box<dyn core::hash::Hasher>,
                Box<dyn core::hash::Hasher>,
            )> = vec![
                (
                    "std::collections::hash_map::DefaultHasher",
                    Box::new(std::collections::hash_map::DefaultHasher::new()) as _,
                    Box::new(std::collections::hash_map::DefaultHasher::new()) as _,
                ),
                (
                    "FnvHasher",
                    Box::new(fnv::FnvBuildHasher::default().build_hasher()) as _,
                    Box::new(fnv::FnvBuildHasher::default().build_hasher()) as _,
                ),
            ];

            for (hasher_kind, mut hasher_a, mut hasher_b) in hashers {
                #[allow(clippy::unreadable_literal)]
                let noises: &[Option<u64>] = &[
                    None,
                    Some(18223650421099562965_u64),
                    Some(579348513557276885_u64),
                    Some(6018745257231369041_u64),
                    Some(4974397919804797078_u64),
                    Some(6574880736321336287_u64),
                    Some(8334883869055102477_u64),
                    Some(8077341428061256032_u64),
                    Some(8702568753483328048_u64),
                ];
                for noise in noises {
                    let assert_hashers_match =
                        |hasher_a: &mut dyn Hasher, hasher_b: &mut dyn Hasher| {
                            let ha = &mut hasher_a.finish();
                            let hb = &mut hasher_b.finish();
                            assert_eq!(
                                ha,
                                hb,
                                "[kind={hasher_kind}, noise={noise:?}] hash{{a {a:?} ({})}} {ha} != hash{{b {b:?} ({})}} {hb}",
                                u8::from(&a),
                                u8::from(&b),
                            );
                        };

                    if let Some(noise) = noise {
                        noise.hash(&mut hasher_a);
                        noise.hash(&mut hasher_b);
                        assert_hashers_match(&mut hasher_a, &mut hasher_b);
                    }
                    a.hash(&mut hasher_a);
                    b.hash(&mut hasher_b);
                    assert_hashers_match(&mut hasher_a, &mut hasher_b);
                }
                assert_eq!(
                    a,
                    b,
                    "[kind={hasher_kind}] hash{{a}} == hash{{b}} must imply a == b, but: a {a:?} ({}) != b {b:?} ({})",
                    u8::from(&a),
                    u8::from(&b)
                );
            }
        }
    }

    #[test]
    fn test_ipprotocol_hash_different() {
        for i in 0..u8::MAX {
            for a in [
                i.into(),
                TestIpProtocol::Unknown {
                    value: i,
                    _private: _test_ip_protocol_private::Sealed,
                },
            ] {
                for j in 0..=u8::MAX {
                    if i == j {
                        continue;
                    }
                    for b in [
                        j.into(),
                        TestIpProtocol::Unknown {
                            value: j,
                            _private: _test_ip_protocol_private::Sealed,
                        },
                    ] {
                        let mut hasher_a =
                            Box::new(std::collections::hash_map::DefaultHasher::new());
                        let mut hasher_b =
                            Box::new(std::collections::hash_map::DefaultHasher::new());
                        a.hash(&mut hasher_a);
                        b.hash(&mut hasher_b);
                        let ha = hasher_a.finish();
                        let hb = hasher_b.finish();
                        // a != b almost surely implies hash{{a}} != hash{{b}}.
                        assert_ne!(ha, hb, "Different values yielded same hashes: hash{{a {a:?} ({int_a})}} ({ha}) == hash{{b {b:?} ({int_b})}} ({hb})", int_a=u8::from(&a), ha=ha, int_b=u8::from(&b), hb=hb);
                    }
                }
            }
        }
    }
}
