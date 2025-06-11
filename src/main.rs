use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};
use gtk4 as gtk;
use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::Orientation::{Horizontal, Vertical};
use gtk4::{
    gdk, AlertDialog, Align, Box, CssProvider, EventControllerKey, Grid, Label, LinkButton, Widget,
};
use rand::{rng, Rng};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::rc::Rc;

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
const MAX_GUESSES: usize = 6;

/// All lowercase letters
const LOWERCASE: &str = "qwertyuiopasdfghjklzxcvbnm"; // "Typo: In word 'qwertyuiopasdfghjklzxcvbnm'" SHUT UP

/// Keys on the top row of the qwerty keyboard
const KEYBOARD_ROW1: &str = "qwertyuiop";
/// Keys on the middle row of the qwerty keyboard
const KEYBOARD_ROW2: &str = "asdfghjkl";
/// Keys on the bottom row of the qwerty keyboard
const KEYBOARD_ROW3: &str = "zxcvbnm";

/// Check a guess
fn get_guess_status(
    guess: [char; WORD_LENGTH],
    target_slice: &str,
    output: &mut [usize; WORD_LENGTH],
    letter_states: &mut HashMap<char, usize>, // grays: &mut Vec<char>
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
            letter_states.insert(c, COLOR_GREEN);
        }
    }
    for i in 0..WORD_LENGTH {
        let c: char = guess[i];
        if target.chars().nth(i) != Some(c) {
            if green_count_map[&c] + yellow_count_map[&c] < total_count_map[&c] {
                output[i] = COLOR_YELLOW;
                *yellow_count_map.get_mut(&c).unwrap() += 1;
                if letter_states[&c] < COLOR_YELLOW {
                    letter_states.insert(c, COLOR_YELLOW);
                }
            } else {
                output[i] = COLOR_GRAY;
                if !target.contains(c) && letter_states[&c] < COLOR_GRAY {
                    letter_states.insert(c, COLOR_GRAY);
                }
            }
        }
    }
    return wins == WORD_LENGTH;
}

/// Update the colors & letters of the board
fn update_board(
    board_chars: [[char; WORD_LENGTH]; MAX_GUESSES],
    board_colors: [[usize; WORD_LENGTH]; MAX_GUESSES],
    cur_row: usize,
    cur_col: usize,
    grid_ref: Ref<Grid>,
) {
    for row in 0..MAX_GUESSES {
        for chr in 0..WORD_LENGTH {
            let mut c: char = board_chars[row][chr];
            let color: usize = board_colors[row][chr];
            let w: Widget = (*grid_ref).child_at(chr as i32, row as i32).unwrap();
            let l: Label = w.downcast::<Label>().ok().unwrap();
            l.remove_css_class("green");
            l.remove_css_class("yellow");
            l.remove_css_class("gray");
            l.remove_css_class("cursor");
            if color == COLOR_GREEN {
                l.add_css_class("green");
            } else if color == COLOR_YELLOW {
                l.add_css_class("yellow");
            } else if color == COLOR_GRAY {
                l.add_css_class("gray");
            } else if row == cur_row && chr == cur_col {
                l.add_css_class("cursor");
                c = '_';
            }
            l.set_text(&*c.to_string().to_uppercase());
        }
    }
}

/// Update the colors on a row of the keyboard
fn update_keyboard_row(letter_states: HashMap<char, usize>, keyboard_row: Box, chars: &str) {
    let mut child = keyboard_row.first_child().expect("Keyboard key missing!");

    for c in chars.chars() {
        if let Ok(l) = child.clone().downcast::<Label>() {
            if let Some(color) = letter_states.get(&c) {
                l.remove_css_class("green");
                l.remove_css_class("yellow");
                l.remove_css_class("gray");
                l.remove_css_class("cursor");
                if *color == COLOR_GREEN {
                    l.add_css_class("green");
                } else if *color == COLOR_YELLOW {
                    l.add_css_class("yellow");
                } else if *color == COLOR_GRAY {
                    l.add_css_class("gray");
                }
            }
        }
        let next_child = child.next_sibling();
        if next_child.is_some() {
            child = next_child.unwrap();
        } else {
            break;
        }
    }
}

