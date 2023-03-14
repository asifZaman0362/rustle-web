use serde::Serialize;

#[derive(Serialize)]
pub enum GuessResult {
    MaxGuess(Guess),
    Guess(Guess),
    Won(Guess, u8),
}

#[derive(Serialize)]
pub enum GuessError {
    LengthError,
    NonexistentWordError,
    GuessLimitReached,
    GameOver,
}

#[derive(Serialize, Clone, Copy)]
pub enum EntryType {
    Correct,
    Misplaced,
    Wrong,
}

#[derive(Serialize, Clone, Copy)]
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
    pub win: bool,
}

impl Game {
    pub fn new(word: &str) -> Self {
        Game {
            answer: word.to_owned(),
            guesses: vec![],
            win: false,
        }
    }

    pub fn restart(&mut self, word: &String) {
        self.answer = word.to_string();
        self.guesses.clear();
        self.win = false;
    }

    pub fn submit_guess(&mut self, input: &String) -> Result<GuessResult, GuessError> {
        if self.win {
            return Err(GuessError::GameOver);
        }
        if self.guesses.len() >= 6 {
            return Err(GuessError::GuessLimitReached);
        }
        if input.len() != self.answer.len() {
            return Err(GuessError::LengthError);
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
                if *a == *b && !mask.contains(&j) && !filter.contains(&j) {
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
        self.guesses.push(matches.clone());
        if filter.len() == 5 {
            self.win = true;
            return Ok(GuessResult::Won(matches, self.guesses.len() as u8));
        } else if self.guesses.len() == 6 {
            return Ok(GuessResult::MaxGuess(matches));
        } else {
            return Ok(GuessResult::Guess(matches));
        }
    }
}
