#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Add,
    Delete,
    Common,
}

#[derive(Debug)]
pub struct DiffLimitExceeded {
    pub comparisons_used: usize,
    pub comparison_budget: usize,
    pub left_size: usize,
    pub right_size: usize,
}

impl std::fmt::Display for DiffLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Diff limit exceeded: {} comparisons used out of budget {}",
            self.comparisons_used, self.comparison_budget
        )
    }
}

impl std::error::Error for DiffLimitExceeded {}

pub struct ONPDiff<'a> {
    a: &'a [&'a str],
    b: &'a [&'a str],
    m: usize,
    n: usize,
    reverse: bool,
    offset: usize,
    delta: isize,
    path: Vec<isize>,
    backtrack: Vec<(usize, usize, isize)>,
    comparison_budget: usize,
    comparisons_used: usize,
}

impl<'a> ONPDiff<'a> {
    pub fn new(a: &'a [&'a str], b: &'a [&'a str], comparison_budget: usize) -> Self {
        let (a, b, reverse) = if a.len() > b.len() {
            (b, a, true)
        } else {
            (a, b, false)
        };

        let m = a.len();
        let n = b.len();
        let offset = m + 1;
        let delta = n as isize - m as isize;

        let path_size = m + n + 3;
        let path = vec![-1; path_size];

        Self {
            a,
            b,
            m,
            n,
            reverse,
            offset,
            delta,
            path,
            backtrack: Vec::new(),
            comparison_budget,
            comparisons_used: 0,
        }
    }

    pub fn diff(&mut self) -> Result<Vec<(String, Operation)>, DiffLimitExceeded> {
        let shortest_path = self.compose()?;
        Ok(self.build_edit_script(&shortest_path))
    }

    pub fn short_diff(&mut self) -> Result<Vec<(String, Operation)>, DiffLimitExceeded> {
        let shortest_path = self.compose()?;
        Ok(self.build_short_edit_script(&shortest_path))
    }

    pub fn paragraph_diff(&mut self) -> Result<Vec<(String, Operation)>, DiffLimitExceeded> {
        let ses = self.short_diff()?;
        Ok(self.build_paragraph_edit_script(&ses))
    }

    fn compose(&mut self) -> Result<Vec<(usize, usize)>, DiffLimitExceeded> {
        let fp_size = self.m + self.n + 3;
        let mut fp = vec![-1; fp_size];

        let mut p = 0;
        loop {
<<<<<<< HEAD
            let mut k = -(p as isize);
            while k <= self.delta - 1 {
=======
            p += 1;

            let mut k = -p;
            while k < self.delta {
>>>>>>> origin/rust-migration-6812869939882800025
                fp[(k + self.offset as isize) as usize] = self.snake(
                    k,
                    fp[(k - 1 + self.offset as isize) as usize] + 1,
                    fp[(k + 1 + self.offset as isize) as usize],
                )?;
                k += 1;
            }

            k = self.delta + p;
            while k > self.delta {
                fp[(k + self.offset as isize) as usize] = self.snake(
                    k,
                    fp[(k - 1 + self.offset as isize) as usize] + 1,
                    fp[(k + 1 + self.offset as isize) as usize],
                )?;
                k -= 1;
            }

            fp[(self.delta + self.offset as isize) as usize] = self.snake(
                self.delta,
                fp[(self.delta - 1 + self.offset as isize) as usize] + 1,
                fp[(self.delta + 1 + self.offset as isize) as usize],
            )?;

            if fp[(self.delta + self.offset as isize) as usize] as usize == self.n {
                break;
            }

            p += 1;
        }

        let mut r = self.path[(self.delta + self.offset as isize) as usize];
        let mut shortest_path = Vec::new();

        while r != -1 {
            let node = &self.backtrack[r as usize];
            shortest_path.push((node.0, node.1));
            r = node.2;
        }

        Ok(shortest_path)
    }

    fn snake(&mut self, k: isize, p: isize, pp: isize) -> Result<isize, DiffLimitExceeded> {
        let k_offset = (k + self.offset as isize) as usize;
        let r: isize;
        let y: isize;

        if p > pp {
            r = self.path[k_offset - 1];
            y = p;
        } else {
            r = self.path[k_offset + 1];
            y = pp;
        }

        let x = y - k;
        let mut y_idx = y as usize;
        let mut x_idx = x as usize;

        while x_idx < self.m && y_idx < self.n {
            self.comparisons_used += 1;
            if self.comparisons_used > self.comparison_budget {
                return Err(DiffLimitExceeded {
                    comparisons_used: self.comparisons_used,
                    comparison_budget: self.comparison_budget,
                    left_size: self.m,
                    right_size: self.n,
                });
            }

            if self.a[x_idx] != self.b[y_idx] {
                break;
            }

            x_idx += 1;
            y_idx += 1;
        }

        self.path[k_offset] = self.backtrack.len() as isize;
        self.backtrack.push((x_idx, y_idx, r));

        Ok(y_idx as isize)
    }

    fn build_edit_script(&self, shortest_path: &[(usize, usize)]) -> Vec<(String, Operation)> {
        let mut ses = Vec::new();
        let mut px = 0;
        let mut py = 0;

        for node in shortest_path.iter().rev() {
            while px < node.0 || py < node.1 {
                if (node.1 as isize - node.0 as isize) > (py as isize - px as isize) {
                    let t = if self.reverse {
                        Operation::Delete
                    } else {
                        Operation::Add
                    };
                    ses.push((self.b[py].to_string(), t));
                    py += 1;
                } else if (node.1 as isize - node.0 as isize) < (py as isize - px as isize) {
                    let t = if self.reverse {
                        Operation::Add
                    } else {
                        Operation::Delete
                    };
                    ses.push((self.a[px].to_string(), t));
                    px += 1;
                } else {
                    ses.push((self.a[px].to_string(), Operation::Common));
                    px += 1;
                    py += 1;
                }
            }
        }
        ses
    }

    fn build_short_edit_script(
        &self,
        shortest_path: &[(usize, usize)],
    ) -> Vec<(String, Operation)> {
        let mut ses: Vec<(String, Operation)> = Vec::new();
        let mut px = 0;
        let mut py = 0;

        for node in shortest_path.iter().rev() {
            while px < node.0 || py < node.1 {
                if (node.1 as isize - node.0 as isize) > (py as isize - px as isize) {
                    let t = if self.reverse {
                        Operation::Delete
                    } else {
                        Operation::Add
                    };
                    if let Some(last) = ses.last_mut() {
                        if last.1 == t {
                            last.0.push_str(self.b[py]);
                        } else {
                            ses.push((self.b[py].to_string(), t));
                        }
                    } else {
                        ses.push((self.b[py].to_string(), t));
                    }
                    py += 1;
                } else if (node.1 as isize - node.0 as isize) < (py as isize - px as isize) {
                    let t = if self.reverse {
                        Operation::Add
                    } else {
                        Operation::Delete
                    };
                    if let Some(last) = ses.last_mut() {
                        if last.1 == t {
                            last.0.push_str(self.a[px]);
                        } else {
                            ses.push((self.a[px].to_string(), t));
                        }
                    } else {
                        ses.push((self.a[px].to_string(), t));
                    }
                    px += 1;
                } else {
                    if let Some(last) = ses.last_mut() {
                        if last.1 == Operation::Common {
                            last.0.push_str(self.a[px]);
                        } else {
                            ses.push((self.a[px].to_string(), Operation::Common));
                        }
                    } else {
                        ses.push((self.a[px].to_string(), Operation::Common));
                    }
                    px += 1;
                    py += 1;
                }
            }
        }
        ses
    }

    fn build_paragraph_edit_script(&self, ses: &[(String, Operation)]) -> Vec<(String, Operation)> {
        let mut paragraph_ses = Vec::new();
        let mut i = 0;

        while i < ses.len() {
            if ses[i].1 == Operation::Common {
                paragraph_ses.push(ses[i].clone());
            } else {
                let op_code = ses[i].1;
                let opposite_op_code = if op_code == Operation::Add {
                    Operation::Delete
                } else {
                    Operation::Add
                };

                let mut j = i + 1;
                while j < ses.len() && ses[j].1 == op_code {
                    j += 1;
                }

                if j >= ses.len() {
                    paragraph_ses.extend_from_slice(&ses[i..j]);
                    i = j;
                    continue;
                } else {
                    let mut k = j;
                    j -= 1;

                    while k < ses.len() && ses[k].1 == opposite_op_code {
                        k += 1;
                    }
                    k -= 1;

                    let num_before = j - i + 1;
                    let num_after = k - j;

                    if num_after > 1 {
                        if num_before > num_after {
                            let i2 = i + num_before - num_after;
                            paragraph_ses.extend_from_slice(&ses[i..i2]);
                            i = i2;
                        } else if num_after > num_before {
                            k -= num_after - num_before;
                        }

                        let pairs = self.pair_paragraphs(ses, i, j);
                        paragraph_ses.extend(pairs);
                    } else {
                        paragraph_ses.extend_from_slice(&ses[i..=k]);
                    }
                    i = k;
                }
            }
            i += 1;
        }

        paragraph_ses
    }

    fn pair_paragraphs(
        &self,
        ses: &[(String, Operation)],
        mut i: usize,
        j: usize,
    ) -> Vec<(String, Operation)> {
        let mut pairs = Vec::new();
        let num_pairs = j - i + 1;
        for _ in 0..num_pairs {
            pairs.push(ses[i].clone());
            pairs.push(ses[i + num_pairs].clone());
            i += 1;
        }
        pairs
    }
}
