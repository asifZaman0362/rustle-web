use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub enum EntryType {
    Correct,
    Misplaced,
    Wrong,
}

#[derive(Serialize)]
pub struct Entry {
    letter: char,
    position: usize,
    e_type: EntryType,
}

impl Entry {
    pub fn new(letter: char, position: usize, e_type: EntryType) -> Self {
        Entry {
            letter,
            position,
            e_type,
        }
    }
}

pub type Guess = Vec<Entry>;

pub struct Game {
    pub answer: String,
    pub guesses: Vec<Guess>,
}

impl Game {
    pub fn new(word: &str) -> Self {
        Game {
            answer: word.to_owned(),
            guesses: vec![],
        }
    }

    pub fn submit_guess(&mut self, input: &String) -> Result<Guess, ()> {
        if input.len() != self.answer.len() {
            return Err(())
        }
        let upper = self.answer.to_uppercase().chars().collect::<Vec<char>>();
        let guess = input.to_uppercase().chars().collect::<Vec<char>>();
        let mut matches = vec![];
        let mut mask: Vec<usize> = vec![];
        // first pass, find correct letters and remove them
        let mut filter = vec![];
        for (i, a) in guess.iter().enumerate() {
            let b = upper[i];
            if *a == b {
                filter.push(i);
                matches.push(Entry::new(*a, i, EntryType::Correct));
            }
        }
        for (i, a) in guess.iter().enumerate() {
            if filter.contains(&i) {
                continue;
            }
            let mut found = false;
            for (j, b) in upper.iter().enumerate() {
                if *a == *b && !mask.contains(&j) && !filter.contains(&j)  {
                    matches.push(Entry::new(*a, i, EntryType::Misplaced));
                    mask.push(j);
                    found = true;
                    break;
                } else {
                    found = false;
                }
            }
            if !found {
                matches.push(Entry::new(*a, i, EntryType::Wrong));
            }
        }
        Ok(matches)
    } 
}
