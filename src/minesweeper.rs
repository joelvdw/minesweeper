use rand::Rng;

#[derive(Clone, Copy)]
pub enum Case {
    Empty { around: u8, flag: bool },
    Discovered { around: u8 },
    Mine { flag: bool },
    Exploded
}
impl Case {
    pub fn open(self) -> Self {
        use Case::*;
        match self {
            Empty { around, flag: false } => Discovered { around },
            Mine { flag: false } => Exploded,
            _ => self,
        }
    }
    pub fn flag(self) -> Self {
        use Case::*;
        match self {
            Empty { around, flag } => Empty { around, flag: !flag },
            Mine { flag } => Mine { flag: !flag },
            _ => self,
        }
    }
}

pub enum Status {
    OK,
    OutOfBound,
    Exploded
}

pub struct Board {
    pub data: Vec<Vec<Case>>,
    pub size: (usize, usize),
    pub mines: usize,
    pub flags: usize
}

impl Board {
    fn is_in(size: (usize, usize), x: i32, y: i32) -> bool {
        0 <= x && x < size.0 as i32 && 0 <= y && y < size.1 as i32
    }

    fn around(size: (usize, usize), x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut vec = Vec::with_capacity(8);
        for i in [x as i32 - 1, x as i32, x as i32 + 1] {
            for j in [y as i32 - 1, y as i32, y as i32 + 1] {
                if Board::is_in(size, i, j) {
                    vec.push((i as usize, j as usize));
                }
            }
        }
        vec
    }

    pub fn new(size: usize, mines: usize) -> Self {
        let width = size + size;
        let height = size;

        // Invert initialisation if more mines than half the cases
        let inverted = mines > ((width*height)/2);
        let init = if inverted {
            Case::Mine { flag: false }
        } else {
            Case::Empty { around: 0, flag: false }
        };

        let mut data = vec![vec![init; width]; height];
        let mut set = 0;
        while (inverted && set < ((width*height)-mines)) || (!inverted && set < mines) {
            let x = rand::thread_rng().gen_range(0..height);
            let y = rand::thread_rng().gen_range(0..width);

            if inverted {
                if let Case::Empty { .. } = data[x][y] {
                    continue;
                }
                let mut n = 0;
                for (i, j) in Board::around((height, width), x, y) {
                    if let Case::Empty { around, .. } = data[i][j] {
                        data[i][j] = Case::Empty { around: around-1, flag: false };
                    } else {
                        n += 1;
                    }
                }
                data[x][y] = Case::Empty { around: n, flag: false };
            } else {
                if let Case::Mine { .. } = data[x][y] {
                    continue;
                }
                for (i, j) in Board::around((height, width), x, y) {
                    if let Case::Empty { around, .. } = data[i][j] {
                        data[i][j] = Case::Empty { around: around+1, flag: false };
                    }
                }
                data[x][y] = Case::Mine { flag: false };
            }
            set += 1;
        }

        Board { data, size: (height, width), mines, flags: 0 }
    }

    fn propagate_open(&mut self, x: usize, y: usize) {
        for (i, j) in Board::around(self.size, x, y) {
            if let Case::Empty { flag: false, .. } = self.data[i][j] {
                self.open(i, j);
            }
        }
    }

    pub fn open(&mut self, x: usize, y: usize) -> Status {
        if !Board::is_in(self.size, x as i32, y as i32) {
            return Status::OutOfBound;
        }

        let new = self.data[x][y].open();
        self.data[x][y] = new;

        match new {
            Case::Exploded => Status::Exploded,
            Case::Discovered { around: 0 } => {
                self.propagate_open(x, y);
                Status::OK
            },
            Case::Discovered { .. } => Status::OK,
            _ => Status::OK
        }
    }

    pub fn flag(&mut self, x: usize, y: usize) -> Status {
        if !Board::is_in(self.size, x as i32, y as i32) {
            return Status::OutOfBound;
        }

        let new = self.data[x][y].flag();
        self.data[x][y] = new;
        // Update flags counter
        match new {
            Case::Empty { flag, .. } |
            Case::Mine { flag } => {
                if flag {
                    self.flags += 1;
                } else {
                    self.flags = 0.max(self.flags as i32 - 1) as usize
                }
            },
            _ => (),
        };
        Status::OK
    }
}