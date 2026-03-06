mod game_state;
mod commands;
mod display;

use std::io::{self, BufRead, Write};

fn main() {
    println!("{}", display::title_screen());

    let mut state = game_state::GameState::new_game();
    println!("{}", display::intro_text());
    println!("\n{}", display::location_display(state.world.current_location().unwrap()));

    let stdin = io::stdin();
    loop {
        print!("\n> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }
        let input = line.trim().to_string();
        if input.is_empty() { continue; }

        match commands::handle_command(&input, &mut state) {
            commands::CommandResult::Message(msg) => println!("{}", msg),
            commands::CommandResult::Quit => {
                println!("Farewell, traveller. May your blade stay sharp.");
                break;
            }
        }
    }
}
