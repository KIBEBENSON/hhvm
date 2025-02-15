// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use oxidized::{i_set::ISet, pos::Pos, prim_defs::Comment, scoured_comments::ScouredComments};
use parser_core_types::{
    indexed_source_text::IndexedSourceText,
    lexable_token::LexablePositionedToken,
    lexable_trivia::{LexablePositionedTrivia, LexableTrivia},
    positioned_syntax::PositionedSyntaxTrait,
    source_text::SourceText,
    syntax::SyntaxVariant::*,
    syntax::*,
    trivia_kind::TriviaKind,
};
use regex::bytes::Regex;

/** The full fidelity parser considers all comments "simply" trivia. Some
 * comments have meaning, though. This meaning can either be relevant for the
 * type checker (like HH_FIXME, etc.), but also for other uses, like
 * Codex, where comments are used for documentation generation.
 *
 * Inlining the scrape for comments in the lowering code would be prohibitively
 * complicated, but a separate pass is fine.
 */
pub struct ScourComment<'a, T, V> {
    pub indexed_source_text: &'a IndexedSourceText<'a>,
    pub collect_fixmes: bool,
    pub incldue_line_comments: bool,
    pub ignored_fixme: Option<&'a Regex>,
    pub disallowed_decl_fixmes: &'a ISet,
    pub phantom: std::marker::PhantomData<(*const T, *const V)>,
}

impl<'src, T, V> ScourComment<'src, T, V>
where
    T: LexablePositionedToken<'src>,
    T::Trivia: LexablePositionedTrivia,
    V: SyntaxValueType<T>,
    Syntax<T, V>: PositionedSyntaxTrait,
{
    pub fn scour_comments(&self, node: &Syntax<T, V>) -> ScouredComments {
        let mut r = self.scour_comments_(node, false, ScouredComments::new());
        r.comments.reverse();
        r
    }

    fn scour_comments_(
        &self,
        node: &Syntax<T, V>,
        in_block: bool,
        acc: ScouredComments,
    ) -> ScouredComments {
        let on_children = |in_block: bool, acc| {
            node.iter_children()
                .fold(acc, |a, n| self.scour_comments_(n, true, a))
        };
        match &node.syntax {
            CompoundStatement(_) => on_children(true, acc),
            Token(t) => {
                if t.has_trivia_kind(TriviaKind::DelimitedComment)
                    || (self.incldue_line_comments
                        && t.has_trivia_kind(TriviaKind::SingleLineComment))
                    || (self.collect_fixmes
                        && (t.has_trivia_kind(TriviaKind::FixMe)
                            || t.has_trivia_kind(TriviaKind::IgnoreError)))
                {
                    t.leading()
                        .iter()
                        .chain(t.trailing().iter())
                        .fold(acc, |a, t| self.on_trivia(in_block, node, t, a))
                } else {
                    on_children(in_block, acc)
                }
            }
            _ => on_children(in_block, acc),
        }
    }

    fn on_trivia(
        &self,
        in_block: bool,
        node: &Syntax<T, V>,
        t: &T::Trivia,
        mut acc: ScouredComments,
    ) -> ScouredComments {
        use TriviaKind::*;
        match t.kind() {
            WhiteSpace | EndOfLine | FallThrough | ExtraTokenError | AfterHaltCompiler => acc,
            DelimitedComment => {
                let start = t.start_offset() + 2;
                let end = t.end_offset();
                let len = end - start - 1;
                let p = self.pos_of_offset(end - 1, end);
                let text = self.source_text().sub_as_str(start, len).to_string();
                acc.comments.push((p, Comment::CmtBlock(text)));
                acc
            }
            SingleLineComment => {
                if !self.incldue_line_comments {
                    acc
                } else {
                    let text = self.source_text().text();
                    let start = t.start_offset();
                    let start = start + if text[start] == b'#' { 1 } else { 2 };
                    let end = t.end_offset();
                    let len = end - start + 1;
                    let p = self.pos_of_offset(start, end);
                    let mut text = self.source_text().sub_as_str(start, len).to_string();
                    text.push('\n');
                    acc.comments.push((p, Comment::CmtLine(text)));
                    acc
                }
            }
            FixMe | IgnoreError => {
                lazy_static! {
                    static ref IGNORE_ERROR: Regex =
                        Regex::new(r#"HH_(?:FIXME|IGNORE_ERROR)[ \t\n]*\[([0-9]+)\]"#).unwrap();
                }

                if !self.collect_fixmes {
                    acc
                } else {
                    let text = t.text_raw(self.source_text());
                    let ignore_fixme = match self.ignored_fixme {
                        None => false,
                        Some(r) => r.is_match(text),
                    };
                    if ignore_fixme {
                        acc
                    } else {
                        let pos = self.p_pos(node);
                        let line = pos.line() as isize;
                        let p = self.pos_of_offset(t.start_offset(), t.end_offset());
                        match IGNORE_ERROR
                            .captures(text)
                            .and_then(|c| c.get(1))
                            .map(|m| m.as_bytes())
                        {
                            Some(code) => {
                                let code = std::str::from_utf8(code).unwrap();
                                let code: isize = std::str::FromStr::from_str(code).unwrap();
                                if !in_block && self.disallowed_decl_fixmes.contains(&code) {
                                    acc.add_to_misuses(line, code, p);
                                    acc
                                } else {
                                    acc.add_to_fixmes(line, code, p);
                                    acc
                                }
                            }
                            None => {
                                // Errors.fixme_format pos;
                                acc.add_format_error(pos);
                                acc
                            }
                        }
                    }
                }
            }
        }
    }

    fn source_text(&self) -> &SourceText {
        self.indexed_source_text.source_text()
    }

    fn p_pos(&self, node: &Syntax<T, V>) -> Pos {
        node.position_exclusive(self.indexed_source_text)
            .unwrap_or_else(Pos::make_none)
    }

    fn pos_of_offset(&self, start: usize, end: usize) -> Pos {
        self.indexed_source_text.relative_pos(start, end)
    }
}
