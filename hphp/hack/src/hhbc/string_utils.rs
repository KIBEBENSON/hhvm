// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

extern crate lazy_static;

use escaper::*;
use lazy_static::lazy_static;
use naming_special_names_rust::{classes, members};
use regex::Regex;
use std::borrow::Cow;
use std::cell::Cell;

lazy_static! {
    static ref NS_RE: Regex = Regex::new(r".*\\").unwrap();
    static ref TYPE_RE: Regex = Regex::new(r"<.*>").unwrap();
}

#[derive(Clone)]
pub struct GetName {
    string: Vec<u8>,

    unescape: fn(String) -> String,
}

impl GetName {
    pub fn new(string: Vec<u8>, unescape: fn(String) -> String) -> GetName {
        GetName { string, unescape }
    }

    pub fn get(&self) -> &Vec<u8> {
        &self.string
    }
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.string).to_string()
    }
    pub fn to_unescaped_string(&self) -> String {
        let unescape = self.unescape;
        unescape(self.to_string())
    }
}

impl std::fmt::Debug for GetName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "GetName {{ string: {}, unescape:? }}", self.to_string())
    }
}

thread_local!(static MANGLE_XHP_MODE: Cell<bool> = Cell::new(true));

pub fn without_xhp_mangling<T>(f: impl FnOnce() -> T) -> T {
    MANGLE_XHP_MODE.with(|cur| {
        let old = cur.replace(false);
        let ret = f();
        cur.set(old); // use old instead of true to support nested calls in the same thread
        ret
    })
}

pub fn is_xhp(name: &str) -> bool {
    name.chars().next().map_or(false, |c| c == ':')
}

pub fn mangle_xhp_id(mut name: String) -> String {
    fn ignore_id(name: &str) -> bool {
        name.starts_with("class@anonymous") || name.starts_with("Closure$")
    }

    if !ignore_id(&name) && MANGLE_XHP_MODE.with(|x| x.get()) {
        if is_xhp(&name) {
            name.replace_range(..1, "xhp_")
        }
        name.replace(":", "__").replace("-", "_")
    } else {
        name
    }
}

pub fn quote_string(s: &str) -> String {
    format!("\\{}\\", escape(s))
}

pub fn quote_string_with_escape(s: &str) -> String {
    format!("\\\"{}\\\"", escape(s))
}

pub fn single_quote_string_with_escape(s: &str) -> String {
    format!("'{}'", escape(s))
}

pub fn triple_quote_string(s: &str) -> String {
    format!("\"\"\"{}\"\"\"", escape(s))
}

pub fn prefix_namespace(n: &str, s: &str) -> String {
    format!("{}\\{}", n, s)
}

pub fn strip_global_ns(s: &str) -> &str {
    s.trim_start_matches("\\")
}

pub fn strip_ns(s: &str) -> Cow<str> {
    NS_RE.replace(&s, "")
}

pub fn has_ns(s: &str) -> bool {
    NS_RE.is_match(s)
}

pub fn strip_type_list(s: &str) -> Cow<str> {
    TYPE_RE.replace_all(&s, "")
}

pub fn cmp(s1: &str, s2: &str, case_sensitive: bool, ignore_ns: bool) -> bool {
    fn canon(s: &str, ignore_ns: bool) -> Cow<str> {
        let mut cow = Cow::Borrowed(s);
        if ignore_ns {
            *cow.to_mut() = strip_ns(&cow).into_owned();
        }

        cow
    }

    let s1 = canon(s1, ignore_ns);
    let s2 = canon(s2, ignore_ns);

    if case_sensitive {
        s1 == s2
    } else {
        s1.eq_ignore_ascii_case(&s2)
    }
}

pub fn is_self(s: &str) -> bool {
    s.eq_ignore_ascii_case(classes::SELF)
}

pub fn is_parent(s: &str) -> bool {
    s.eq_ignore_ascii_case(classes::PARENT)
}

