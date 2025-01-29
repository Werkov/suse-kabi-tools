// Copyright (C) 2024 SUSE LLC <petr.pavlu@suse.com>
// SPDX-License-Identifier: GPL-2.0-or-later

// Implementation of the Myers diff alrogithm:
// Myers, E.W. An O(ND) difference algorithm and its variations. Algorithmica 1, 251--266 (1986).
// https://doi.org/10.1007/BF01840446

use crate::MapIOErr;
use std::fmt::Display;
use std::io::{prelude::*, BufWriter};
use std::ops::{Index, IndexMut};
use std::path::Path;

#[cfg(test)]
mod tests;

/// A step in the edit script.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edit {
    KeepA(usize),
    RemoveA(usize),
    InsertB(usize),
}

/// An edit script which describes how to transform `a` to `b`.
type EditScript = Vec<Edit>;

/// A limited [`Vec`] wrapper which allows indexing by `isize` in range
/// `(-self.0.len() / 2)..((self.0.len() + 1) / 2`) instead of `0..self.0.len()`.
struct IVec<T>(Vec<T>);

impl<T> Index<isize> for IVec<T> {
    type Output = T;
    fn index(&self, index: isize) -> &T {
        let real_index = (self.0.len() / 2).wrapping_add_signed(index);
        &self.0[real_index]
    }
}

impl<T> IndexMut<isize> for IVec<T> {
    fn index_mut(&mut self, index: isize) -> &mut T {
        let real_index = (self.0.len() / 2).wrapping_add_signed(index);
        &mut self.0[real_index]
    }
}

/// An edit step + an identifier of the previous steps leading to the current point during the edit
/// graph traversal.
#[derive(Clone, Copy)]
struct EditChain {
    prev: usize,
    step: Edit,
}

/// A state of a diagonal during the edit graph traversal.
#[derive(Clone)]
struct DiagonalState {
    x: usize,
    edit_index: usize,
}

/// Compares `a` with `b` and returns an edit script describing how to transform the former to the
/// latter.
fn myers<T: AsRef<str> + PartialEq>(a: &[T], b: &[T]) -> EditScript {
    let max = a.len() + b.len();
    let mut v = IVec(vec![
        DiagonalState {
            x: usize::MAX,
            edit_index: usize::MAX,
        };
        // Minium of 3 diagonals to allow accessing `v[1].x` when the inputs are empty.
        std::cmp::max(2 * max + 1, 3)
    ]);
    v[1].x = 0;
    let mut edit_chains = Vec::new();

    for d in 0..(max as isize + 1) {
        for k in (-d..d + 1).step_by(2) {
            // Determine where to progress, insert from `b` or remove from `a`.
            let insert_b = k == -d || (k != d && v[k - 1].x < v[k + 1].x);
            let (mut x, mut edit_index) = if insert_b {
                (v[k + 1].x, v[k + 1].edit_index)
            } else {
                (v[k - 1].x + 1, v[k - 1].edit_index)
            };
            let mut y = x.wrapping_add_signed(-k);

            // Record the step in the edit script. Skip the first step in the algorithm which
            // initially brings the traversal to (0,0).
            if d != 0 {
                edit_chains.push(EditChain {
                    prev: edit_index,
                    step: if insert_b {
                        Edit::InsertB(y - 1)
                    } else {
                        Edit::RemoveA(x - 1)
                    },
                });
                edit_index = edit_chains.len() - 1;
            }

            // Look for a snake.
            while x < a.len() && y < b.len() && a[x] == b[y] {
                (x, y) = (x + 1, y + 1);
                edit_chains.push(EditChain {
                    prev: edit_index,
                    step: Edit::KeepA(x - 1),
                });
                edit_index = edit_chains.len() - 1;
            }

            // Check if the end is reached or more steps are needed.
            if x >= a.len() && y >= b.len() {
                // Traverse the edit chain and turn it into a proper edit script.
                let mut edit_script = EditScript::new();
                while edit_index != usize::MAX {
                    let edit_chain = edit_chains[edit_index];
                    edit_script.push(edit_chain.step);
                    edit_index = edit_chain.prev;
                }
                edit_script.reverse();
                return edit_script;
            }
            v[k] = DiagonalState { x, edit_index };
        }
    }
    unreachable!();
}

pub fn unified<T, W>(a: &[T], b: &[T], path: &Path, writer: W) -> Result<(), crate::Error>
where
    T: AsRef<str> + PartialEq + Display,
    W: Write,
{
    let mut writer = BufWriter::new(writer);

    for edit in myers(a, b) {
        match edit {
            Edit::KeepA(index_a) => writeln!(writer, " {}", a[index_a]).map_io_err(path)?,
            Edit::RemoveA(index_a) => writeln!(writer, "-{}", a[index_a]).map_io_err(path)?,
            Edit::InsertB(index_b) => writeln!(writer, "+{}", b[index_b]).map_io_err(path)?,
        }
    }
    Ok(())
}
