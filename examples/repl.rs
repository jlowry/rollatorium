#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![forbid(unsafe_code)]

use rollatorium::roll;

use std::io::{self, Write};

fn main() {
    println!("Rollatorium REPL. Type a dice expression and press Enter. Ctrl-C to exit.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF (e.g. piped input or Ctrl-D)
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match roll(&trimmed) {
                    Ok(result) => {
                        println!("Result: {}", result.total);
                        println!("Details: {:?}", result);
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                break;
            }
        }
    }
}