pub fn is_static(s: &str) -> bool {
    s.eq_ignore_ascii_case(classes::STATIC)
}

pub fn is_class(s: &str) -> bool {
    s.eq_ignore_ascii_case(members::M_CLASS)
}

pub fn mangle_meth_caller(mangled_cls_name: &str, f_name: &str) -> String {
    format!("\\MethCaller${}${}", mangled_cls_name, f_name)
}

pub mod types {
    pub fn fix_casing(s: &str) -> &str {
        match s.to_lowercase().as_str() {
            "vector" => "Vector",
            "immvector" => "ImmVector",
            "set" => "Set",
            "immset" => "ImmSet",
            "map" => "Map",
            "immmap" => "ImmMap",
            "pair" => "Pair",
            _ => s,
        }
    }
}

/* Integers are represented as strings */
pub mod integer {
    pub fn to_decimal(s: &str) -> Result<String, std::num::ParseIntError> {
        /* Don't accidentally convert 0 to 0o */
        let num = if s.len() > 1 && s.as_bytes()[0] == b'0' {
            match s.as_bytes()[1] {
                /* Binary */
                b'b' | b'B' => i64::from_str_radix(&s[2..], 2),
                /* Hex */
                b'x' | b'X' => i64::from_str_radix(&s[2..], 16),
                /* Octal */
                _ => i64::from_str_radix(s, 8),
            }
        } else {
            i64::from_str_radix(s, 10)
        }
        .map(|n| n.to_string());

        num
    }
}

pub mod locals {
    pub fn strip_dollar(s: &str) -> &str {
        if s.len() > 0 && s.as_bytes()[0] == b'$' {
            &s[1..]
        } else {
            s
        }
    }
}

