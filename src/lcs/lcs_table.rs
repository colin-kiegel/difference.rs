use std::ops::{Index, IndexMut};

#[cfg(test)] use std::fmt::{self, Display};
#[cfg(test)] use prettytable::Table;
#[cfg(test)] use prettytable::row::Row;
#[cfg(test)] use prettytable::cell::Cell;

pub struct LcsTable<'a, T: 'a + PartialEq> {
    table: Vec<LcsEntry>,
    a: &'a [T],
    b: &'a [T],
}

#[derive(Debug, Copy, Clone)]
pub struct LcsEntry {
    pub step: Step,
    pub len: usize,
}

const LCS_ENTRY_AT_BOUNDARY: &'static LcsEntry = &LcsEntry {
    step: XorY,
    len: 0,
};

#[derive(Debug, Copy, Clone)]
pub enum Step {
    Both,
    OnlyX,
    OnlyY,
    XorY,
}

pub use self::Step::*;

pub struct LcsIter<'a, T: 'a + PartialEq> {
    lcs: &'a LcsTable<'a, T>,
    x: usize,
    y: usize,
    size_x: usize,
    size_y: usize,
}

impl<'a, T: PartialEq> LcsTable<'a, T> {
    pub fn _new(a: &'a [T], b: &'a [T]) -> LcsTable<'a, T> {
        let cap = a.len() * b.len();
        let mut table = Vec::with_capacity(cap);

        for _ in 0..cap {
            table.push(*LCS_ENTRY_AT_BOUNDARY)
        };

        LcsTable{
            table: table,
            a: a,
            b: b,
        }
    }

    pub fn from(a: &'a [T], b: &'a [T]) -> LcsTable<'a, T> {
        // https://en.wikipedia.org/wiki/Longest_common_subsequence_problem
        // can be accessed via lcs[i + j*N], where i=0..N, j=0..M
        let cap = a.len() * b.len();
        let mut table = Vec::with_capacity(cap);

        // TODO: optimize this push away!
        for _ in 0..cap {
            table.push(*LCS_ENTRY_AT_BOUNDARY)
        };


        let mut lcs = LcsTable{
            table: table,
            a: a,
            b: b,
        };

        for (i, x) in a.iter().enumerate().rev() {
            for (j, y) in b.iter().enumerate().rev() {

                let (step, len) = if x == y {
                    let prev_len = lcs[(i+1, j+1)].len;

                    (Both, prev_len+1)
                } else {
                    let len_x = lcs[(i+1, j)].len;
                    let len_y = lcs[(i, j+1)].len;

                    if len_x == len_y {
                        (XorY, len_x)
                    } else if len_x > len_y {
                        (OnlyX, len_x)
                    } else {
                        (OnlyY, len_y)
                    }
                };

                lcs[(i, j)].set(step, len);
            }
        }

        lcs
    }

    pub fn iter(&'a self) -> LcsIter<'a, T> {
        let (size_x, size_y) = (self.a.len(), self.b.len());

        LcsIter {
            lcs: self,
            x: 0,
            y: 0,
            size_x: size_x,
            size_y: size_y,
        }
    }
}

impl LcsEntry {
    pub fn set(&mut self, step: Step, len: usize) {
        self.step = step;
        self.len = len;
    }
}

impl<'a, T: PartialEq> Index<(usize, usize)> for LcsTable<'a, T> {
    type Output = LcsEntry;

    fn index(&self, i: (usize, usize)) -> &LcsEntry {
        let (x, y) = (i.0, i.1);
        let (size_x, size_y) = (self.a.len(), self.b.len());

        if x >= size_x || y >= size_y {
            LCS_ENTRY_AT_BOUNDARY
        } else {
            unsafe {
                // we may only enter this block, when `x < size_x && y < size_y`
                self.table.get_unchecked(x + y*size_x)
            }
        }
    }
}

