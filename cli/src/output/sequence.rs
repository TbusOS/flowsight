//! ASCII sequence diagram output
//!
//! Renders multi-lane sequence diagrams showing async execution flows
//! across execution domains (user space / kernel space / hardware).

use flowsight_knowledge::{AsyncTimeline, CallChain, ExecutionContext};

/// Calculate display width (CJK = 2 columns, ASCII = 1)
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| if c.is_ascii() { 1 } else { 2 })
        .sum()
}

/// Pad string to exact display width with trailing spaces
fn pad_to(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target - w))
    }
}

/// Truncate string to fit within target display width
fn fit_to(s: &str, target: usize) -> String {
    let mut result = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = if c.is_ascii() { 1 } else { 2 };
        if w + cw > target {
            break;
        }
        result.push(c);
        w += cw;
    }
    // Pad remaining
    if w < target {
        result.push_str(&" ".repeat(target - w));
    }
    result
}

/// Center string within target display width
fn center(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        let left = (target - w) / 2;
        let right = target - w - left;
        format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
    }
}

const COL: usize = 36;

pub struct SequenceDiagram {
    labels: Vec<String>,
    lines: Vec<String>,
    n: usize,
}

impl SequenceDiagram {
    pub fn new(labels: &[&str]) -> Self {
        Self {
            labels: labels.iter().map(|s| s.to_string()).collect(),
            lines: Vec::new(),
            n: labels.len(),
        }
    }

    fn add_header(&mut self) {
        let mut line = String::new();
        for label in &self.labels {
            line.push_str(&center(label, COL));
        }
        self.lines.push(line);
    }

    fn add_lane_line(&mut self) {
        self.lines.push(self.make_pipes());
    }

    fn make_pipes(&self) -> String {
        let mut line = String::new();
        for i in 0..self.n {
            if i == self.n - 1 {
                // Last lane: just centered pipe, no need to pad fully
                line.push_str(&center("|", COL));
            } else {
                line.push_str(&center("|", COL));
            }
        }
        line
    }

    /// Make a cell that fits exactly COL display width.
    /// For non-last lanes, truncate if too wide.
    /// For the last lane, allow overflow.
    fn make_cell(&self, lane: usize, content: &str) -> String {
        let w = display_width(content);
        if lane < self.n - 1 {
            // Non-last lane: must be exactly COL wide
            if w <= COL {
                pad_to(content, COL)
            } else {
                fit_to(content, COL)
            }
        } else {
            // Last lane: pad if short, allow overflow
            if w < COL {
                pad_to(content, COL)
            } else {
                content.to_string()
            }
        }
    }

    fn add_action(&mut self, lane: usize, text: &str) {
        let mut line = String::new();
        for i in 0..self.n {
            if i == lane {
                let cell = format!("| {}", text);
                line.push_str(&self.make_cell(i, &cell));
            } else {
                line.push_str(&center("|", COL));
            }
        }
        self.lines.push(line);
    }

    fn add_arrow(&mut self, from: usize, to: usize) {
        if from == to || from >= self.n || to >= self.n {
            return;
        }

        let mut line = String::new();
        let going_right = to > from;
        let min_l = from.min(to);
        let max_l = from.max(to);

        for i in 0..self.n {
            if i < min_l || i > max_l {
                line.push_str(&center("|", COL));
            } else if i == from && going_right {
                let mut cell = String::from("|");
                for _ in 0..(COL - 1) {
                    cell.push('-');
                }
                line.push_str(&cell);
            } else if i == to && going_right {
                let mut cell = String::new();
                for _ in 0..(COL - 2) {
                    cell.push('-');
                }
                cell.push_str(">|");
                line.push_str(&cell);
            } else if i == from && !going_right {
                let mut cell = String::new();
                for _ in 0..(COL - 1) {
                    cell.push('-');
                }
                cell.push('|');
                line.push_str(&cell);
            } else if i == to && !going_right {
                let mut cell = String::from("|<");
                for _ in 0..(COL - 2) {
                    cell.push('-');
                }
                line.push_str(&cell);
            } else {
                let mut cell = String::new();
                for _ in 0..COL {
                    cell.push('-');
                }
                line.push_str(&cell);
            }
        }
        self.lines.push(line);
    }