#[cfg(test)]
mod string_utils_tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn quote_string_test() {
        let some_string = "test";
        assert_eq!(super::quote_string(&some_string), "\\test\\");
    }

    #[test]
    fn quote_string_with_escape_test() {
        let some_string = "test";
        assert_eq!(
            super::quote_string_with_escape(&some_string),
            "\\\"test\\\""
        );
    }

    #[test]
    fn single_quote_string_with_escape_test() {
        let some_string = "test";
        assert_eq!(
            super::single_quote_string_with_escape(&some_string),
            "'test'"
        );
    }

    #[test]
    fn triple_quote_string_test() {
        let some_string = "test";
        assert_eq!(super::triple_quote_string(&some_string), "\"\"\"test\"\"\"");
    }

    #[test]
    fn prefix_namespace_test() {
        let namespace = "ns";
        let some_string = "test";
        assert_eq!(
            super::prefix_namespace(&namespace, &some_string),
            "ns\\test"
        );
    }

    #[test]
    fn strip_global_ns_test() {
        let some_string = "\\test";
        assert_eq!(super::strip_global_ns(&some_string), "test");
    }

    #[test]
    fn strip_ns_test() {
        let with_ns = "ns1\\test";
        let without_ns = "test";
        assert_eq!(super::strip_ns(&with_ns), "test");
        assert_eq!(super::strip_ns(&without_ns), without_ns);
    }

    #[test]
    fn has_ns_test() {
        let with_ns = "\\test";
        let without_ns = "test";
        assert_eq!(super::has_ns(&with_ns), true);
        assert_eq!(super::has_ns(&without_ns), false);
    }

    #[test]
    fn strip_type_list_test() {
        let s = "MutableMap<Tk, Tv>";
        assert_eq!(super::strip_type_list(&s).into_owned(), "MutableMap");
    }

    #[test]
    fn cmp_test() {
        let s1 = "ns1\\s1";
        let s1_uppercase = "NS1\\S1";

        let ns2_s1 = "ns2\\s1";
        let ns2_s1_uppercase = "NS2\\S1";

        let ns2_s2 = "ns2\\s2";

        assert_eq!(true, super::cmp(&s1, &s1_uppercase, false, false));
        assert_eq!(false, super::cmp(&s1, &s1_uppercase, true, false));
        assert_eq!(true, super::cmp(&s1, &s1_uppercase, false, true));
        assert_eq!(false, super::cmp(&s1, &s1_uppercase, true, true));

        assert_eq!(false, super::cmp(&s1, &ns2_s1, false, false));
        assert_eq!(false, super::cmp(&s1, &ns2_s1, true, false));
        assert_eq!(true, super::cmp(&s1, &ns2_s1, false, true));
        assert_eq!(true, super::cmp(&s1, &ns2_s1, true, true));

        assert_eq!(false, super::cmp(&s1, &ns2_s1_uppercase, false, false));
        assert_eq!(false, super::cmp(&s1, &ns2_s1_uppercase, true, false));
        assert_eq!(true, super::cmp(&s1, &ns2_s1_uppercase, false, true));
        assert_eq!(false, super::cmp(&s1, &ns2_s1_uppercase, true, true));

        assert_eq!(false, super::cmp(&s1, &ns2_s2, false, false));
        assert_eq!(false, super::cmp(&s1, &ns2_s2, true, false));
        assert_eq!(false, super::cmp(&s1, &ns2_s2, false, true));
        assert_eq!(false, super::cmp(&s1, &ns2_s2, true, true));
    }

    #[test]
    fn is_self_test() {
        let s1 = "self";
        let s2 = "not_self";

        assert_eq!(super::is_self(&s1), true);
        assert_eq!(super::is_self(&s2), false);
    }

    #[test]
    fn is_parent_test() {
        let s1 = "parent";
        let s2 = "not_parent";

        assert_eq!(super::is_parent(&s1), true);
        assert_eq!(super::is_parent(&s2), false);
    }

    #[test]
    fn is_static_test() {
        let s1 = "static";
        let s2 = "not_static";

        assert_eq!(super::is_static(&s1), true);
        assert_eq!(super::is_static(&s2), false);
    }

    #[test]
    fn is_class_test() {
        let s1 = "class";
        let s2 = "not_a_class";

        assert_eq!(super::is_class(&s1), true);
        assert_eq!(super::is_class(&s2), false);
    }

    #[test]
    fn mangle_meth_caller_test() {
        let cls = "SomeClass";
        let f = "some_function";

        assert_eq!(
            super::mangle_meth_caller(cls, f),
            "\\MethCaller$SomeClass$some_function"
        );
    }

    mod types {
        mod fix_casing {

            macro_rules! test_case {
                ($name: ident, $input: expr, $expected: expr) => {
                    #[test]
                    fn $name() {
                        assert_eq!(crate::types::fix_casing($input), $expected);
                    }
                };
            }

            test_case!(lowercase_vector, "vector", "Vector");
            test_case!(mixedcase_vector, "vEcTor", "Vector");
            test_case!(uppercase_vector, "VECTOR", "Vector");

            test_case!(lowercase_immvector, "immvector", "ImmVector");
            test_case!(mixedcase_immvector, "immvEcTor", "ImmVector");
            test_case!(uppercase_immvector, "IMMVECTOR", "ImmVector");

            test_case!(lowercase_set, "set", "Set");
            test_case!(mixedcase_set, "SeT", "Set");
            test_case!(uppercase_set, "SET", "Set");

            test_case!(lowercase_immset, "immset", "ImmSet");
            test_case!(mixedcase_immset, "ImMSeT", "ImmSet");
            test_case!(uppercase_immset, "IMMSET", "ImmSet");

            test_case!(lowercase_map, "map", "Map");
            test_case!(mixedcase_map, "MaP", "Map");
            test_case!(uppercase_map, "MAP", "Map");

            test_case!(lowercase_immmap, "immmap", "ImmMap");
            test_case!(mixedcase_immmap, "immMaP", "ImmMap");
            test_case!(uppercase_immmap, "IMMMAP", "ImmMap");

            test_case!(lowercase_pair, "pair", "Pair");
            test_case!(mixedcase_pair, "pAiR", "Pair");
            test_case!(uppercase_pair, "PAIR", "Pair");

            test_case!(
                non_hack_collection_returns_original_string,
                "SomeStRinG",
                "SomeStRinG"
            );
            test_case!(
                hack_collection_with_leading_whitespace_returns_original_string,
                " pair",
                " pair"
            );
            test_case!(
                hack_collection_with_trailing_whitespace_returns_original_string,
                "pair ",
                "pair "
            );

        }
    }

    mod integer {
        mod to_decimal {
            use crate::integer::*;

            #[test]
            fn decimal_zero() {
                assert_eq!(to_decimal("0"), Ok("0".to_string()));
            }

            #[test]
            fn octal_zero() {
                assert_eq!(to_decimal("00"), Ok("0".to_string()));
            }

            #[test]
            fn binary_zero_lowercase() {
                assert_eq!(to_decimal("0b0"), Ok("0".to_string()));
            }

            #[test]
            fn binary_zero_uppercase() {
                assert_eq!(to_decimal("0B0"), Ok("0".to_string()));
            }

            #[test]
            fn hex_zero_lowercase() {
                assert_eq!(to_decimal("0x0"), Ok("0".to_string()));
            }

            #[test]
            fn hex_zero_uppercase() {
                assert_eq!(to_decimal("0X0"), Ok("0".to_string()));
            }

            #[test]
            fn decimal_random_value() {
                assert_eq!(to_decimal("1245"), Ok("1245".to_string()));
            }

            #[test]
            fn octal_random_value() {
                assert_eq!(to_decimal("02335"), Ok("1245".to_string()));
            }

            #[test]
            fn binary_random_value_lowercase() {
                assert_eq!(to_decimal("0b10011011101"), Ok("1245".to_string()));
            }

            #[test]
            fn binary_random_value_uppercase() {
                assert_eq!(to_decimal("0B10011011101"), Ok("1245".to_string()));
            }

            #[test]
            fn hex_random_value_lowercase() {
                assert_eq!(to_decimal("0x4DD"), Ok("1245".to_string()));
            }

            #[test]
            fn hex_random_value_uppercase() {
                assert_eq!(to_decimal("0X4DD"), Ok("1245".to_string()));
            }

            #[test]
            fn decimal_max_value() {
                assert_eq!(
                    to_decimal("9223372036854775807"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn octal_max_value() {
                assert_eq!(
                    to_decimal("0777777777777777777777"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn binary_max_value_lowercase() {
                assert_eq!(
                    to_decimal("0b111111111111111111111111111111111111111111111111111111111111111"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn binary_max_value_uppercase() {
                assert_eq!(
                    to_decimal("0B111111111111111111111111111111111111111111111111111111111111111"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn hex_max_value_lowercase() {
                assert_eq!(
                    to_decimal("0x7FFFFFFFFFFFFFFF"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn hex_max_value_uppercase() {
                assert_eq!(
                    to_decimal("0X7FFFFFFFFFFFFFFF"),
                    Ok("9223372036854775807".to_string())
                );
            }

            #[test]
            fn unparsable_string() {
                assert!(to_decimal("bad_string").is_err());
            }
        }
    }

    mod locals {
        use crate::locals::*;

        #[test]
        fn strip_single_leading_dollar() {
            assert_eq!(strip_dollar("$foo"), "foo");
        }

        #[test]
        fn return_string_if_no_leading_dollar() {
            assert_eq!(strip_dollar("foo"), "foo");
        }

        #[test]
        fn empty_string() {
            assert_eq!(strip_dollar(""), "");
        }

        #[test]
        fn string_of_single_dollar() {
            assert_eq!(strip_dollar("$"), "");
        }
    }
}
