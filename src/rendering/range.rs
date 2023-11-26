use std::{
    cmp::Ordering,
    ops::{Add, Sub},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}

impl Range {
    pub fn new(start: i32, end: i32) -> Self {
        assert!(end >= start);
        Self { start, end }
    }

    pub fn until(end: i32) -> Self {
        Self::new(0, end)
    }

    #[allow(unused)]
    pub const fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn covered(self) -> Self {
        Self::new(self.start, self.start)
    }

    pub fn cover(self, cover: Self) -> Self {
        match self.start.cmp(&cover.start) {
            // |...
            //    -...
            Ordering::Less => match self.end.cmp(&cover.start) {
                // |    |
                //         ------
                // ^    ^
                Ordering::Less => self,

                // |    |
                //      ------
                // ^    ^
                Ordering::Equal => self,

                // |    |
                //    ------
                // ^  ^
                Ordering::Greater => Self::new(self.start, cover.start),
            },

            // |...
            // -...
            Ordering::Equal => match self.end.cmp(&cover.end) {
                // |    |
                // --------
                // ^
                // ^
                Ordering::Less => self.covered(),

                // |    |
                // ------
                // ^
                // ^
                Ordering::Equal => self.covered(),

                // |    |
                // ---
                //   ^  ^
                Ordering::Greater => Self::new(cover.end, self.end),
            },

            //    |...
            // -...
            Ordering::Greater => {
                match self.start.cmp(&cover.end) {
                    //   |...
                    // ------
                    Ordering::Less => match self.end.cmp(&cover.end) {
                        //   |    |
                        // ----------
                        //   ^
                        //   ^
                        Ordering::Less => self.covered(),

                        //   |    |
                        // --------
                        //   ^
                        //   ^
                        Ordering::Equal => self.covered(),

                        //   |    |
                        // -----
                        //     ^  ^
                        Ordering::Greater => Self::new(cover.end, self.end),
                    },

                    //      |...
                    // ------
                    Ordering::Equal => self,

                    //          |...
                    // ------
                    Ordering::Greater => self,
                }
            }
        }
    }
}

impl Add<i32> for Range {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self::new(self.start + rhs, self.end + rhs)
    }
}

impl Sub<i32> for Range {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::new(self.start - rhs, self.end - rhs)
    }
}
