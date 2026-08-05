use crate::geometry::Rect;

pub struct Grid {
    pub size: u32,
}

impl Grid {
    pub fn new(size: u32) -> Grid {
        let size = size.clamp(2, 9);
        Grid { size }
    }

    /// Number of labels in the grid (size x size).
    pub fn label_count(&self) -> u32 {
        self.size * self.size
    }

    /// Resolve a 1-based label (row-major: 1..=size^2) into a cell rect
    /// within `region`. Returns None for out-of-range labels.
    pub fn cell_rect(&self, region: Rect, label: u32) -> Option<Rect> {
        if label == 0 || label > self.label_count() {
            return None;
        }
        let idx = label - 1;
        let col = idx % self.size;
        let row = idx / self.size;
        Some(region.sub_rect(col, row, self.size))
    }

    /// A grid session can no longer meaningfully zoom when a cell would be
    /// smaller than `min_cell`.
    pub fn at_min_zoom(&self, region: Rect) -> bool {
        region.w < self.size || region.h < self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_count_matches_size() {
        assert_eq!(Grid::new(3).label_count(), 9);
        assert_eq!(Grid::new(2).label_count(), 4);
    }

    #[test]
    fn cell_layout_is_row_major() {
        let grid = Grid::new(3);
        let region = Rect {
            x: 0,
            y: 0,
            w: 300,
            h: 300,
        };
        // cell 1 = top-left, cell 5 = center, cell 9 = bottom-right
        assert_eq!(grid.cell_rect(region, 1).unwrap(), Rect { x: 0, y: 0, w: 100, h: 100 });
        assert_eq!(grid.cell_rect(region, 5).unwrap(), Rect { x: 100, y: 100, w: 100, h: 100 });
        assert_eq!(grid.cell_rect(region, 9).unwrap(), Rect { x: 200, y: 200, w: 100, h: 100 });
    }

    #[test]
    fn out_of_range_labels_rejected() {
        let grid = Grid::new(3);
        let region = Rect { x: 0, y: 0, w: 100, h: 100 };
        assert!(grid.cell_rect(region, 0).is_none());
        assert!(grid.cell_rect(region, 10).is_none());
    }

    #[test]
    fn at_min_zoom_when_cells_are_tiny() {
        let grid = Grid::new(3);
        let tiny = Rect { x: 0, y: 0, w: 2, h: 2 };
        assert!(grid.at_min_zoom(tiny));
    }
}
