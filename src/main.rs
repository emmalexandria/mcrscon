use crossterm::style::{ContentStyle, StyledContent, Stylize};
use game_mapper::{Game, GameMapper};
use std::{str::FromStr, thread::sleep, time::Duration};

use argh::FromArgs;
use shell::RCONShell;

use crate::minecraft::responses::MinecraftResponse;

mod game_mapper;
mod minecraft;
mod rcon;
mod shell;

const VERSION: &str = "1.0.0";

#[derive(FromArgs)]
#[argh(description = "Minecraft RCON Implementation for Rust")]
struct Args {
    #[argh(option, description = "address of the server", short = 'a')]
    address: String,

    #[argh(
        option,
        description = "RCON port of the server",
        default = "25575",
        short = 'p'
    )]
    port: u16,

    #[argh(option, description = "RCON password", short = 'P')]
    password: String,

    #[argh(
        option,
        description = "enables game specific prompt features (minecraft, rust, factorio, source)",
        short = 'g',
        default = "Game::from_str(\"minecraft\").unwrap()"
    )]
    game: game_mapper::Game,

    #[argh(
        positional,
        description = "will be executed and shell mode will not be entered"
    )]
    commands: Vec<String>,

    #[argh(
        option,
        description = "seconds to wait between each passed command",
        short = 'w'
    )]
    wait: Option<u64>,

    #[argh(
        switch,
        description = "disables output printing for passed commands",
        short = 's'
    )]
    silent: Option<bool>,

    #[argh(switch, description = "prints version information", short = 'v')]
    version: Option<bool>,
}

#[tokio::main]
async fn main() {
    let args: Args = argh::from_env();

    if args.version == Some(true) {
        print_version();
        std::process::exit(0);
    }

    // Used as an ID for the RCON protocol
    let pid = std::process::id();

    let mut rcon = rcon::RCONConnection::new(&args.address, args.port, pid as i32)
        .await
        .unwrap();
    print_if_not_silent("Connected to RCON.".white(), &args);
    match rcon.auth(&args.password).await {
        Ok(_) => {
            print_if_not_silent("Logged in.".white(), &args);
        }
        Err(_) => print_if_not_silent("Failed to log in.".red(), &args),
    }

    if args.commands.len() > 0 {
        for cmd in &args.commands {
            match rcon.send_command(cmd.trim()).await {
                Ok(s) => {
                    if args.silent == None {
                        print_if_not_silent(s.as_str().white(), &args);
                    }
                }
                Err(e) => eprintln!("{}", e),
            }

            if args.wait.is_some() {
                sleep(Duration::from_secs(args.wait.unwrap()))
            }
        }

        std::process::exit(0);
    }

    println!("Creating a {} prompt.", args.game);
    let shell;
    match args.game {
        Game::MINECRAFT => {
            shell =
                RCONShell::<minecraft::Minecraft, MinecraftResponse>::new(&mut rcon, args.address);
        }
        _ => {}
    }

    match shell.run().await {
        Err(e) => {
            println!("Shell exited with error: {}", e);
            std::process::exit(1)
        }
        Ok(_) => {}
    }
}

fn print_version() {
    let v_string = StyledContent::new(
        ContentStyle::new().bold(),
        "mcrscon v".to_string() + VERSION,
    );
    println!("{}", v_string);
    println!("──────────────");
    println!("Licensed under MIT");
}

fn print_if_not_silent(output: StyledContent<&str>, args: &Args) {
    if args.silent == None {
        println!("{}", output);
    }
}
