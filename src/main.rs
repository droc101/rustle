use getch_rs::{Getch, Key};
use std::collections::HashMap;

/// The color for a letter that has not yet been checked
const COLOR_UNSET: usize = 0;
/// The color for a letter that is not in the word
const COLOR_GRAY: usize = 1;
/// The color for a letter that is in the word, but not at that position
const COLOR_YELLOW: usize = 2;
// The color for a letter that is in the word at that position
const COLOR_GREEN: usize = 3;

/// The length of the word / width of the board
const WORD_LENGTH: usize = 5;
/// The number of allowed guesses / height of the board
const MAX_GUESSES: usize = 5;

static LOWERCASE: &str = "qwertyuiopasdfghjklzxcvbnm";

/// Draws a row separator for the board
fn draw_board_separator(row: usize) {
    if row == 0 {
        print!("╔");
    } else if row == MAX_GUESSES {
        print!("╚");
    } else {
        print!("╠");
    }
    for col in 0..(WORD_LENGTH * 2 - 1) {
        if col & 1 != 0 {
            if row == 0 {
                print!("╦");
            } else if row == MAX_GUESSES {
                print!("╩");
            } else {
                print!("╬");
            }
        } else {
            print!("═");
        }
    }
    if row == 0 {
        print!("╗");
    } else if row == MAX_GUESSES {
        print!("╝");
    } else {
        print!("╣");
    }
    print!("\n");
}

/// Draw the board
fn draw_board(
    board_chars: [[char; WORD_LENGTH]; MAX_GUESSES],
    board_colors: [[usize; WORD_LENGTH]; MAX_GUESSES],
    cur_row: usize,
    cur_col: usize,
) {
    const COLORS: [&str; 4] = ["\x1b[0m", "\x1b[37;40m", "\x1b[30;43m", "\x1b[30;42m"];

    print!("\x1b[H\x1b[J\x1b[H");

    for row in 0..MAX_GUESSES {
        draw_board_separator(row);
        for chr in 0..WORD_LENGTH {
            let mut c: char = board_chars[row][chr];
            if row == cur_row && chr == cur_col {
                c = '_';
            }
            print!(
                "\x1b[0m║\x1b[1m{}{}\x1b[0m",
                COLORS[board_colors[row][chr]], c
            );
        }
        print!("║\n");
    }
    draw_board_separator(MAX_GUESSES);
}

/// Check a guess
fn get_guess_status(
    guess: [char; WORD_LENGTH],
    target_slice: &str,
    output: &mut [usize; WORD_LENGTH],
) -> bool {
    let target: String = target_slice.chars().collect();
    let lowercase_letters: String = String::from(LOWERCASE);
    let mut wins: usize = 0;
    let mut green_count_map: HashMap<char, usize> = HashMap::new();
    let mut yellow_count_map: HashMap<char, usize> = HashMap::new();
    let mut total_count_map: HashMap<char, usize> = HashMap::new();
    for c in lowercase_letters.chars() {
        green_count_map.insert(c, 0);
        yellow_count_map.insert(c, 0);
        total_count_map.insert(c, target.chars().filter(|&ch| ch == c).count());
    }
    for i in 0..WORD_LENGTH {
        let c: char = guess[i];
        if target.chars().nth(i) == Some(c) {
            output[i] = COLOR_GREEN;
            wins += 1;
            *green_count_map.get_mut(&c).unwrap() += 1;
        }
    }
    for i in 0..WORD_LENGTH {
        let c: char = guess[i];
        if target.chars().nth(i) != Some(c) {
            if green_count_map[&c] + yellow_count_map[&c] < total_count_map[&c] {
                output[i] = COLOR_YELLOW;
                *yellow_count_map.get_mut(&c).unwrap() += 1;
            } else {
                output[i] = COLOR_GRAY;
            }
        }
    }
    return wins == WORD_LENGTH;
}

fn main() {
    let lowercase_letters: String = String::from(LOWERCASE); // "Typo: In word 'qwertyuiopasdfghjklzxcvbnm'" SHUT UP
    let g: Getch = Getch::new();
    let word: String = String::from("apple");
    let mut board_chars: [[char; WORD_LENGTH]; MAX_GUESSES] = [[' '; WORD_LENGTH]; MAX_GUESSES];
    let mut board_colors: [[usize; WORD_LENGTH]; MAX_GUESSES] =
        [[COLOR_UNSET; WORD_LENGTH]; MAX_GUESSES];
    let mut cur_x: usize = 0;
    let mut guess: usize = 0;

    print!("\x1b[?25l"); // Steal cursor

    draw_board(board_chars, board_colors, guess, cur_x);

    loop {
        match g.getch() {
            Ok(Key::Ctrl('c')) => break,
            Ok(Key::Delete) => {
                if cur_x != 0 {
                    board_chars[guess][cur_x - 1] = ' ';
                    cur_x -= 1;
                }
            }
            Ok(Key::Char('\r')) => {
                // enter key
                if cur_x == WORD_LENGTH {
                    let winner: bool = get_guess_status(
                        board_chars[guess],
                        word.as_str(),
                        &mut board_colors[guess],
                    );
                    if winner {
                        draw_board(board_chars, board_colors, guess, cur_x);
                        println!("you're winner");
                        break;
                    } else if guess == MAX_GUESSES - 1 {
                        draw_board(board_chars, board_colors, guess, cur_x);
                        println!("you're LOSER");
                        break;
                    }
                    guess += 1;
                    cur_x = 0;
                }
            }
            Ok(key) => {
                if cur_x + 1 <= WORD_LENGTH {
                    // Is this a crime? Yeah, probably. Am I going to fix it? Probably not.
                    let mut chr = format!("{:?}", key);
                    if chr.contains("Char('") {
                        chr = chr.replace("Char('", "");
                        chr = chr.replace("')", "");
                        chr = chr.to_ascii_lowercase();

                        if chr.len() == 1 && lowercase_letters.contains(chr.as_str()) {
                            board_chars[guess][cur_x] = chr.pop().unwrap();
                            cur_x += 1; // @rust you stupid language add ++ operator
                        }
                    }
                }
            }
            Err(e) => println!("{:?}", e),
        }
        draw_board(board_chars, board_colors, guess, cur_x);
    }

    print!("\x1b[?25h"); // Return the cursor to the user
}
