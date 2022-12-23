use std::collections::{HashMap, HashSet};

pub fn one2() -> eyre::Result<u64> {
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input.txt")?;
    let elves: Vec<&str> = input.split("\n\n").collect();
    let mut counts: Vec<u64> = elves
        .into_iter()
        .map(|elf_log| {
            elf_log
                .split("\n")
                .collect::<Vec<&str>>()
                .into_iter()
                .flat_map(|cal| cal.parse())
                .collect::<Vec<u64>>()
                .into_iter()
                .sum()
        })
        .collect();

    // Sort counts.
    counts.sort();
    counts.reverse();

    // Get sum of top-three elves.
    let top_three_sum: u64 = counts.into_iter().take(3).sum();

    Ok(top_three_sum)
}

#[derive(Debug)]
enum Move {
    Rock,
    Paper,
    Scissors,
}

// Turns a move into a respective point.
impl From<&Move> for u64 {
    fn from(m: &Move) -> Self {
        match m {
            Move::Rock => 1,
            Move::Paper => 2,
            Move::Scissors => 3,
        }
    }
}

#[derive(Debug)]
enum Outcome {
    Win,
    Lose,
    Draw,
}

// Turns an outcome into a respective point.
impl From<Outcome> for u64 {
    fn from(o: Outcome) -> Self {
        match o {
            Outcome::Lose => 0,
            Outcome::Draw => 3,
            Outcome::Win => 6,
        }
    }
}

impl From<&str> for Move {
    fn from(s: &str) -> Self {
        match s {
            "A" | "X" => Move::Rock,
            "B" | "Y" => Move::Paper,
            "C" | "Z" => Move::Scissors,
            _ => panic!("no move matches input string"),
        }
    }
}

fn player_outcome(player: &Move, opp: &Move) -> Outcome {
    use Move::*;
    use Outcome::*;
    match (player, opp) {
        // Rock moves.
        (Rock, Rock) => Draw,
        (Rock, Paper) => Lose,
        (Rock, Scissors) => Win,
        // Paper moves.
        (Paper, Rock) => Win,
        (Paper, Paper) => Draw,
        (Paper, Scissors) => Lose,
        // Scissor moves.
        (Scissors, Rock) => Lose,
        (Scissors, Paper) => Win,
        (Scissors, Scissors) => Draw,
    }
}

fn compute_round_scores(rounds: &Vec<Vec<Move>>) -> Vec<u64> {
    rounds
        .into_iter()
        .map(|round| {
            let opp = round.first().unwrap();
            let player = round.last().unwrap();
            let move_score = u64::from(player);
            let round_score = u64::from(player_outcome(player, opp));
            move_score + round_score
        })
        .collect()
}

pub fn two1() -> eyre::Result<u64> {
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input2.txt")?;
    let rounds: Vec<Vec<Move>> = input
        .trim()
        .split("\n")
        .map(|round| {
            round
                .split(" ")
                .collect::<Vec<&str>>()
                .into_iter()
                .map(Move::from)
                .collect::<Vec<Move>>()
        })
        .collect();

    let total_round_scores: u64 = compute_round_scores(&rounds).into_iter().sum();
    Ok(total_round_scores)
}

pub fn two2() -> eyre::Result<u64> {
    todo!()
}

pub fn three1() -> eyre::Result<u64> {
    let alphab = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let priorities: HashMap<&str, u64> = alphab
        .split("")
        .enumerate()
        .map(|(i, s)| (s, i as u64))
        .collect();

    let i = 0;
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input3.txt")?;
    let results: Vec<u64> = input
        .trim()
        .split("\n")
        .map(|example| {
            let midpoint = example.len() / 2;
            let parts = example.split_at(midpoint);
            let left: HashSet<&str> = parts.0.split("").into_iter().collect();
            let right: HashSet<&str> = parts.1.split("").into_iter().collect();

            let mut intersect: Vec<&&str> =
                left.intersection(&right).filter(|s| **s != "").collect();

            assert!(intersect.len() == 1);
            let in_common = *intersect.pop().unwrap();
            *priorities.get(in_common).unwrap()
        })
        .collect();
    let total: u64 = results.into_iter().sum();
    Ok(total)
}

pub fn three2() -> eyre::Result<u64> {
    let alphab = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let priorities: HashMap<&str, u64> = alphab
        .split("")
        .enumerate()
        .map(|(i, s)| (s, i as u64))
        .collect();

    let input = std::fs::read_to_string("/home/zodiark/Downloads/input3.txt")?;
    let results: Vec<u64> = input
        .trim()
        .split("\n")
        .collect::<Vec<&str>>()
        .chunks(3)
        .map(|elem| {
            assert!(elem.len() == 3);
            let first: HashSet<&str> = elem[0].split("").into_iter().collect();
            let second: HashSet<&str> = elem[1].split("").into_iter().collect();
            let third: HashSet<&str> = elem[2].split("").into_iter().collect();

            let intersect: HashSet<&str> = first.intersection(&second).map(|s| *s).collect();

            let mut badge: Vec<&str> = intersect
                .intersection(&third)
                .map(|s| *s)
                .filter(|s| *s != "")
                .collect();

            assert!(badge.len() == 1);
            let badge_letter = badge.pop().unwrap();
            *priorities.get(badge_letter).unwrap()
        })
        .collect();

    let total: u64 = results.into_iter().sum();
    Ok(total)
}

fn transform_example(example: &str) -> Vec<(u64, u64)> {
    example
        .split(",")
        .map(|example| {
            let str_range: Vec<u64> = example
                .split("-")
                .map(|s| s.parse::<u64>().unwrap())
                .collect();
            assert!(str_range.len() == 2);
            let lower = str_range[0];
            let upper = str_range[1];
            (lower, upper)
        })
        .collect()
}

fn first_contains_second(ranges: &Vec<(u64, u64)>) -> bool {
    assert!(ranges.len() == 2);
    let first = ranges[0];
    let second = ranges[1];
    first.0 <= second.0 && first.1 >= second.1
}

fn is_fully_contained(ranges: &mut Vec<(u64, u64)>) -> bool {
    let first = first_contains_second(&ranges);
    ranges.reverse();
    let second = first_contains_second(&ranges);
    return first || second;
}

pub fn four1() -> eyre::Result<u64> {
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input4.txt")?;
    let results: Vec<bool> = input
        .trim()
        .split("\n")
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|s| {
            let mut example = transform_example(s);
            is_fully_contained(&mut example)
        })
        .collect();

    let num_contained = results.into_iter().filter(|x| *x).count() as u64;
    Ok(num_contained)
}

pub fn six1() -> eyre::Result<u64> {
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input6.txt")?;
    let marker = determine_marker_index(&input, 4);
    Ok(marker as u64)
}

pub fn six2() -> eyre::Result<u64> {
    let input = std::fs::read_to_string("/home/zodiark/Downloads/input6.txt")?;
    let marker = determine_marker_index(&input, 14);
    Ok(marker as u64)
}

fn determine_marker_index(input: &str, window_size: usize) -> u64 {
    let example: Vec<&str> = input.trim().split("").filter(|s| *s != "").collect();

    let windows = example.windows(14);
    let mut marker = 0;
    for (idx, window) in windows.enumerate() {
        let uniq: HashSet<&str> = window.into_iter().map(|s| *s).collect();
        if uniq.len() == 14 {
            marker = idx + 14;
            break;
        }
    }
    marker as u64
}
