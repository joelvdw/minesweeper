use std::time::Duration;
use terminal_size::{Width, Height, terminal_size};
use console::measure_text_width;

use crate::minesweeper::{Board, Case};

pub struct Display {
    screen_size: (u16, u16),
    board_size: (u16, u16),
    pub cursor: (u16, u16)
}

pub enum Direction {
    Up, Down, Left, Right
}

enum TermColor {
    FrontRed,
    FrontReset,
    BackGrey,
    BackReset
}

impl Display {
    pub fn init(board_size: (usize, usize)) -> Self {
        if let Some((Width(w), Height(h))) = terminal_size() {
            let d = Display { 
                screen_size: (h, w),
                board_size: (board_size.0 as u16, board_size.1 as u16),
                cursor: (0, 0)
            };
            if !d.is_valid() {
                panic!("Terminal too small to display a {}x{} board", board_size.1, board_size.0);
            }
            d.clear();
            print!("\x1B[?25l");
            d
        } else {
            panic!("Unable to retrieve terminal size")
        }
    }

    fn is_valid(&self) -> bool {
        (self.board_size.0 + 2) <= self.screen_size.0 && (self.board_size.1 + 2) <= self.screen_size.1
    }

    fn get_char(&self, case: &Case) -> String {
        match case {
            Case::Empty { flag: true, .. } |
            Case::Mine { flag: true } => self.colored("þ", TermColor::FrontRed),
            Case::Discovered { around } => if *around == 0 {
                " ".to_string()
            } else {
                format!("{around}").chars().next().unwrap_or(' ').to_string()
            },
            Case::Empty { .. } => "■".to_string(),
            Case::Mine { .. } => "■".to_string(),
            Case::Exploded => self.colored("*", TermColor::FrontRed),
        }
    }

    fn set_color(&self, color: TermColor) -> String {
        let c = match color {
            TermColor::FrontRed => "31",
            TermColor::FrontReset => "39",
            TermColor::BackGrey => "100",
            TermColor::BackReset => "49",
        };
        format!("\x1B[{}m", c)
    }

    fn reset_color(&self) -> String {
        self.set_color(TermColor::FrontReset) + 
        self.set_color(TermColor::BackReset).as_str()
    }

    fn colored(&self, str: &str, color: TermColor) -> String {
        self.set_color(color) + str + self.reset_color().as_str()
    }

    fn board_strings(&self, board: &Board, lost: bool) -> Vec<String> {
        let mut lines = Vec::with_capacity(board.size.0+2);
        lines.push(format!("┏{}┓", "━".repeat(board.size.1)));

        for i in 0..board.size.0 {
            let mut line = "┃".to_string();
            for j in 0..board.size.1 {
                let mut symbol = self.get_char(&board.data[i][j]);

                let is_cursor = i == self.cursor.0 as usize && j == self.cursor.1 as usize;
                if !lost && is_cursor {
                    symbol = self.colored(symbol.as_str(), TermColor::BackGrey);
                }
                
                line += symbol.as_str();
            }
            line += "┃";
            lines.push(line);
        }

        lines.push(format!("┗{}┛", "━".repeat(board.size.1)));
        lines
    }

    fn stats_string(&self, board: &Board, elapsed: Duration, min_size: usize) -> Vec<String> {
        let max_flag = (board.size.0*board.size.1).to_string().len();
        let sec = elapsed.as_secs();
        let mines_str = format!("Mines: {: >width$}/{}", board.flags, board.mines, width=max_flag);
        let time_str = format!("Time: {:0>2}:{:0>2}", sec/60, sec%60);
        let formatted = format!(
            "{}{: >width$}{}",
            mines_str, " ", time_str,
            width=((measure_text_width(mines_str.as_str())+measure_text_width(time_str.as_str())+1)%2) + 2
        );
        let stats = ["Minesweeper".to_string(), formatted];
        let stats_size = min_size.max(stats.iter().map(|x|measure_text_width(x)).max().unwrap_or(0)) + 2;
        
        let mut lines = Vec::with_capacity(stats.len()+2);
        lines.push(format!("┏{}┓", "━".repeat(stats_size)));
        for s in stats {
            lines.push(format!("┃{: >width$}{}{: >width$}┃", " ", s," ", width=(stats_size-measure_text_width(s.as_str()))/2).to_string());
        }
        lines.push(format!("┗{}┛", "━".repeat(stats_size)));
        lines
    }

    pub fn update(&self, board: &Board, elapsed: Duration, lost: bool) {
        print!("\x1Bc");
        let board_str = self.board_strings(board, lost);
        let stats = self.stats_string(board, elapsed, 27);

        let maxb = board_str.iter().map(|x| measure_text_width(x)).max().unwrap_or(0);
        let maxs = stats.iter().map(|x| measure_text_width(x)).max().unwrap_or(0);
        let aside = (maxb + maxs) <= self.screen_size.1 as usize;

        for i in 0..board_str.len().max(stats.len()) {
            let mut space = self.screen_size.1 as i32;

            if i < board_str.len() {
                let s = &board_str[i];
                print!("{}", s);
                space -= measure_text_width(s) as i32;
            }

            // Display stats aside the board if possible
            if aside && i < stats.len() {
                let s = &stats[i];
                print!("{: >width$}{}", " ", s, width=(space as usize - measure_text_width(s)) / 2);
            }
            println!();
        }

        // Display stats under the board if possible
        let stats = self.stats_string(board, elapsed, board.size.1);
        let diff = (board_str.len()+stats.len()) as i32 - self.screen_size.0 as i32;
        if !aside && diff <= 0 {
            for s in stats {
                println!("{}", s);
            }
        }

        if lost {
            println!("{}", self.colored("You have lost !", TermColor::FrontRed));
        }
    }

    pub fn move_cursor(&mut self, dir: Direction) {
        let (x, y) = match dir {
            Direction::Up => (
                0.max(self.cursor.0 as i32 - 1) as u16,
                self.cursor.1
            ),
            Direction::Down => (
                (self.board_size.0 as i32 - 1).min(self.cursor.0 as i32 + 1) as u16,
                self.cursor.1
            ),
            Direction::Left => (
                self.cursor.0,
                0.max(self.cursor.1 as i32 - 1) as u16
            ),
            Direction::Right => (
                self.cursor.0,
                (self.board_size.1 as i32 - 1).min(self.cursor.1 as i32 + 1) as u16
            ),
        };
        self.cursor = (x, y)
    }

    pub fn clear(&self) {
        print!("\x1Bc");
        for _ in 0..self.screen_size.0 {
            println!();
        }
        print!("\x1Bc");
    }

    pub fn close(&self) {
        print!("\x1B[?25h");
    }
}