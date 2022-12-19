//! Types of lists:
//!
//! * inline raw: `a, b, c`, splits with `a` staying on first line
//!
//! * inline: `a, b, c`, splits with `a` on the next line, indented (comes after something
//!    else); where statements, generic + separated
//!
//! * bracketed: `(a, b, c)` - inline within brackets, comma after final element; [], <>,
//!    {} in use statements
//!
//! * curly bracketed: `{ a, b, c }` - inline within brackets with spaces
use proc_macro2::LineColumn;
use syn::{
    punctuated::Punctuated,
    Expr,
};
use crate::{
    Formattable,
    FormattablePunct,
    MakeSegsState,
    Alignment,
    SplitGroupBuilder,
    sg_general::{
        InlineListSuffix,
        append_comments,
    },
    SplitGroupIdx,
    new_sg,
};

pub(crate) fn append_inline_list_raw<
    E: Formattable,
    T: FormattablePunct,
    F: Formattable,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    sg: &mut SplitGroupBuilder,
    punct: &str,
    exprs: &Punctuated<E, T>,
    suffix: InlineListSuffix<F>,
) {
    let mut next_punct: Option<&T> = None;
    for (i, pair) in exprs.pairs().enumerate() {
        if i > 0 {
            if let Some(p) = next_punct.take() {
                append_comments(out, base_indent, sg, p.span_start());
                sg.seg(out, punct);
            }
            sg.split(out, base_indent.clone(), true);
            sg.seg_unsplit(out, " ");
        }
        sg.child(pair.value().make_segs(out, &base_indent));
        next_punct = pair.punct().map(|p| *p);
    }
    match suffix {
        InlineListSuffix::None => { },
        InlineListSuffix::Punct => {
            if let Some(p) = next_punct {
                append_comments(out, base_indent, sg, p.span_start());
                sg.seg_split(out, punct);
            }
        },
        InlineListSuffix::Extra(e) => {
            if let Some(p) = next_punct {
                append_comments(out, base_indent, sg, p.span_start());
                sg.seg(out, punct);
            } else if !exprs.is_empty() {
                sg.seg(out, punct);
            }
            if !exprs.is_empty() {
                sg.seg_unsplit(out, " ");
            }
            e.make_segs(out, base_indent);
        },
    }
}

pub(crate) fn append_inline_list<
    E: Formattable,
    T: FormattablePunct,
    F: Formattable,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    sg: &mut SplitGroupBuilder,
    punct: &str,
    exprs: &Punctuated<E, T>,
    suffix: InlineListSuffix<F>,
) {
    let indent = base_indent.indent();
    sg.split(out, indent.clone(), true);
    append_inline_list_raw(out, &indent, sg, punct, exprs, suffix);
}

pub(crate) fn append_bracketed_list<
    E: Formattable,
    T: FormattablePunct,
    F: Formattable,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    sg: &mut SplitGroupBuilder,
    prefix_start: LineColumn,
    prefix: &str,
    bracket_space: bool,
    exprs: &Punctuated<E, T>,
    list_suffix: InlineListSuffix<F>,
    suffix_start: LineColumn,
    suffix: &str,
) {
    append_comments(out, base_indent, sg, prefix_start);
    sg.seg(out, prefix);
    if bracket_space && !exprs.is_empty() {
        sg.seg_unsplit(out, " ");
    }
    let indent = base_indent.indent();
    sg.split(out, indent.clone(), true);
    append_inline_list_raw(out, &indent, sg, ",", exprs, list_suffix);
    if bracket_space && !exprs.is_empty() {
        sg.seg_unsplit(out, " ");
    }
    append_comments(out, &indent, sg, suffix_start);
    sg.split(out, base_indent.clone(), false);
    sg.seg(out, suffix);
}

pub(crate) fn append_bracketed_list_common<
    E: Formattable,
    T: FormattablePunct,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    sg: &mut SplitGroupBuilder,
    prefix_start: LineColumn,
    prefix: &str,
    exprs: &Punctuated<E, T>,
    suffix_start: LineColumn,
    suffix: &str,
) {
    append_bracketed_list(
        out,
        base_indent,
        sg,
        prefix_start,
        prefix,
        false,
        exprs,
        InlineListSuffix::<Expr>::None,
        suffix_start,
        suffix,
    );
}

pub(crate) fn append_bracketed_list_curly<
    E: Formattable,
    T: FormattablePunct,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    sg: &mut SplitGroupBuilder,
    prefix_start: LineColumn,
    exprs: &Punctuated<E, T>,
    extra: Option<impl Formattable>,
    suffix_start: LineColumn,
) {
    append_bracketed_list(out, base_indent, sg, prefix_start, " {", true, exprs, match extra {
        Some(e) => InlineListSuffix::Extra(e),
        None => InlineListSuffix::None,
    }, suffix_start, "}")
}

pub(crate) fn new_sg_bracketed_list<
    E: Formattable,
    T: FormattablePunct,
    F: Formattable,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    prefix_start: LineColumn,
    prefix: &str,
    bracket_space: bool,
    exprs: &Punctuated<E, T>,
    list_suffix: InlineListSuffix<F>,
    suffix_start: LineColumn,
    suffix: &str,
) -> SplitGroupIdx {
    let mut sg = new_sg(out);
    append_bracketed_list(
        out,
        base_indent,
        &mut sg,
        prefix_start,
        prefix,
        bracket_space,
        exprs,
        list_suffix,
        suffix_start,
        suffix,
    );
    sg.build(out)
}

pub(crate) fn new_sg_bracketed_list_common<
    E: Formattable,
    T: FormattablePunct,
>(
    out: &mut MakeSegsState,
    base_indent: &Alignment,
    prefix_start: LineColumn,
    prefix: &str,
    exprs: &Punctuated<E, T>,
    suffix_start: LineColumn,
    suffix: &str,
) -> SplitGroupIdx {
    new_sg_bracketed_list(
        out,
        base_indent,
        prefix_start,
        prefix,
        false,
        exprs,
        InlineListSuffix::<Expr>::None,
        suffix_start,
        suffix,
    )
}
