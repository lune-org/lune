// Memory range for ref or box data. For boundary checking
pub struct FfiRefBounds {
    // Indicates how much data is above the pointer
    pub(crate) high: usize,
    // Indicates how much data is below the pointer
    pub(crate) low: usize,
}

impl FfiRefBounds {
    pub fn new(high: usize, low: usize) -> Self {
        Self { high, low }
    }

    // Check boundary
    pub fn check(&self, offset: isize) -> bool {
        let sign = offset.signum();
        let offset_abs = offset.unsigned_abs();
        if sign == -1 {
            self.high >= offset_abs
        } else if sign == 1 {
            self.low >= offset_abs
        } else {
            // sign == 0
            true
        }
    }

    // Check boundary
    pub fn check_sized(&self, offset: isize, size: usize) -> bool {
        let end = offset + (size as isize) - 1;
        let sign = end.signum();
        let end_abs = end.unsigned_abs();
        if sign == -1 {
            self.high >= end_abs
        } else if sign == 1 {
            self.low >= end_abs
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
            self.high - offset_abs
        } else if sign == 1 {
            self.high + offset_abs
        } else {
            self.high
        };

        let low: usize = if sign == -1 {
            self.low + offset_abs
        } else if sign == 1 {
            self.low - offset_abs
        } else {
            self.low
        };

        Self { high, low }
    }
}
