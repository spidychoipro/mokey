use crate::geometry::{Point, Rect};

pub struct GridSession {
    /// The monitor this session is operating on.
    pub monitor: Rect,
    /// The current zoom region within the monitor.
    pub region: Rect,
    /// Zoom depth (0 = whole monitor).
    pub depth: u32,
    pub grid_size: u32,
    pub max_depth: u32,
    /// When vim mode is on, a leading digit sequence is interpreted as a
    /// repeat count *if* the next key is a motion key.
    pub pending_count: Option<u32>,
    history: Vec<Rect>,
}

impl GridSession {
    pub fn start(monitor: Rect, grid_size: u32, max_depth: u32) -> GridSession {
        GridSession {
            monitor,
            region: monitor,
            depth: 0,
            grid_size: grid_size.clamp(2, 9),
            max_depth: max_depth.max(1),
            pending_count: None,
            history: Vec::new(),
        }
    }

    pub fn grid(&self) -> crate::grid::Grid {
        crate::grid::Grid::new(self.grid_size)
    }

    pub fn can_zoom(&self) -> bool {
        self.depth < self.max_depth && !self.grid().at_min_zoom(self.region)
    }

    pub fn click_point(&self) -> Point {
        self.region.center()
    }

    pub fn cell_rect(&self, label: u32) -> Option<Rect> {
        self.grid().cell_rect(self.region, label)
    }

    /// Zoom into the cell labelled `label`. Returns the new region.
    pub fn zoom_to(&mut self, label: u32) -> Option<Rect> {
        if !self.can_zoom() {
            return None;
        }
        let cell = self.cell_rect(label)?;
        self.history.push(self.region);
        self.region = cell;
        self.depth += 1;
        self.pending_count = None;
        Some(cell)
    }

    pub fn zoom_out(&mut self) {
        if let Some(parent) = self.history.pop() {
            self.region = parent;
            self.depth = self.depth.saturating_sub(1);
        }
    }

    /// Record a digit key. In grid mode a digit is a cell label; in vim mode
    /// a digit can be a repeat-count prefix. Returns true if consumed as count.
    pub fn push_digit(&mut self, digit: u32) -> bool {
        let n = self.pending_count.unwrap_or(0);
        self.pending_count = Some((n * 10 + digit).clamp(1, 999));
        true
    }

    pub fn take_count(&mut self) -> u32 {
        self.pending_count.take().unwrap_or(1)
    }

    pub fn clear_count(&mut self) {
        self.pending_count = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor() -> Rect {
        Rect { x: 0, y: 0, w: 1920, h: 1080 }
    }

    #[test]
    fn starts_on_full_monitor() {
        let s = GridSession::start(monitor(), 3, 4);
        assert_eq!(s.region, monitor());
        assert_eq!(s.depth, 0);
        assert_eq!(s.click_point(), Point { x: 960, y: 540 });
    }

    #[test]
    fn zoom_into_cells_and_back() {
        let mut s = GridSession::start(monitor(), 3, 4);
        let cell = s.zoom_to(5).unwrap(); // center
        assert_eq!(cell, Rect { x: 640, y: 360, w: 640, h: 360 });
        assert_eq!(s.depth, 1);
        s.zoom_out();
        assert_eq!(s.region, monitor());
        assert_eq!(s.depth, 0);
    }

    #[test]
    fn zoom_stops_at_max_depth() {
        let mut s = GridSession::start(monitor(), 2, 2);
        assert!(s.zoom_to(1).is_some());
        assert!(s.zoom_to(1).is_some());
        assert!(s.zoom_to(1).is_none());
    }

    #[test]
    fn digit_count_accumulates() {
        let mut s = GridSession::start(monitor(), 3, 4);
        s.push_digit(3);
        s.push_digit(2);
        assert_eq!(s.take_count(), 32);
        assert_eq!(s.pending_count, None);
    }
}
