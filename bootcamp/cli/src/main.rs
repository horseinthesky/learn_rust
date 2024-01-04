use std::rc::Rc;

mod models;

mod db;
use db::*;

mod ui;

mod io_utils;
use io_utils::*;

mod navigator;
use navigator::*;

fn main() {
    let db = Rc::new(JiraDatabase::new(
        "./data/db.json".to_owned(),
    ));
    let mut navigator = Navigator::new(Rc::clone(&db));

    loop {
        clearscreen::clear().unwrap();

        let cur_page = navigator.get_current_page();
        if cur_page.is_none() {
            break;
        }

        let page = cur_page.unwrap();
        if let Err(error) = page.draw_page() {
            println!(
                "Error rendering page: {}\nPress any key to continue...",
                error
            );
            wait_for_key_press();
        };

        let user_input = get_user_input();

        match page.handle_input(&user_input) {
            Err(error) => {
                println!("Error getting user input: {}\nPress any key to continue...", error);
                wait_for_key_press();
            }
            Ok(opt_action) => {
                if opt_action.is_none() {
                    continue;
                }

                let action = opt_action.unwrap();
                if let Err(error) = navigator.handle_action(action) {
                    println!("Error handling processing user input: {}\nPress any key to continue...", error);
                    wait_for_key_press();
                }
            }
        }
    }
}
