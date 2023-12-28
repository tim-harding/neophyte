use prettytable::Table;
use regex::Regex;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{self, BufRead},
};

#[macro_use]
extern crate prettytable;

fn main() -> anyhow::Result<()> {
    let re = Regex::new(r"^INFO \[([\w:]+)\] EXECUTION_TIME\((\w+)\): (\d+)$")?;
    let mut timings = HashMap::new();
    for line in io::stdin().lock().lines() {
        let line = line?;
        for capture in re.captures_iter(line.as_str()) {
            let (_, [module, function, microseconds]) = capture.extract();
            let microseconds: u128 = microseconds.parse()?;
            let path = format!("{module}::{function}");
            match timings.entry(path) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![microseconds]);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(microseconds);
                }
            }
        }
    }

    let mut timings: Vec<_> = timings
        .into_iter()
        .map(|(path, timings)| (path, Stats::from_timings(timings)))
        .collect();
    timings.sort_unstable_by(|l, r| r.1.cmp(&l.1));

    let mut table = Table::new();
    table.add_row(row![
        "Path", "Total", "Count", "Mean", "Median", "Min", "Max"
    ]);
    for (path, stats) in timings {
        table.add_row(row![
            path,
            stats.total,
            stats.count,
            stats.mean,
            stats.median,
            stats.min,
            stats.max
        ]);
    }
    table.printstd();

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stats {
    pub total: u128,
    pub mean: u128,
    pub median: u128,
    pub count: u128,
    pub min: u128,
    pub max: u128,
}

impl Stats {
    pub fn from_timings(mut timings: Vec<u128>) -> Self {
        timings.sort_unstable();
        let count = timings.len() as u128;
        let total: u128 = timings.iter().cloned().sum();
        let mean = total / count;
        let mid = timings.len() / 2;
        let median = if count % 2 == 0 {
            (timings[mid - 1] + timings[mid]) / 2
        } else {
            timings[mid - 1]
        };
        let min = timings.first().cloned().unwrap();
        let max = timings.last().cloned().unwrap();
        Self {
            total,
            count,
            mean,
            median,
            min,
            max,
        }
    }
}
