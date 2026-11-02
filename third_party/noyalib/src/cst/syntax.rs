// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Noyalib. All rights reserved.

//! Syntax-kind tags for green-tree nodes and tokens.
//!
//! Mirrors the design doc's enumeration. Token (leaf) kinds are
//! ordered before composite (non-leaf) kinds in the variant list to
//! make the leaf-vs-composite distinction visible at a glance.

/// Kind of a green-tree node or token.
///
/// Token kinds describe leaves whose `text` slice is the source
/// substring they came from. Composite kinds describe parent nodes
/// whose `text` is the concatenation of their descendants.
///
/// # Examples
///
/// ```
/// use noyalib::cst::SyntaxKind;
/// assert!(SyntaxKind::PlainScalar.is_token());
/// assert!(!SyntaxKind::Document.is_token());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SyntaxKind {
    // ── trivia (leaves) ─────────────────────────────────────────
    /// Run of inline blanks (spaces and tabs) between tokens.
    Whitespace,
    /// A single line break (`\n` or `\r\n`).
    Newline,
    /// A `# …` comment up to the next line break.
    Comment,
    /// A UTF-8 byte-order mark consumed at stream start.
    Bom,
    /// A `%YAML` / `%TAG` / reserved directive line, *including* the
    /// leading `%` and excluding the terminating line break.
    Directive,

    // ── structural punctuation (leaves) ─────────────────────────
    /// `-` block-sequence indicator.
    DashIndicator,
    /// `?` explicit-key indicator.
    QuestionIndicator,
    /// `:` value indicator.
    ColonIndicator,
    /// `,` flow-entry indicator.
    Comma,
    /// `[` flow-sequence open.
    OpenBracket,
    /// `]` flow-sequence close.
    CloseBracket,
    /// `{` flow-mapping open.
    OpenBrace,
    /// `}` flow-mapping close.
    CloseBrace,
    /// `&name` anchor mark — leaf includes the `&`.
    AnchorMark,
    /// `*name` alias mark — leaf includes the `*`.
    AliasMark,
    /// `!tag` tag mark — leaf includes the `!` and any handle / suffix.
    TagMark,
    /// `---` document-start indicator.
    DocStart,
    /// `...` document-end indicator.
    DocEnd,

    // ── scalar leaves ───────────────────────────────────────────
    /// A plain (unquoted) scalar.
    PlainScalar,
    /// A `'…'` single-quoted scalar.
    SingleQuotedScalar,
    /// A `"…"` double-quoted scalar.
    DoubleQuotedScalar,
    /// A `|…` literal block scalar (header + content).
    LiteralScalar,
    /// A `>…` folded block scalar (header + content).
    FoldedScalar,

    // ── composite (non-leaf) kinds ──────────────────────────────
    /// The whole stream — root of the green tree returned by
    /// [`crate::cst::parse_document`] and [`crate::cst::parse_stream`].
    Stream,
    /// A single YAML document inside the stream.
    Document,
    /// A block-style mapping. Children are [`Self::MappingEntry`]
    /// nodes (or trivia) in source order.
    BlockMapping,
    /// A block-style sequence. Children are [`Self::SequenceItem`]
    /// nodes (or trivia) in source order.
    BlockSequence,
    /// A `{ … }` flow mapping. Children are the brace tokens, the
    /// entries' tokens, and inter-token trivia. Flow content is
    /// kept flat — flow entries are not subdivided into
    /// [`Self::MappingEntry`] composites.
    FlowMapping,
    /// A `[ … ]` flow sequence. Flow content is kept flat — see
    /// [`Self::FlowMapping`].
    FlowSequence,
    /// A single key/value entry of a [`Self::BlockMapping`]. Holds
    /// the (optional) `?` indicator, the key tokens, the `:`
    /// indicator, the value tokens (which may themselves be a nested
    /// block / flow collection), and any inline trailing trivia.
    MappingEntry,
    /// A single item of a [`Self::BlockSequence`], including the `-`
    /// indicator and the value tokens (or a nested collection).
    SequenceItem,
}

impl SyntaxKind {
    /// `true` for leaves whose `text` is a verbatim source slice;
    /// `false` for composite parent nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use noyalib::cst::SyntaxKind;
    /// assert!(SyntaxKind::Whitespace.is_token());
    /// assert!(!SyntaxKind::Stream.is_token());
    /// assert!(!SyntaxKind::BlockMapping.is_token());
    /// ```
    #[must_use]
    pub const fn is_token(self) -> bool {
        !matches!(
            self,
            Self::Stream
                | Self::Document
                | Self::BlockMapping
                | Self::BlockSequence
                | Self::FlowMapping
                | Self::FlowSequence
                | Self::MappingEntry
                | Self::SequenceItem
        )
    }
}
