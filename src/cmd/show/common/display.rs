use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
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

pub fn popup<'s, B: Backend>(f: &mut Frame<B>, message: Span<'s>, title: &str, border_style: Style, button: Span<'s>) {
    let area = centered_rect(60, 20, f.size());
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Max(10)])
        .split(area);

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let text = Paragraph::new(vec![
        Spans::from(message),
        Spans::from(Span::raw("")),
        Spans::from(button),
    ])
    .block(popup_block)
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(Clear, layout[0]);
    f.render_widget(text, layout[0]);
}

pub fn err_popup<B: Backend>(f: &mut Frame<B>, error: &str, button: &str) {
    popup(
        f,
        Span::styled(error, Style::default().add_modifier(Modifier::ITALIC)),
        "ERROR",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Span::raw(button),
    )
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
