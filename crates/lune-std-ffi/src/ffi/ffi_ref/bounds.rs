use std::clone;

// Memory range for ref or box data. For boundary checking
pub struct FfiRefBounds {
    // Indicates how much data is above the pointer
    pub(crate) above: usize,
    // Indicates how much data is below the pointer
    pub(crate) below: usize,
}

pub const UNSIZED_BOUNDS: FfiRefBounds = FfiRefBounds {
    above: usize::MAX,
    below: usize::MAX,
};

impl FfiRefBounds {
    pub fn new(above: usize, below: usize) -> Self {
        Self { above, below }
    }

    pub fn is_unsized(&self) -> bool {
        self.above == usize::MAX && self.below == usize::MAX
    }

    // Check boundary
    pub fn check_boundary(&self, offset: isize) -> bool {
        if self.is_unsized() {
            return true;
        }
        let sign = offset.signum();
        let offset_abs = offset.unsigned_abs();
        if sign == -1 {
            self.above >= offset_abs
        } else if sign == 1 {
            self.below >= offset_abs
        } else {
            // sign == 0
            true
        }
    }

    // Check boundary
    // Check required here
    pub fn check_sized(&self, offset: isize, size: usize) -> bool {
        if self.is_unsized() {
            return true;
        }
        if offset < 0 && self.above < offset.unsigned_abs() {
            return true;
        }
        let end = offset + (size as isize) - 1;
        let end_sign = end.signum();
        let end_abs = end.unsigned_abs();
        if end_sign == -1 {
            self.above >= end_abs
        } else if end_sign == 1 {
            self.below >= end_abs
        } else {
            // sign == 0
            true
        }
    }

    // Calculate new bounds from bounds and offset
    // No boundary checking in here
    pub fn offset(&self, offset: isize) -> Self {
        let sign = offset.signum();
        let offset_abs = offset.unsigned_abs();

        let high: usize = if sign == -1 {
            self.above - offset_abs
        } else if sign == 1 {
            self.above + offset_abs
        } else {
            self.above
        };

        let low: usize = if sign == -1 {
            self.below + offset_abs
        } else if sign == 1 {
            self.below - offset_abs
        } else {
            self.below
        };

        Self {
            above: high,
            below: low,
        }
    }
}

impl Clone for FfiRefBounds {
    fn clone(&self) -> Self {
        Self {
            above: self.above,
            below: self.below,
        }
    }
}
