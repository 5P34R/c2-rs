// cli.rs

use rustyline::error::ReadlineError;
use rustyline::Editor;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::task;
use std::collections::HashMap;

pub async fn handle_cli_commands(map: Arc<Mutex<HashMap<usize, mpsc::Sender<String>>>>) {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.clone());

                let map_clone = Arc::clone(&map);
                task::spawn_blocking(move || {
                    let mut parts = line.split_whitespace();
                    
                    match parts.next() {
                        Some("list") => {
                            println!("Connected sessions:");
                            for (id, _) in map_clone.lock().unwrap().iter() {
                                println!(" - Client {}", id);
                            }
                        }
                        Some("send") => {
                            if let (Some(id_str), message) = (parts.next(), parts.collect::<Vec<_>>().join(" ")) {
                                println!("message: {}", message);
                                if let Ok(id) = id_str.parse::<usize>() {
                                    if let Some(sender) = map_clone.lock().unwrap().get(&id).cloned() {
                                        tokio::spawn(async move {
                                            match sender.send(message.to_string()).await {
                                                Ok(()) => println!("Message sent successfully"),
                                                Err(e) => eprintln!("Failed to send message: {:?}", e),
                                            }
                                        });
                                    } else {
                                        println!("Invalid client ID");
                                    }
                                } else {
                                    println!("Invalid client ID format");
                                }
                            } else {
                                println!("Usage: send <client_id> <message>");
                            }
                        }
                        Some("clients") => {
                            println!("Connected sessions:");
                            for (id, _) in map_clone.lock().unwrap().iter() {
                                println!(" - Client {}", id);
                            }
                        }
                        Some("exit") => {
                            println!("Exiting...");
                            std::process::exit(0);
                        }
                        Some(command) => {
                            println!("Unknown command: {}", command);
                        }
                        None => {}
                    }
                })
                .await
                .unwrap();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
}
