use super::{
    common::{single_widget_loop, StatefulWidget},
    events::Config,
};
use anyhow::Result;
use rsys::linux::ps::{processes, Process};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Row, Table},
    Frame,
};

const PS_HEADERS: &[&str] = &["pid", "name", "state", "vsize", "rss", "utime", "stime"];

pub struct ProcessMonitor {
    processes: Vec<Process>,
}

impl StatefulWidget for ProcessMonitor {
    fn update(&mut self) -> Result<()> {
        for process in &mut self.processes {
            process.stat.update()?;
        }
        Ok(())
    }
    fn render_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area);

        self.render_storage_info_widget(f, chunks[0]);
    }
}

impl ProcessMonitor {
    pub fn new() -> Result<ProcessMonitor> {
        Ok(ProcessMonitor {
            processes: processes()?,
        })
    }

    fn render_storage_info_widget<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let data = self.processes.iter().map(|s| {
            Row::StyledData(
                vec![
                    s.stat.pid.to_string(),
                    s.stat.name.to_string(),
                    s.stat.state.to_string(),
                    s.stat.vsize.to_string(),
                    s.stat.rss.to_string(),
                    s.stat.utime.to_string(),
                    s.stat.stime.to_string(),
                ]
                .into_iter(),
                Style::default(),
            )
        });

        let table = Table::new(PS_HEADERS.iter(), data).widths(&[
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
        ]);

        f.render_widget(table, area);
    }

    pub fn display_loop() -> Result<()> {
        let mut pmon = ProcessMonitor::new()?;
        single_widget_loop(&mut pmon, Config::default())
    }
}
