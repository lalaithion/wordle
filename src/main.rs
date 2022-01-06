use indicatif::ProgressIterator;
use rayon::prelude::*;

use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::str;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Word {
    word: [u8; 5],
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = str::from_utf8(&self.word).expect("word contains invalid characters!");
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
struct ScoredWord {
    score: f32,
    word: Word,
}

impl fmt::Display for ScoredWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.word, self.score)
    }
}

fn green(guess: Word, answer: Word) -> f32 {
    guess
        .word
        .into_iter()
        .zip(answer.word.into_iter())
        .map(|(c, d)| if c == d { 1.0 } else { 0.0 })
        .sum()
}

fn yellow(guess: Word, answer: Word) -> f32 {
    let mut hashmap: [u8; 5] /* lol */ = [0,0,0,0,0];

    guess
        .word
        .into_iter()
        .enumerate()
        .filter(|(i, c)| {
            let r = !hashmap.contains(c);
            hashmap[*i] = *c;
            r
        })
        .filter(|(_, c)| answer.word.contains(&c))
        .count() as f32
}

fn color_averages(words: Vec<Word>) {
    let mut greens: Vec<ScoredWord> = Vec::with_capacity(words.len());
    let mut yellows: Vec<ScoredWord> = Vec::with_capacity(words.len());

    for guess in words.iter() {
        let green_sum: f32 = words.par_iter().map(|answer| green(*guess, *answer)).sum();
        let yellow_sum: f32 = words.par_iter().map(|answer| yellow(*guess, *answer)).sum();
        greens.push(ScoredWord {
            word: *guess,
            score: green_sum / (words.len() as f32),
        });
        yellows.push(ScoredWord {
            word: *guess,
            score: yellow_sum / (words.len() as f32),
        });
    }

    let mut totals: Vec<ScoredWord> = greens
        .par_iter()
        .zip(yellows.par_iter())
        .map(|(g, y)| ScoredWord {
            word: g.word,
            score: g.score + y.score,
        })
        .collect();

    greens.par_sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());
    yellows.par_sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());
    totals.par_sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());

    println!("Best overall:");
    for x in totals.iter().rev().take(10) {
        println!("\t{}", x)
    }
    println!("Best for greens:");
    for x in greens.iter().rev().take(10) {
        println!("\t{}", x)
    }
    println!("Best for yellows:");
    for x in yellows.iter().rev().take(10) {
        println!("\t{}", x)
    }

    println!("Worst overall:");
    for x in totals.iter().take(10) {
        println!("\t{}", x)
    }
    println!("Worst for greens:");
    for x in greens.iter().take(10) {
        println!("\t{}", x)
    }
    println!("Worst for yellows:");
    for x in yellows.iter().take(10) {
        println!("\t{}", x)
    }
}

fn num_left(words: &[Word], guess: Word, answer: Word) -> usize {
    words
        .par_iter()
        .filter(|w| {
            guess
                .word
                .iter()
                .filter(|x| answer.word.contains(x))
                .all(|x| w.word.contains(x))
        })
        .filter(|w| {
            guess
                .word
                .iter()
                .zip(answer.word.iter())
                .zip(w.word.iter())
                .all(|((g, a), w)| if g == a { g == w } else { true })
        })
        .count()
}

fn inference(words: Vec<Word>) {
    let mut avg_remaining: Vec<ScoredWord> = Vec::with_capacity(words.len());
    for guess in words.iter().progress() {
        let remaining_sum: usize = words
            .par_iter()
            .map(|answer| num_left(&words, *guess, *answer))
            .sum();
        avg_remaining.push(ScoredWord {
            word: *guess,
            score: remaining_sum as f32 / words.len() as f32,
        })
    }
    avg_remaining.par_sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());
    println!("Best remaining:");
    for x in avg_remaining.iter().rev().take(10) {
        println!("\t{}", x)
    }

    println!("Worst remaining:");
    for x in avg_remaining.iter().take(10) {
        println!("\t{}", x)
    }
}

fn main() -> io::Result<()> {
    let data = io::BufReader::new(File::open("words.txt")?);

    let mut words: Vec<Word> = Vec::with_capacity(1000);

    for line in data.lines() {
        if let Ok(line) = line {
            let s = line.trim();
            if s.len() == 5 && s.is_ascii() {
                words.push(Word {
                    word: s
                        .as_bytes()
                        .try_into()
                        .expect("unable to convert 5-element ascii string into bytes"),
                })
            }
        }
    }

    // Takes 3 seconds on my computer
    color_averages(words);

    // Takes ~1 hour on my computer
    // inference(words);

    Ok(())
}
