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

    #[inline]
    pub fn is_unsized(&self) -> bool {
        self.above == usize::MAX && self.below == usize::MAX
    }

    // Check boundary
    #[inline]
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

    // Check boundary with specific size
    #[inline]
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

    // Calculate new boundaries from bounds and offset
    // No boundary checking in here
    #[inline]
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

impl Clone for RefBounds {
    fn clone(&self) -> Self {
        Self {
            above: self.above,
            below: self.below,
        }
    }
}
