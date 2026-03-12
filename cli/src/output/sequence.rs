//! ASCII sequence diagram output
//!
//! Renders multi-lane sequence diagrams showing async execution flows
//! across execution domains (user space / kernel space / hardware).

use flowsight_knowledge::{AsyncTimeline, CallChain, ExecutionContext};

/// Lane configuration
struct Lane {
    label: String,
    col: usize,
}

const COL_WIDTH: usize = 26;

/// A sequence diagram renderer
pub struct SequenceDiagram {
    lanes: Vec<Lane>,
    lines: Vec<String>,
}

impl SequenceDiagram {
    /// Create a new sequence diagram with the given lane labels
    pub fn new(labels: &[&str]) -> Self {
        let lanes: Vec<Lane> = labels
            .iter()
            .enumerate()
            .map(|(i, label)| Lane {
                label: label.to_string(),
                col: i,
            })
            .collect();

        Self {
            lanes,
            lines: Vec::new(),
        }
    }

    /// Add the header line with lane labels
    fn add_header(&mut self) {
        let mut parts: Vec<String> = Vec::new();
        for lane in &self.lanes {
            parts.push(format!("{:^width$}", lane.label, width = COL_WIDTH));
        }
        self.lines.push(parts.join(""));
    }

    /// Add vertical lane markers
    fn add_separator(&mut self) {
        self.lines.push(self.make_lane_line());
    }

    fn add_blank(&mut self) {
        self.add_separator();
    }

    /// Add an action label next to a lane's vertical bar
    fn add_action(&mut self, lane_idx: usize, text: &str) {
        let mut parts: Vec<String> = Vec::new();
        for (i, _) in self.lanes.iter().enumerate() {
            if i == lane_idx {
                // Put text right of the bar
                let content = format!("| {}", text);
                parts.push(format!("{:<width$}", content, width = COL_WIDTH));
            } else {
                parts.push(format!("{:^width$}", "|", width = COL_WIDTH));
            }
        }
        self.lines.push(parts.join(""));
    }

    /// Add an arrow from one lane to another
    fn add_arrow(&mut self, from: usize, to: usize, label: &str) {
        let mut parts: Vec<String> = Vec::new();
        let min_lane = from.min(to);
        let max_lane = from.max(to);
        let going_right = to > from;

        for (i, _) in self.lanes.iter().enumerate() {
            if i == min_lane && i == max_lane {
                // Same lane, no arrow needed
                parts.push(format!("{:^width$}", "|", width = COL_WIDTH));
            } else if i == from && going_right {
                // Start of right arrow
                let mut s = String::from("|");
                for _ in 0..(COL_WIDTH - 1) {
                    s.push('-');
                }
                parts.push(s);
            } else if i == from && !going_right {
                // Start of left arrow (arrow goes left from here)
                let mut s = String::from("|");
                for _ in 0..(COL_WIDTH - 1) {
                    s.push(' ');
                }
                parts.push(s);
            } else if i == to && going_right {
                // End of right arrow
                let mut s = String::new();
                for _ in 0..(COL_WIDTH - 2) {
                    s.push('-');
                }
                s.push_str(">|");
                parts.push(s);
            } else if i == to && !going_right {
                // End of left arrow
                let mut s = String::from("|<");
                for _ in 0..(COL_WIDTH - 2) {
                    s.push('-');
                }
                parts.push(s);
            } else if i > min_lane && i < max_lane {
                // Middle of arrow
                let mut s = String::new();
                for _ in 0..COL_WIDTH {
                    s.push('-');
                }
                parts.push(s);
            } else {
                parts.push(format!("{:^width$}", "|", width = COL_WIDTH));
            }
        }

        if !label.is_empty() {
            // Add label line before arrow
            let mid_lane = (from + to) / 2;
            self.add_action(mid_lane, label);
        }

        self.lines.push(parts.join(""));
    }

    /// Add a note spanning across lanes
    fn add_note(&mut self, text: &str) {
        let total = COL_WIDTH * self.lanes.len();
        let note = format!("... {} ...", text);
        self.lines
            .push(format!("{:^width$}", note, width = total));
    }

    /// Add a section header
    fn add_section(&mut self, text: &str) {
        self.add_blank();
        let header = format!("[{}]", text);
        self.add_action(0, &header);
        self.add_blank();
    }

    fn make_lane_line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for _ in &self.lanes {
            parts.push(format!("{:^width$}", "|", width = COL_WIDTH));
        }
        parts.join("")
    }

    /// Render all lines to stdout
    pub fn render(&self) {
        for line in &self.lines {
            println!("{}", line.trim_end());
        }
    }
}

/// Map execution context to lane index
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
    let mut dia = SequenceDiagram::new(&["User Space", "Kernel", "Hardware"]);

    dia.add_header();
    dia.add_separator();
    dia.add_blank();

    dia.add_action(1, &format!("<<< {} >>>", name));
    dia.add_blank();

    // Phase 1
    dia.add_action(1, &format!("=== {} ===", timeline.phase1.name));
    render_phase_calls(&mut dia, &timeline.phase1.call_chain);

    // Separation
    dia.add_blank();
    dia.add_note(&timeline.separation);
    dia.add_blank();

    // Phase 2
    dia.add_action(1, &format!("=== {} ===", timeline.phase2.name));
    render_phase_calls(&mut dia, &timeline.phase2.call_chain);

    dia.add_blank();
    dia.add_separator();

    dia.render();
}

fn render_phase_calls(dia: &mut SequenceDiagram, chain: &CallChain) {
    dia.add_section(&chain.trigger_source);

    let mut prev_lane: Option<usize> = None;

    for node in &chain.nodes {
        let lane = context_to_lane(&node.context);

        // If switching lanes, draw an arrow
        if let Some(pl) = prev_lane {
            if pl != lane {
                dia.add_arrow(pl, lane, "");
            }
        }

        let marker = if node.is_user_entry {
            " <-- YOUR CODE"
        } else {
            ""
        };
        dia.add_action(lane, &format!("{}(){}", node.function, marker));

        if let Some(ref desc) = node.description {
            dia.add_action(lane, &format!("  // {}", desc));
        }

        prev_lane = Some(lane);
    }
}

/// Render a simple call chain as a sequence diagram (for framework callbacks)
pub fn print_chain_sequence(chain: &CallChain) {
    let mut dia = SequenceDiagram::new(&["User Space", "Kernel", "Hardware"]);

    dia.add_header();
    dia.add_separator();
    dia.add_blank();

    dia.add_action(1, &format!("<<< {} >>>", chain.name));
    dia.add_blank();

    dia.add_section(&chain.trigger_source);

    let mut prev_lane: Option<usize> = None;

    for node in &chain.nodes {
        let lane = context_to_lane(&node.context);

        if let Some(pl) = prev_lane {
            if pl != lane {
                dia.add_arrow(pl, lane, "");
            }
        }

        let marker = if node.is_user_entry {
            " <-- YOUR CODE"
        } else {
            ""
        };
        dia.add_action(lane, &format!("{}(){}", node.function, marker));

        if let Some(ref desc) = node.description {
            dia.add_action(lane, &format!("  // {}", desc));
        }

        if let Some(ref file) = node.file {
            dia.add_action(lane, &format!("  // {}", file));
        }

        prev_lane = Some(lane);
    }

    dia.add_blank();
    dia.add_separator();

    dia.render();
}