    fn add_note(&mut self, text: &str) {
        let total = COL * self.n;
        let note = format!("... {} ...", text);
        self.lines.push(center(&note, total));
    }

    fn add_section(&mut self, lane: usize, text: &str) {
        self.add_lane_line();
        self.add_action(lane, &format!("[{}]", text));
        self.add_lane_line();
    }

    fn render(&self) {
        for line in &self.lines {
            println!("{}", line.trim_end());
        }
    }
}

fn context_to_lane(ctx: &ExecutionContext) -> usize {
    match ctx {
        ExecutionContext::User => 0,
        ExecutionContext::Process => 1,
        ExecutionContext::SoftIrq => 1,
        ExecutionContext::HardIrq => 2,
        _ => 1,
    }
}

pub fn print_async_sequence(timeline: &AsyncTimeline, name: &str) {
    let mut d = SequenceDiagram::new(&["User Space", "Kernel", "Hardware"]);

    d.add_header();
    d.add_lane_line();
    d.add_lane_line();

    d.add_action(1, &format!("<<< {} >>>", name));
    d.add_lane_line();

    // Phase 1
    let p1_lane = context_to_lane(&timeline.phase1.context);
    d.add_action(p1_lane, &format!("=== {} ===", timeline.phase1.name));
    render_chain_nodes(&mut d, &timeline.phase1.call_chain);

    d.add_lane_line();
    d.add_note(&timeline.separation);
    d.add_lane_line();

    // Phase 2
    let p2_lane = context_to_lane(&timeline.phase2.context);
    d.add_action(p2_lane, &format!("=== {} ===", timeline.phase2.name));
    render_chain_nodes(&mut d, &timeline.phase2.call_chain);

    d.add_lane_line();
    d.add_lane_line();

    d.render();
}

fn render_chain_nodes(d: &mut SequenceDiagram, chain: &CallChain) {
    let trigger_lane = chain
        .nodes
        .first()
        .map(|n| context_to_lane(&n.context))
        .unwrap_or(1);
    d.add_section(trigger_lane, &chain.trigger_source);

    let mut prev_lane: Option<usize> = None;

    for node in &chain.nodes {
        let lane = context_to_lane(&node.context);

        if let Some(pl) = prev_lane {
            if pl != lane {
                d.add_arrow(pl, lane);
            }
        }

        let marker = if node.is_user_entry { " <<< YOU" } else { "" };
        d.add_action(lane, &format!("{}(){}", node.function, marker));

        if let Some(ref desc) = node.description {
            d.add_action(lane, &format!("  // {}", desc));
        }

        prev_lane = Some(lane);
    }
}

pub fn print_chain_sequence(chain: &CallChain) {
    let mut d = SequenceDiagram::new(&["User Space", "Kernel", "Hardware"]);

    d.add_header();
    d.add_lane_line();
    d.add_lane_line();

    d.add_action(1, &format!("<<< {} >>>", chain.name));
    d.add_lane_line();

    let trigger_lane = chain
        .nodes
        .first()
        .map(|n| context_to_lane(&n.context))
        .unwrap_or(1);
    d.add_section(trigger_lane, &chain.trigger_source);

    let mut prev_lane: Option<usize> = None;

    for node in &chain.nodes {
        let lane = context_to_lane(&node.context);

        if let Some(pl) = prev_lane {
            if pl != lane {
                d.add_arrow(pl, lane);
            }
        }

        let marker = if node.is_user_entry { " <<< YOU" } else { "" };
        d.add_action(lane, &format!("{}(){}", node.function, marker));

        if let Some(ref desc) = node.description {
            d.add_action(lane, &format!("  // {}", desc));
        }

        if let Some(ref file) = node.file {
            d.add_action(lane, &format!("  // {}", file));
        }

        prev_lane = Some(lane);
    }

    d.add_lane_line();
    d.add_lane_line();

    d.render();
}