impl<'a, T: PartialEq> IndexMut<(usize, usize)> for LcsTable<'a, T> {
    fn index_mut(&mut self, i: (usize, usize)) -> &mut LcsEntry {
        let (x, y) = (i.0, i.1);
        let (size_x, size_y) = (self.a.len(), self.b.len());

        if x >= size_x || y >= size_y {
            panic!("Mutable LcsTable access is out of bounds. \
                Tried to access element ({}, {}) in 0..{} x 0..{}",
                x, y, size_x, size_y);
        } else {
            unsafe {
                // we may only enter this block, when `x < size_x && y < size_y`
                self.table.get_unchecked_mut(x + y*size_x)
            }
        }
    }
}

impl<'a, T: PartialEq> Iterator for LcsIter<'a, T> {
    type Item = (Step, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x == self.size_x || self.y == self.size_y {
            let result = if self.x < self.size_x {
                let t = &self.lcs.a[self.x];
                self.x = self.x + 1;
                Some((OnlyX, t))
            } else if self.y < self.size_y {
                let t = &self.lcs.b[self.y];
                self.y = self.y + 1;
                Some((OnlyY, t))
            } else {
                None
            };

            return result;
        }

        Some(match self.lcs[(self.x, self.y)].step {
            Both => {
                let t = &self.lcs.a[self.x];
                self.x = self.x + 1;
                self.y = self.y + 1;
                (Both, t)
            }
            OnlyX => {
                let t = &self.lcs.a[self.x];
                self.x = self.x + 1;
                (OnlyX, t)
            }
            OnlyY => {
                let t = &self.lcs.b[self.y];
                self.y = self.y + 1;
                (OnlyY, t)
            }
            XorY => {
                let t = &self.lcs.b[self.y];
                self.y = self.y + 1;
                (OnlyY, t)
            }
        })
    }
}

#[cfg(test)]
// Useful for debugging/inspecting the algorithm.
impl<'a, T: PartialEq + Display + Default + Clone> Display for LcsTable<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const TRUNCATE_LENGTH: usize = 5;
        let mut table = Table::new();

        let mut row: Vec<_> = self.a.iter()
            .map(|s| Cell::new(&format!("{:.*}", TRUNCATE_LENGTH, s)))
            .collect();
        row.insert(0, Cell::new(""));
        table.add_row(Row::new(row));

        for y in 0..self.b.len() {
            let mut row = Vec::with_capacity(self.a.len()+1);
            row.push(Cell::new(&format!("{:.*}", TRUNCATE_LENGTH, self.b[y])));

            for x in 0..self.a.len() {
                let entry = self[(x, y)];
                let arrow = match entry.step {
                    Both => "↘",
                    OnlyY => "↓",
                    OnlyX => "→",
                    XorY => "↓ or →",
                };
                row.push(Cell::new(&format!("{} {}", entry.len, arrow)));
            }

            table.add_row(Row::new(row));
        }

        write!(f, "{}", table)
    }
}

#[test]
fn test_lcs_table() {
    let x: Vec<&str> = "A G C A T".split(" ").collect();
    let y: Vec<&str> = "G A C".split(" ").collect();

    let lcs = LcsTable::from(&x, &y);

    assert_eq!(format!("{}", lcs),
        "+---+----------+----------+----------+----------+----------+\
       \n|   | A        | G        | C        | A        | T        |\
       \n+---+----------+----------+----------+----------+----------+\
       \n| G | 2 ↓ or → | 2 ↘      | 1 ↓ or → | 1 ↓      | 0 ↓ or → |\
       \n+---+----------+----------+----------+----------+----------+\
       \n| A | 2 ↘      | 1 ↓ or → | 1 ↓ or → | 1 ↘      | 0 ↓ or → |\
       \n+---+----------+----------+----------+----------+----------+\
       \n| C | 1 →      | 1 →      | 1 ↘      | 0 ↓ or → | 0 ↓ or → |\
       \n+---+----------+----------+----------+----------+----------+\
       \n");
}
