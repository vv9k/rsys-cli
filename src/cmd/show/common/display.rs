use std::borrow::Cow;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

type KvSpan<'a> = [Span<'a>; 2];

pub fn kv_span<'a, T: Into<String>>(k: T, v: T, color: Color, bold: bool) -> KvSpan<'a> {
    let val = if bold {
        Span::styled(v.into(), Style::default().fg(color).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(v.into(), Style::default().fg(color))
    };
    [Span::raw(k.into()), val]
}

pub fn spans_from<'a>(kvspans: Vec<KvSpan<'a>>) -> Spans<'a> {
    Spans::from(kvspans.concat())
}

pub fn err_tab<'a, T: Into<Cow<'a, str>>>(message: T) -> Paragraph<'a> {
    Paragraph::new(Span::styled(
        message.into(),
        Style::default().fg(Color::Red).add_modifier(Modifier::ITALIC),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Info")
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
}
