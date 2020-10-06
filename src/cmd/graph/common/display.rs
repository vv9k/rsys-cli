use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
};

type KvSpan<'a> = [Span<'a>; 2];

pub(crate) fn kv_span<'a, T: Into<String>>(k: T, v: T, color: Color, bold: bool) -> KvSpan<'a> {
    let val = if bold {
        Span::styled(v.into(), Style::default().fg(color).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(v.into(), Style::default().fg(color))
    };
    [Span::raw(k.into()), val]
}

pub(crate) fn spans_from<'a>(kvspans: Vec<KvSpan<'a>>) -> Spans<'a> {
    Spans::from(kvspans.concat())
}