/// Update the colors on the keyboard
fn update_keyboard(
    letter_states: HashMap<char, usize>,
    keyboard_row1: Box,
    keyboard_row2: Box,
    keyboard_row3: Box,
) {
    update_keyboard_row(letter_states.clone(), keyboard_row1, KEYBOARD_ROW1);
    update_keyboard_row(letter_states.clone(), keyboard_row2, KEYBOARD_ROW2);
    update_keyboard_row(letter_states.clone(), keyboard_row3, KEYBOARD_ROW3);
}

/// Create a row of keys
fn build_keyboard_row(keys: &str) -> Box {
    let keyboard_row: Box = Box::new(Horizontal, 4);
    keyboard_row.set_halign(Align::Center);
    for c in keys.chars() {
        let key: Label = Label::builder().build();
        key.set_text(&*c.to_uppercase().to_string());
        key.set_size_request(40, 60);
        key.add_css_class("key");
        keyboard_row.append(&key);
    }
    return keyboard_row;
}

fn main() -> glib::ExitCode {
    if !std::fs::exists("words.txt").expect("Failed to check if file exists") {
        println!("words.txt not found!");
        return glib::ExitCode::FAILURE;
    }

    let app: Application = Application::builder()
        .application_id("dev.droc101.rustle")
        .build();

    app.connect_startup(|_app| {
        let provider: CssProvider = CssProvider::new();
        provider.load_from_string(include_str!("style.css"));

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let settings = gtk::Settings::default().unwrap();
        if settings.is_gtk_application_prefer_dark_theme() {
            let dark_provider = CssProvider::new();
            dark_provider.load_from_string(include_str!("style.dark.css"));
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Could not connect to a display."),
                &dark_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });

    app.connect_activate(|app| {
        let lowercase_letters: String = String::from(LOWERCASE);
        let words: Vec<String> = read_to_string("words.txt")
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        let answers: Vec<String> = read_to_string("answers.txt")
            .unwrap()
            .lines()
            .map(String::from)
            .collect();
        let board_chars: [[char; WORD_LENGTH]; MAX_GUESSES] = [[' '; WORD_LENGTH]; MAX_GUESSES];
        let board_colors: [[usize; WORD_LENGTH]; MAX_GUESSES] =
            [[COLOR_UNSET; WORD_LENGTH]; MAX_GUESSES];
        let mut letter_states: HashMap<char, usize> = HashMap::new();
        let cur_x: usize = 0;
        let guess: usize = 0;
        let answer_index: usize = rng().random_range(0..answers.len());
        let answer: String = answers[answer_index].clone();
        let locked: bool = false;

        for c in LOWERCASE.chars() {
            letter_states.insert(c, COLOR_UNSET);
        }

        let window: ApplicationWindow = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(750)
            .title("Rustle!")
            .resizable(false)
            .build();

        let outer_box: Box = Box::new(Horizontal, 6);
        outer_box.set_halign(Align::Center);

        let main_box: Box = Box::new(Vertical, 6);
        main_box.set_vexpand(true);
        main_box.set_margin_top(10);

        let title: Label = Label::builder().build();
        title.set_text("Rustle!");
        title.add_css_class("title");
        title.set_margin_bottom(10);
        main_box.append(&title);

        let grid_box: Box = Box::new(Horizontal, 6);
        grid_box.set_halign(Align::Center);

        let grid: Grid = Grid::builder().build();
        grid.set_column_homogeneous(true);
        grid.set_row_homogeneous(true);
        grid.set_column_spacing(4);
        grid.set_row_spacing(4);
        grid.set_hexpand(false);
        for _ in 0..MAX_GUESSES {
            grid.insert_row(0);
            for _ in 0..WORD_LENGTH {
                grid.insert_column(0);
            }
        }
        for y in 0i32..MAX_GUESSES as i32 {
            for x in 0i32..WORD_LENGTH as i32 {
                let label: Label = Label::builder().build();
                label.set_size_request(60, 60);
                label.add_css_class("tile");
                label.set_text("");
                grid.attach(&label, x, y, 1, 1);
            }
        }
        grid_box.append(&grid);
        main_box.append(&grid_box);

        let keyboard_row_1: Box = build_keyboard_row(KEYBOARD_ROW1);
        keyboard_row_1.set_margin_top(50);
        main_box.append(&keyboard_row_1);

        let keyboard_row_2: Box = build_keyboard_row(KEYBOARD_ROW2);
        main_box.append(&keyboard_row_2);

        let keyboard_row_3: Box = build_keyboard_row(KEYBOARD_ROW3);
        main_box.append(&keyboard_row_3);

        let new_game: LinkButton = LinkButton::builder().label("New Game").build();
        new_game.add_css_class("new_game");
        new_game.set_visible(false);
        main_box.append(&new_game);

        outer_box.append(&main_box);
        window.set_child(Some(&outer_box));

        //#region refcells
        let grid_rc: Rc<RefCell<Grid>> = Rc::new(RefCell::new(grid.clone()));
        let new_game_rc: Rc<RefCell<LinkButton>> = Rc::new(RefCell::new(new_game.clone()));
        let keyboard_row_1_rc: Rc<RefCell<Box>> = Rc::new(RefCell::new(keyboard_row_1.clone()));
        let keyboard_row_2_rc: Rc<RefCell<Box>> = Rc::new(RefCell::new(keyboard_row_2.clone()));
        let keyboard_row_3_rc: Rc<RefCell<Box>> = Rc::new(RefCell::new(keyboard_row_3.clone()));
        let letter_states_rc: Rc<RefCell<HashMap<char, usize>>> =
            Rc::new(RefCell::new(letter_states.clone()));
        let window_rc: Rc<RefCell<ApplicationWindow>> = Rc::new(RefCell::new(window.clone()));
        let cur_x_rc: Rc<RefCell<usize>> = Rc::new(RefCell::new(cur_x));
        let board_chars_rc: Rc<RefCell<[[char; WORD_LENGTH]; MAX_GUESSES]>> =
            Rc::new(RefCell::new(board_chars));
        let board_colors_rc: Rc<RefCell<[[usize; WORD_LENGTH]; MAX_GUESSES]>> =
            Rc::new(RefCell::new(board_colors));
        let guess_rc: Rc<RefCell<usize>> = Rc::new(RefCell::new(guess));
        let locked_rc: Rc<RefCell<bool>> = Rc::new(RefCell::new(locked));
        let answer_rc: Rc<RefCell<String>> = Rc::new(RefCell::new(answer));
        let answers_rc: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(answers));

        let grid_rc_2: Rc<RefCell<Grid>> = grid_rc.clone();
        let new_game_rc_2: Rc<RefCell<LinkButton>> = new_game_rc.clone();
        let keyboard_row_1_rc_2: Rc<RefCell<Box>> = keyboard_row_1_rc.clone();
        let keyboard_row_2_rc_2: Rc<RefCell<Box>> = keyboard_row_2_rc.clone();
        let keyboard_row_3_rc_2: Rc<RefCell<Box>> = keyboard_row_3_rc.clone();
        let letter_states_rc_2: Rc<RefCell<HashMap<char, usize>>> = letter_states_rc.clone();
        let window_rc_2: Rc<RefCell<ApplicationWindow>> = window_rc.clone();
        let cur_x_rc_2: Rc<RefCell<usize>> = cur_x_rc.clone();
        let board_chars_rc_2: Rc<RefCell<[[char; WORD_LENGTH]; MAX_GUESSES]>> =
            board_chars_rc.clone();
        let board_colors_rc_2: Rc<RefCell<[[usize; WORD_LENGTH]; MAX_GUESSES]>> =
            board_colors_rc.clone();
        let guess_rc_2: Rc<RefCell<usize>> = guess_rc.clone();
        let locked_rc_2: Rc<RefCell<bool>> = locked_rc.clone();
        let answer_rc_2: Rc<RefCell<String>> = answer_rc.clone();

        let answer_index_rc: Rc<RefCell<usize>> = Rc::new(RefCell::new(answer_index));

        //#endregion

        update_board(board_chars, board_colors, guess, cur_x, grid_rc.borrow());

        new_game.connect_activate_link(move |_| {
            let new_game_val: Ref<LinkButton> = new_game_rc_2.borrow();
            let mut locked_val: RefMut<bool> = locked_rc_2.borrow_mut();
            let mut cur_x_val: RefMut<usize> = cur_x_rc_2.borrow_mut();
            let mut board_chars_val: RefMut<[[char; WORD_LENGTH]; MAX_GUESSES]> =
                board_chars_rc_2.borrow_mut();
            let mut board_colors_val: RefMut<[[usize; WORD_LENGTH]; MAX_GUESSES]> =
                board_colors_rc_2.borrow_mut();
            let mut guess_val: RefMut<usize> = guess_rc_2.borrow_mut();
            let mut letter_states_val: RefMut<HashMap<char, usize>> =
                letter_states_rc_2.borrow_mut();
            let grid_val: Ref<Grid> = grid_rc_2.borrow();
            let keyboard_row_1_val: Ref<Box> = keyboard_row_1_rc_2.borrow();
            let keyboard_row_2_val: Ref<Box> = keyboard_row_2_rc_2.borrow();
            let keyboard_row_3_val: Ref<Box> = keyboard_row_3_rc_2.borrow();
            let window_val: Ref<ApplicationWindow> = window_rc_2.borrow();
            let mut answer_index_val: RefMut<usize> = answer_index_rc.borrow_mut();
            let mut answer_val: RefMut<String> = answer_rc.borrow_mut();
            let answers_val: Ref<Vec<String>> = answers_rc.borrow();
            new_game_val.set_visible(false);
            for i in 0..MAX_GUESSES {
                for j in 0..WORD_LENGTH {
                    board_chars_val[i][j] = ' ';
                    board_colors_val[i][j] = COLOR_UNSET;
                }
            }
            for c in LOWERCASE.chars() {
                letter_states_val.insert(c, COLOR_UNSET);
            }

            update_board(
                *board_chars_val,
                *board_colors_val,
                *guess_val,
                *cur_x_val,
                grid_val,
            );
            update_keyboard(
                letter_states_val.clone(),
                keyboard_row_1_val.clone(),
                keyboard_row_2_val.clone(),
                keyboard_row_3_val.clone(),
            );

            window_val.grab_focus();

            *locked_val = false;
            *cur_x_val = 0;
            *guess_val = 0;

            *answer_index_val = rng().random_range(0..answers_val.len());
            *answer_val = answers_val[*answer_index_val].clone();

            return Propagation::Stop;
        });

        let k: EventControllerKey = EventControllerKey::builder().build();
        k.connect_key_pressed(move |_, k: Key, _, _| {
            let mut cur_x_val: RefMut<usize> = cur_x_rc.borrow_mut();
            let mut board_chars_val: RefMut<[[char; WORD_LENGTH]; MAX_GUESSES]> =
                board_chars_rc.borrow_mut();
            let mut board_colors_val: RefMut<[[usize; WORD_LENGTH]; MAX_GUESSES]> =
                board_colors_rc.borrow_mut();
            let grid_val: Ref<Grid> = grid_rc.borrow();
            let new_game_val: Ref<LinkButton> = new_game_rc.borrow();
            let keyboard_row_1_val: Ref<Box> = keyboard_row_1_rc.borrow();
            let keyboard_row_2_val: Ref<Box> = keyboard_row_2_rc.borrow();
            let keyboard_row_3_val: Ref<Box> = keyboard_row_3_rc.borrow();
            let window_val: Ref<ApplicationWindow> = window_rc.borrow();
            let mut guess_val: RefMut<usize> = guess_rc.borrow_mut();
            let mut letter_states_val: RefMut<HashMap<char, usize>> = letter_states_rc.borrow_mut();
            let mut locked_val: RefMut<bool> = locked_rc.borrow_mut();
            let answer_val: Ref<String> = answer_rc_2.borrow();

            if *locked_val {
                return Propagation::Proceed;
            }

            if k == Key::BackSpace {
                if *cur_x_val != 0 {
                    board_chars_val[*guess_val][*cur_x_val - 1] = ' ';
                    *cur_x_val -= 1;
                }
                update_board(
                    *board_chars_val,
                    *board_colors_val,
                    *guess_val,
                    *cur_x_val,
                    grid_val,
                );
                update_keyboard(
                    letter_states_val.clone(),
                    keyboard_row_1_val.clone(),
                    keyboard_row_2_val.clone(),
                    keyboard_row_3_val.clone(),
                );
                return Propagation::Stop;
            } else if k == Key::Return || k == Key::KP_Enter {
                if *cur_x_val == WORD_LENGTH {
                    let guess_str: String = String::from_iter(board_chars_val[*guess_val].iter());
                    if words.contains(&guess_str) {
                        let winner: bool = get_guess_status(
                            board_chars_val[*guess_val],
                            answer_val.as_str(),
                            &mut board_colors_val[*guess_val],
                            &mut *letter_states_val,
                        );
                        if winner {
                            update_board(
                                *board_chars_val,
                                *board_colors_val,
                                *guess_val,
                                *cur_x_val,
                                grid_val,
                            );
                            update_keyboard(
                                letter_states_val.clone(),
                                keyboard_row_1_val.clone(),
                                keyboard_row_2_val.clone(),
                                keyboard_row_3_val.clone(),
                            );
                            *locked_val = true;
                            new_game_val.set_visible(true);
                            let message: String = format!("Guessed in {} tries", *guess_val + 1);
                            AlertDialog::builder()
                                .message("You Win!")
                                .detail(message)
                                .build()
                                .show(Some(&*window_val));
                            return Propagation::Stop;
                        } else if *guess_val == MAX_GUESSES - 1 {
                            update_board(
                                *board_chars_val,
                                *board_colors_val,
                                *guess_val,
                                *cur_x_val,
                                grid_val,
                            );
                            update_keyboard(
                                letter_states_val.clone(),
                                keyboard_row_1_val.clone(),
                                keyboard_row_2_val.clone(),
                                keyboard_row_3_val.clone(),
                            );
                            let message: String = format!("The word was \"{}\"", *answer_val);
                            AlertDialog::builder()
                                .detail(message)
                                .message("You Lose!")
                                .build()
                                .show(Some(&*window_val));
                            *locked_val = true;
                            new_game_val.set_visible(true);
                            return Propagation::Stop;
                        }
                        *guess_val += 1;
                        *cur_x_val = 0;
                        update_board(
                            *board_chars_val,
                            *board_colors_val,
                            *guess_val,
                            *cur_x_val,
                            grid_val,
                        );
                        update_keyboard(
                            letter_states_val.clone(),
                            keyboard_row_1_val.clone(),
                            keyboard_row_2_val.clone(),
                            keyboard_row_3_val.clone(),
                        );
                    }
                }
                return Propagation::Stop;
            } else {
                if *cur_x_val + 1 <= WORD_LENGTH {
                    let unicode: Option<char> = k.to_unicode();
                    if unicode.is_some() {
                        let mut chr: String = String::from(unicode.unwrap());
                        chr = chr.to_lowercase();
                        if chr.len() == 1 && lowercase_letters.contains(chr.as_str()) {
                            board_chars_val[*guess_val][*cur_x_val] = chr.pop().unwrap();
                            *cur_x_val += 1; // @rust you stupid language add ++ operator
                            update_board(
                                *board_chars_val,
                                *board_colors_val,
                                *guess_val,
                                *cur_x_val,
                                grid_val,
                            );
                            update_keyboard(
                                letter_states_val.clone(),
                                keyboard_row_1_val.clone(),
                                keyboard_row_2_val.clone(),
                                keyboard_row_3_val.clone(),
                            );
                            return Propagation::Stop;
                        }
                    }
                }
            }
            return Propagation::Proceed;
        });
        window.add_controller(k);

        window.present();
    });

    return app.run();
}
