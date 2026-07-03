// Memory boundaries
#[derive(Clone, Copy)]
pub struct RefBounds {
    // How much bytes available above
    pub(crate) above: usize,
    // How much bytes available below
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

    // Check offset is in boundary
    #[inline]
    pub fn check_offset(&self, offset: isize) -> bool {
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

        // (Case2) end over below; overflow means the range cannot fit
        let Ok(size) = isize::try_from(size) else {
            return false;
        };
        let Some(end) = offset.checked_add(size) else {
            return false;
        };
        end <= 0 || self.below >= end.unsigned_abs()
    }

    // Calculate new boundaries with offset
    // No boundary checking in here
    //
    // Above = 3
    //   ∧ ───∧───
    // -3│    │ New above = 2
    // -2│    │
    // -1│ <────── Offset = -1
    //  0│    │
    //  1│    │
    //  2│    │
    //  3│    │ New below = 4
    //   ∨ ───∨───
    // Below = 3
    //
    // Saturating keeps UNSIZED_BOUNDS (usize::MAX) unsized after offsetting
    #[inline]
    pub fn offset(&self, offset: isize) -> Self {
        let sign = offset.signum();
        let offset_abs = offset.unsigned_abs();

        Self {
            above: match sign {
                -1 => self.above.saturating_sub(offset_abs),
                1 => self.above.saturating_add(offset_abs),
                0 => self.above,
                _ => unreachable!(),
            },
            below: match sign {
                -1 => self.below.saturating_add(offset_abs),
                1 => self.below.saturating_sub(offset_abs),
                0 => self.below,
                _ => unreachable!(),
            },
        }
    }
}
