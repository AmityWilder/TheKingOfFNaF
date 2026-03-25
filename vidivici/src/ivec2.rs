//! Screen position

/// Integer 2D vector
#[derive(Debug, Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct IVec2 {
    /// Horizontal coordinate
    pub x: i32,
    /// Vertical coordinate
    pub y: i32,
}

impl const Default for IVec2 {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

const _: () = {
    assert!(
        (i32::MIN as i64).unsigned_abs() > i32::MAX as u64,
        "sanity check: min integer is greater magnitude than max integer"
    );
    // Untrue
    // assert!(
    //     (i32::MIN as i64).unsigned_abs() == (i32::MIN as i64).cast_unsigned(),
    //     "sanity check: abs of min i32 widened to i64 is same as casting to unsigned"
    // );
};

impl IVec2 {
    /// Construct a new `IVec2` from `x` and `y`
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Dot product
    pub const fn dot(self, rhs: Self) -> i64 {
        const WORST_CASE: Option<i64> = (i32::MIN as i64).checked_mul(i32::MIN as i64);
        const _: () = assert!(WORST_CASE.is_some(), "worst case should fit in i64");

        self.x as i64 * rhs.x as i64 + self.y as i64 * rhs.y as i64
    }

    /// Vector norm - square of [`len`](Self::len)
    pub const fn mag_sqr(self) -> u64 {
        const WORST_CASE: Option<u64> = (i32::MIN as i64)
            .unsigned_abs()
            .checked_mul((i32::MIN as i64).unsigned_abs());
        const _: () = assert!(WORST_CASE.is_some(), "worst case should fit in u64");

        let x = (self.x as i64).unsigned_abs();
        let y = (self.y as i64).unsigned_abs();
        x * x + y * y
    }

    /// Vector magnitude - length
    pub const fn mag(self) -> u32 {
        const LARGEST_LEN: u64 =
            ((i32::MIN as i64).unsigned_abs() * (i32::MIN as i64).unsigned_abs()).isqrt();
        const _: () = assert!(
            LARGEST_LEN <= u32::MAX as u64, // 2147483648 <= 4294967295
            "sqrt of largest len_sqr should fit in u32"
        );

        self.mag_sqr().isqrt() as u32
    }

    /// Get the `n`th vector component
    pub const fn get(&self, n: usize) -> Option<&i32> {
        match n {
            0 => Some(&self.x),
            1 => Some(&self.y),
            _ => None,
        }
    }

    /// Get the `n`th vector component mutably
    pub const fn get_mut(&mut self, n: usize) -> Option<&mut i32> {
        match n {
            0 => Some(&mut self.x),
            1 => Some(&mut self.y),
            _ => None,
        }
    }

    /// An iterator over the vector components
    #[inline]
    pub fn iter(&self) -> std::array::IntoIter<&i32, 2> {
        [&self.x, &self.y].into_iter()
    }

    /// An iterator over the mutable vector components
    #[inline]
    pub fn iter_mut(&mut self) -> std::array::IntoIter<&mut i32, 2> {
        [&mut self.x, &mut self.y].into_iter()
    }
}
impl IntoIterator for IVec2 {
    type Item = i32;
    type IntoIter = std::array::IntoIter<Self::Item, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        <[i32; 2]>::from(self).into_iter()
    }
}
impl<'a> IntoIterator for &'a IVec2 {
    type Item = &'a i32;
    type IntoIter = std::array::IntoIter<Self::Item, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut IVec2 {
    type Item = &'a mut i32;
    type IntoIter = std::array::IntoIter<Self::Item, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl From<(i32, i32)> for IVec2 {
    #[inline]
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}
impl From<IVec2> for (i32, i32) {
    #[inline]
    fn from(IVec2 { x, y }: IVec2) -> Self {
        (x, y)
    }
}
impl From<[i32; 2]> for IVec2 {
    #[inline]
    fn from([x, y]: [i32; 2]) -> Self {
        Self { x, y }
    }
}
impl From<IVec2> for [i32; 2] {
    #[inline]
    fn from(IVec2 { x, y }: IVec2) -> Self {
        [x, y]
    }
}
impl std::ops::Index<usize> for IVec2 {
    type Output = i32;

    #[track_caller]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of range")
    }
}
impl std::ops::IndexMut<usize> for IVec2 {
    #[track_caller]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of range")
    }
}
impl std::ops::Add for IVec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl std::ops::AddAssign for IVec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::Sub for IVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl std::ops::SubAssign for IVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl std::ops::Mul for IVec2 {
    type Output = Self;

    /// Hadamard product
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}
impl std::ops::MulAssign for IVec2 {
    /// Hadamard product
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}
impl std::ops::Div for IVec2 {
    type Output = Self;

    /// Hadamard quotient
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}
impl std::ops::DivAssign for IVec2 {
    /// Hadamard quotient
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}
impl std::ops::Mul<i32> for IVec2 {
    type Output = Self;

    /// Scalar product
    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl std::ops::MulAssign<i32> for IVec2 {
    /// Scalar product
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
impl std::ops::Div<i32> for IVec2 {
    type Output = Self;

    /// Scalar quotient
    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
impl std::ops::DivAssign<i32> for IVec2 {
    /// Scalar quotient
    #[inline]
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
