//! ASCII sequence diagram output
//!
//! Renders multi-lane sequence diagrams showing async execution flows
//! across execution domains (user space / kernel space / hardware).

use flowsight_knowledge::{AsyncTimeline, CallChain, ExecutionContext};

/// Calculate display width of a string (CJK chars = 2 columns, ASCII = 1)
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c.is_ascii() {
                1
            } else {
                // CJK and other wide characters
                2
            }
        })
        .sum()
}

/// Pad a string to a target display width with trailing spaces
fn pad_to_width(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target - w))
    }
}

/// Center a string within a target display width
fn center_in_width(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        let left = (target - w) / 2;
        let right = target - w - left;
        format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
    }
}

const COL_W: usize = 28;

/// A sequence diagram renderer
pub struct SequenceDiagram {
    labels: Vec<String>,
    lines: Vec<String>,
    num_lanes: usize,
}

impl SequenceDiagram {
    pub fn new(labels: &[&str]) -> Self {
        Self {
            labels: labels.iter().map(|s| s.to_string()).collect(),
            lines: Vec::new(),
            num_lanes: labels.len(),
        }
    }

    fn add_header(&mut self) {
        let mut line = String::new();
        for label in &self.labels {
            line.push_str(&center_in_width(label, COL_W));
        }
        self.lines.push(line);
    }

    fn add_lane_line(&mut self) {
        self.lines.push(self.make_lane_line());
    }

    fn make_lane_line(&self) -> String {
        let mut line = String::new();
        for _ in 0..self.num_lanes {
            line.push_str(&center_in_width("|", COL_W));
        }
        line
    }

    /// Place text right of lane's `|` marker
    fn add_action(&mut self, lane: usize, text: &str) {
        let mut line = String::new();
        for i in 0..self.num_lanes {
            if i == lane {
                let cell = format!("| {}", text);
                line.push_str(&pad_to_width(&cell, COL_W));
            } else {
                line.push_str(&center_in_width("|", COL_W));
            }
        }
        self.lines.push(line);
    }

    /// Draw arrow between two lanes
    fn add_arrow(&mut self, from: usize, to: usize) {
        if from == to || from >= self.num_lanes || to >= self.num_lanes {
            return;
        }

        let mut line = String::new();
        let going_right = to > from;
        let min_l = from.min(to);
        let max_l = from.max(to);

        for i in 0..self.num_lanes {
            if i < min_l || i > max_l {
                // Outside arrow range
                line.push_str(&center_in_width("|", COL_W));
            } else if i == from && going_right {
                // Start: |--------
                let mut cell = String::from("|");
                for _ in 0..(COL_W - 1) {
                    cell.push('-');
                }
                line.push_str(&cell);
            } else if i == to && going_right {
                // End: -------->|
                let mut cell = String::new();
                for _ in 0..(COL_W - 2) {
                    cell.push('-');
                }
                cell.push_str(">|");
                line.push_str(&cell);
            } else if i == from && !going_right {
                // Start going left: just |
                let mut cell = String::new();
                for _ in 0..(COL_W - 2) {
                    cell.push('-');
                }
                cell.push_str("-|");
                line.push_str(&cell);
            } else if i == to && !going_right {
                // End going left: |<-------
                let mut cell = String::from("|<");
                for _ in 0..(COL_W - 2) {
                    cell.push('-');
                }
                line.push_str(&cell);
            } else {
                // Middle of arrow
                let mut cell = String::new();
                for _ in 0..COL_W {
                    cell.push('-');
                }
                line.push_str(&cell);
            }
        }
        self.lines.push(line);
    }

    fn add_note(&mut self, text: &str) {
        let total = COL_W * self.num_lanes;
        let note = format!("... {} ...", text);
        self.lines.push(center_in_width(&note, total));
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

/// Render a timeline-based async sequence diagram
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

    // Separation
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
    // Determine trigger lane from first node context
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

        let marker = if node.is_user_entry {
            " <-- YOUR CODE"
        } else {
            ""
        };
        d.add_action(lane, &format!("{}(){}", node.function, marker));

        if let Some(ref desc) = node.description {
            d.add_action(lane, &format!("  // {}", desc));
        }

        prev_lane = Some(lane);
    }
}

/// Render a simple call chain as a sequence diagram
pub fn print_chain_sequence(chain: &CallChain) {
    let mut d = SequenceDiagram::new(&["User Space", "Kernel", "Hardware"]);

    d.add_header();
    d.add_lane_line();
    d.add_lane_line();

    d.add_action(1, &format!("<<< {} >>>", chain.name));
    d.add_lane_line();

    // Determine trigger lane
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

        let marker = if node.is_user_entry {
            " <-- YOUR CODE"
        } else {
            ""
        };
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
