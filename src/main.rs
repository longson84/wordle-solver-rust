use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

pub fn load_word_list(file_path: &str) -> Result<Vec<String>, Box<dyn Error>, >{

    let content = fs::read_to_string(file_path)?;

    let re = content.lines().map(String::from).collect();

    Ok(re)
}

fn build_new_word_list(word_list: Vec<String>, guess: String, feedback: Vec<FeedbackResponse>) -> Vec<String>{

    let mut new_word_list = word_list;

    for (i, char) in guess.chars().enumerate() {

        let current_feedback = &feedback[i];

        new_word_list = match current_feedback.result {
            GuessResult::absent => {
                new_word_list.into_iter().filter(|word| !word.contains(char)).collect()
            },
            GuessResult::correct => new_word_list.into_iter().filter(|word| word.chars().nth(i) == Some(char)).collect(),
            GuessResult::present => new_word_list.into_iter().filter(|word| word.contains(char) && word.chars().nth(i) != Some(char)).collect()
        };
    }

    new_word_list

}


async fn guess_and_get_feedback(word_list: &Vec<String>, url: &str, seed: usize, size: usize) -> Result<GuessWithFeedBack, Box<dyn Error>> {

    let mut rng = rand::thread_rng();

    let guess = word_list.choose(&mut rng).unwrap().to_string();

    let params = [("guess", guess.clone()), ("size", size.to_string()), ("seed", seed.to_string())];

    let client = reqwest::Client::new();

    let response = client.get(url).query(&params).send().await?.text().await?;

    let feedback = from_str::<Vec<FeedbackResponse>>(&response).unwrap();

    Ok(GuessWithFeedBack{
        guess,
        feedback
    })
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum GuessResult{
    absent,
    correct,
    present
}

impl Display for GuessResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            GuessResult::absent => "absent",
            GuessResult::correct => "correct",
            GuessResult::present => "present"
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FeedbackResponse {
    slot: usize,
    guess: String,
    result: GuessResult,
}


struct GuessWithFeedBack{
    guess: String,
    feedback: Vec<FeedbackResponse>
}

impl GuessWithFeedBack {

    fn describe_feedback(&self) -> String {

        let mut re = String::new();

        self.feedback.iter().enumerate().for_each(|(i, f)| {
            re.push_str(format!("{}:{}; ", self.guess.chars().nth(i).unwrap(), f.result).as_str())
        });

        re

    }
}

#[tokio::main]
async fn main() {
    let file_path = "src/wordle_list/words";

    let url = "https://wordle.votee.dev:8000/random";

    let mut word_list = load_word_list(file_path).unwrap();

    for turn in 0..6 {

        let re = guess_and_get_feedback(&word_list, url, 123, 5).await.unwrap();

        if re.feedback.iter().filter(|feedback| feedback.result == GuessResult::correct).count() == 5 {
            println!("Your guess in turn {} is correct: {}", 1+ turn, re.guess);
            break
        }
        else {
            println!("Your guess in turn {} is {} and is not correct", 1 + turn, re.guess);
            println!("The feedback is {}\n", re.describe_feedback());
            word_list = build_new_word_list(word_list, re.guess, re.feedback)
        }
    }

}
