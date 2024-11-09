// Memory boundaries
pub struct RefBounds {
    // How much data available above
    pub(crate) above: usize,
    // How much data available below
    pub(crate) below: usize,
}

pub const UNSIZED_BOUNDS: RefBounds = RefBounds {
    above: usize::MAX,
    below: usize::MAX,
};

impl RefBounds {
    pub fn new(above: usize, below: usize) -> Self {
        Self { above, below }
    }

    // Check boundary
    #[inline]
    pub fn check_boundary(&self, offset: isize) -> bool {
        let offset_abs = offset.unsigned_abs();
        match offset.signum() {
            -1 => self.above >= offset_abs,
            1 => self.below >= offset_abs,
            0 => true,
            _ => unreachable!(),
        }
    }

    // Check boundary with specific size
    //
    // -4 ∧ ────── Above = 4
    // -3 │                      (Case1)
    // -2 │  ┌──── Offset = -2 : offset >= 0 || abs(offset) <= above
    // -1 │  │
    //  0 │  │ Size = 4
    //  1 │  │                   (Case2)
    //  2 │  ∨ ─── End    = 2  : end = offset + size;
    //  3 │                      end <= 0 || end <= below
    //  4 ∨ ────── Below = 4
    //
    #[inline]
    pub fn check_sized(&self, offset: isize, size: usize) -> bool {
        // (Case1) offset over above
        if offset < 0 && self.above < offset.unsigned_abs() {
            return false;
        }

        // (Case2) end over below
        let end = offset + (size as isize);
        end <= 0 || self.below >= end.unsigned_abs()
    }

    // Calculate new boundaries with bounds and offset
    // No boundary checking in here
    #[inline]
    pub fn offset(&self, offset: isize) -> Self {
        let sign = offset.signum();
        let offset_abs = offset.unsigned_abs();

        Self {
            above: match sign {
                -1 => self.above - offset_abs,
                1 => self.above + offset_abs,
                0 => self.above,
                _ => unreachable!(),
            },
            below: match sign {
                -1 => self.below + offset_abs,
                1 => self.below - offset_abs,
                0 => self.below,
                _ => unreachable!(),
            },
        }
    }
}

impl Clone for RefBounds {
    fn clone(&self) -> Self {
        Self {
            above: self.above,
            below: self.below,
        }
    }
}
