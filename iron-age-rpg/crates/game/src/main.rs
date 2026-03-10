mod game_state;
mod commands;
mod display;

use std::io::{self, BufRead, Write};

fn main() {
    println!("{}", display::title_screen());

    let args: Vec<String> = std::env::args().collect();
    let load_on_start = args.iter().any(|a| a == "--load");

    let mut state = game_state::GameState::new_game();

    if load_on_start {
        match commands::try_load_game(&mut state) {
            Ok(msg) => println!("{}", msg),
            Err(e)  => {
                println!("{}", e);
                println!("Starting a new game instead.");
                println!("{}", display::intro_text());
            }
        }
    } else {
        println!("{}", display::intro_text());
    }

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
