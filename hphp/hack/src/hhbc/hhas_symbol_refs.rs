// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use std::collections::{BTreeMap, BTreeSet};

/// Data structure for keeping track of symbols (and includes) we encounter in
///the course of emitting bytecode for an AST. We split them into these four
/// categories for the sake of HHVM, which has lookup function corresponding to each.
#[derive(Clone, Debug)]
pub struct HhasSymbolRefs {
    pub includes: IncludePathSet,
    pub constants: SSet,
    pub functions: SSet,
    pub classes: SSet,
}

/// NOTE(hrust): order matters (hhbc_hhas write includes in sorted order)
pub type IncludePathSet = BTreeSet<IncludePath>;

type SSet = BTreeSet<String>;

#[derive(Clone, Debug)]
pub enum IncludePath {
    Absolute(String),                    // /foo/bar/baz.php
    SearchPathRelative(String),          // foo/bar/baz.php
    IncludeRootRelative(String, String), // $_SERVER['PHP_ROOT'] . "foo/bar/baz.php"
    DocRootRelative(String),
}
impl IncludePath {
    pub fn into_doc_root_relative(
        self,
        include_roots: Option<BTreeMap<String, String>>,
    ) -> IncludePath {
        if let IncludePath::IncludeRootRelative(var, lit) = &self {
            let include_roots = include_roots.unwrap_or_else(BTreeMap::new);
            use std::path::Path;
            match include_roots.get(var) {
                Some(prefix) => {
                    let path = Path::new(prefix).join(lit);
                    let relative = path.is_relative();
                    let path_str = path.to_str().expect("non UTF-8 path").to_owned();
                    return if relative {
                        IncludePath::DocRootRelative(path_str)
                    } else {
                        IncludePath::Absolute(path_str)
                    };
                }
                _ => panic!("missing var in include_roots: {}", var.to_string()),
            };
        }
        self
    }
}
