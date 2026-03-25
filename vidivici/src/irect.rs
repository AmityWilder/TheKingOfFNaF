//! Screen region

use core::range::{Range, RangeIter};

use crate::IVec2;

/// Integer rectangle
#[derive(Debug, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct IRect {
    /// Horizontal start position
    pub x: i32,
    /// Vertical start position
    pub y: i32,
    /// Width
    pub w: i32,
    /// Height
    pub h: i32,
}

impl const Default for IRect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl IRect {
    const MAX_AREA: u64 = Self::new(0, 0, i32::MIN, i32::MIN).area();

    /// Construct an [`IRect`] from components
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    /// The position (x,y) as a vector
    pub const fn pos(&self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }

    /// The size as a vector
    pub const fn size(&self) -> IVec2 {
        IVec2::new(self.w, self.h)
    }

    /// The total number of pixels within the region.
    ///
    /// Result is guaranteed to be between 0 and [`Self::MAX_AREA`] inclusively.
    pub const fn area(&self) -> u64 {
        const MAX_AREA_CHECKED: Option<u64> =
            (i32::MIN.unsigned_abs() as u64).checked_mul(i32::MIN.unsigned_abs() as u64);
        const _: () = {
            assert!(MAX_AREA_CHECKED.is_some(), "maximum area should fit in u64");
            assert!(
                IRect::MAX_AREA <= (i64::MAX as u64),
                "maximum area should fit in i64"
            );
        };
        self.w.unsigned_abs() as u64 * self.h.unsigned_abs() as u64
    }
}

/// An iterator over [`IVec2`] positions in an [`IRect`].
///
/// Iterates by row, then by column, starting at `ymin` and ending at `ymax`.
#[derive(Debug, Clone)]
pub struct ScanlineIter {
    it: RangeIter<i64>,
    xmin: i32,
    ymin: i32,
    w: i32,
}

impl ScanlineIter {
    fn new(rect: IRect) -> Self {
        Self {
            it: Range::from(0..rect.area() as i64).into_iter(),
            xmin: rect.x,
            ymin: rect.y,
            w: rect.w,
        }
    }

    fn index_to_vec2(&self, i: i64) -> IVec2 {
        let w = self.w as i64;
        let (row, col) = (i / w, i % w);
        IVec2::new(
            (self.xmin as i64 + col)
                .try_into()
                .expect("column exceeds i32::MAX"),
            (self.ymin as i64 + row)
                .try_into()
                .expect("row exceeds i32::MAX"),
        )
    }
}

impl IntoIterator for IRect {
    type Item = IVec2;
    type IntoIter = ScanlineIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ScanlineIter::new(self)
    }
}

impl Iterator for ScanlineIter {
    type Item = IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.w == 0 {
            return None;
        }
        self.it.next().map(|i| self.index_to_vec2(i))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }
}

impl std::iter::FusedIterator for ScanlineIter where RangeIter<i64>: std::iter::FusedIterator {}

impl DoubleEndedIterator for ScanlineIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.w == 0 {
            return None;
        }
        self.it.next_back().map(|i| self.index_to_vec2(i))
    }
}
