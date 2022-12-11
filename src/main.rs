mod minesweeper;
mod display;

use std::fmt;
use std::process::exit;
use std::time::{Duration, SystemTime};
use clap::{Parser, ValueEnum};
use console::{Term, Key};

use crate::minesweeper::Board;
use crate::display::Display;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Difficulty {
    Easy, Normal, Hard
}
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = Difficulty::Normal)]
    difficulty: Difficulty,
    #[arg(short, long, conflicts_with = "difficulty", default_value_t = 0)]
    mines: usize,
    #[arg(short, long)]
    size: Option<usize>
}

fn get_values(args: Args) -> (usize, usize) {
    let size = if let Some(s) = args.size {
        s
    } else {
        match args.difficulty {
            Difficulty::Easy => 7,
            Difficulty::Normal => 11,
            Difficulty::Hard => 16
        }
    };
    let mines = if args.mines > 0 {
        args.mines
    } else {
        match args.difficulty {
            Difficulty::Easy => ((size * size * 2) as f32 * 0.12346) as usize,
            Difficulty::Normal => ((size * size * 2) as f32 * 0.16) as usize,
            Difficulty::Hard => ((size * size * 2) as f32 * 0.21) as usize
        }
    };
    (size, mines)
}

fn read_key(term: &Term) -> Key {
    match Term::read_key(&term) {
        Err(e) => {
            eprintln!("Error while reading key: {}", e);
            exit(1);
        },
        Ok(k) => k
    }
}

fn game_loop(board: &mut Board, display: &mut Display) {
    let mut end = false;
    let term = Term::stdout();
    let start = SystemTime::now();

    while !end {
        display.update(board, start.elapsed().unwrap_or(Duration::from_millis(0)), false);
        match read_key(&term) {
            Key::ArrowLeft => display.move_cursor(display::Direction::Left),
            Key::ArrowRight => display.move_cursor(display::Direction::Right),
            Key::ArrowUp => display.move_cursor(display::Direction::Up),
            Key::ArrowDown => display.move_cursor(display::Direction::Down),
            Key::Char(' ') => {
                match board.open(display.cursor.0 as usize, display.cursor.1 as usize) {
                    minesweeper::Status::Exploded => end = true,
                    _ => ()
                }
            },
            Key::Char('f') => match board.flag(display.cursor.0 as usize, display.cursor.1 as usize) {
                _ => ()
            },
            Key::Char('q') => return,
            _ => (),
        }
    }
    display.update(board, start.elapsed().unwrap_or(Duration::from_millis(0)), true);
}

fn main() {
    let args = Args::parse();
    let (size, mines) = get_values(args);

    let mut board = Board::new(size, mines);
    let mut display = Display::init(board.size);
    game_loop(&mut board, &mut display);
    display.close();
}
