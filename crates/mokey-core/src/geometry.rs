#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn center(&self) -> Point {
        Point {
            x: self.x + (self.w as i32) / 2,
            y: self.y + (self.h as i32) / 2,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x
            && p.x < self.x + self.w as i32
            && p.y >= self.y
            && p.y < self.y + self.h as i32
    }

    pub fn sub_rect(&self, col: u32, row: u32, size: u32) -> Rect {
        let cw = self.w / size;
        let ch = self.h / size;
        Rect {
            x: self.x + (col * cw) as i32,
            y: self.y + (row * ch) as i32,
            w: cw,
            h: ch,
        }
    }
}
