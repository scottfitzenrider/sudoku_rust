use std::env;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Instant;

#[derive(Debug)]
struct CellList {
    r: usize,
    c: usize,
    v: Vec<usize>,
}
#[derive(Clone)]
struct Board {
    board: [[[usize; 10]; 10]; 10],
    solvecount: i64,
}
fn get_cells_to_clear(r: usize, c: usize) -> Vec<(usize, usize)> {
    let mut res = Vec::<(usize, usize)>::new();
    for rr in 1..10 {
        res.push((rr, c));
    }
    for cc in 1..10 {
        res.push((r, cc))
    }
    let rs = r - (r - 1) % 3;
    let cs = c - (c - 1) % 3;
    for rr in rs..rs + 3 {
        if rr == r {
            continue;
        }
        for cc in cs..cs + 3 {
            if cc == c {
                continue;
            }
            res.push((rr, cc));
        }
    }
    res
}
fn get_cells_of_block(b: usize) -> Vec<(usize, usize)> {
    let r = ((b - 1) / 3) % 9 * 3 + 1;
    let c = ((b - 1) * 3) % 9 + 1;
    let mut res = Vec::<(usize, usize)>::new();
    for rr in r..r + 3 {
        for cc in c..c + 3 {
            res.push((rr, cc));
        }
    }
    res
}

impl Board {
    fn new() -> Board {
        let mut board = [[[0; 10]; 10]; 10];
        for r in 1..10 {
            for c in 1..10 {
                for v in 1..10 {
                    board[r][c][v] = v;
                }
            }
        }
        return Board {
            board: board,
            solvecount: 0,
        };
    }
    fn get_row_level(&self, row: usize, level: usize) -> Vec<usize> {
        (1..10).map(|c| self.board[row][c][level]).collect()
    }
    fn setsolved(&mut self, row: usize, col: usize, val: usize) {
        // println!("setting {} {} to {}", row, col, val);
        if self.board[row][col][0] != 0 {
            return;
        }
        self.board[row][col][0] = val;
        self.solvecount += 1;
        let cells = get_cells_to_clear(row, col);
        for (r, c) in cells {
            self.board[r][c][val] = 0;
        }
    }
    fn read_board(filename: &String) -> Result<Board, String> {
        let mut board = Board::new();
        let f = File::open(filename);
        match f {
            Err(_) => return Err(format!("Could not read file {}", filename)),
            Ok(_) => {}
        }
        let mut fr = BufReader::new(f.unwrap());
        let mut contents = String::new();
        fr.read_to_string(&mut contents)
            .expect("could not read from file");
        let mut i = 0;
        for c in contents.chars() {
            if c == '\n' || c == '\r' {
                continue;
            }
            let row = i / 9 + 1;
            let col = i % 9 + 1;
            let val = c.to_string().parse::<usize>().unwrap();
            if val != 0 {
                board.setsolved(row, col, val);
            }
            i += 1;
        }
        return Ok(board);
    }
    fn check_rows(&mut self) -> bool {
        if self.is_solved() {
            return false;
        }
        let mut found = false;
        for r in 1..10 {
            for v in 1..10 {
                let nzvals: Vec<usize> = (1..10).filter(|&c| self.board[r][c][v] != 0).collect();
                if nzvals.len() == 1 {
                    self.setsolved(r, nzvals[0], v);
                    found = true;
                }
            }
        }
        found
    }
    fn check_cols(&mut self) -> bool {
        if self.is_solved() {
            return false;
        }
        let mut found = false;
        for c in 1..10 {
            for v in 1..10 {
                let nzvals: Vec<usize> = (1..10).filter(|&r| self.board[r][c][v] != 0).collect();

                if nzvals.len() == 1 {
                    self.setsolved(nzvals[0], c, v);
                    found = true;
                }
            }
        }
        found
    }
    fn check_cells(&mut self) -> Result<bool, String> {
        if self.is_solved() {
            return Ok(false);
        }
        let mut found = false;
        for r in 1..10 {
            for c in 1..10 {
                if self.board[r][c][0] == 0 {
                    let nzvals: Vec<usize> =
                        (1..10).filter(|&v| self.board[r][c][v] != 0).collect();
                    if nzvals.len() == 1 {
                        self.setsolved(r, c, nzvals[0]);
                        found = true;
                    }
                    if nzvals.len() == 0 {
                        return Err(format!("check_cells failed {} {}", r, c));
                    }
                }
            }
        }
        Ok(found)
    }
    fn check_blocks(&mut self) -> Result<bool, String> {
        if self.is_solved() {
            return Ok(false);
        }
        let mut found = false;
        for v in 1..10 {
            for b in 1..10 {
                let mut nzvals = Vec::<(usize, usize)>::new();
                for (r, c) in get_cells_of_block(b) {
                    if self.board[r][c][0] == 0 && self.board[r][c][v] != 0 {
                        nzvals.push((r, c));
                    }
                }

                if nzvals.len() == 1 {
                    self.setsolved(nzvals[0].0, nzvals[0].1, v);
                    found = true;
                } else if nzvals.len() == 2 || nzvals.len() == 3 {
                    let rs: Vec<usize> = nzvals.iter().map(|&x| x.0).collect();
                    let cs: Vec<usize> = nzvals.iter().map(|&x| x.1).collect();
                    if rs.iter().all(|&x| x == rs[0]) {
                        for i in 1..10 {
                            if !cs.iter().any(|&x| x == i) {
                                if self.board[rs[0]][i][v] != 0 {
                                    self.board[rs[0]][i][v] = 0;
                                    found = true;
                                }
                            }
                        }
                    }
                    if cs.iter().all(|&x| x == cs[0]) {
                        for i in 1..10 {
                            if !rs.iter().any(|&x| x == i) {
                                if self.board[i][cs[0]][v] != 0 {
                                    self.board[i][cs[0]][v] = 0;
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(found)
    }

    fn shortest_cell(&mut self) -> Vec<CellList> {
        let mut res = Vec::<CellList>::new();
        for r in 1..10 {
            for c in 1..10 {
                if self.board[r][c][0] == 0 {
                    res.push(CellList {
                        r: r,
                        c: c,
                        v: (1..10).filter(|&v| self.board[r][c][v] != 0).collect(),
                    });
                }
            }
        }
        res.sort_by_key(|n| n.v.len());
        res.reverse();
        return res;
    }
    fn try_board(&mut self) -> Result<(), String> {
        loop {
            let fcell = self.check_cells()?;
            let frow = self.check_rows();
            let fcol = self.check_cols();
            let fblock = self.check_blocks()?;
            if self.is_solved() {
                return Ok(());
            }
            if !(fcell || frow || fcol || fblock) {
                return Ok(());
            }
        }
    }
    fn solve_board(&mut self) -> Result<Board, String> {
        self.try_board()?;
        if self.is_solved() {
            return Ok(self.clone());
        }
        for cell in self.shortest_cell() {
            for v in cell.v {
                let mut bcopy = self.clone();
                // println!("guessing {},{} {}", cell.r, cell.c, v);
                bcopy.setsolved(cell.r, cell.c, v);
                match bcopy.solve_board() {
                    Err(_) => {}
                    b => return b,
                }
                // println!("guess failed ({}, {}), {}", cell.r, cell.c, v);
            }

            return Err("Solve Failed".to_string());
        }
        Err("Nope".to_string())
    }
    fn is_solved(&self) -> bool {
        self.solvecount == 9 * 9
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for r in 1..10 {
            s += &format!("{:?}\n", self.get_row_level(r, 0));
        }
        write!(f, "{}", s)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!(
            "usage: {:?} <filename>\n\twhere <filename> is a text file defining the Sudoku board",
            &args[0]
        );
        return ();
    }
    let filename = &args[1];
    let b = Board::read_board(&filename);
    match b {
        Err(_) => {
            println!("could not open {}", filename);
            return ();
        }
        Ok(_) => {}
    }
    let mut board = b.unwrap();
    println!("{}", board);
    let now = Instant::now();
    let result = board.solve_board();
    let elapsed = now.elapsed();
    match result {
        Err(s) => println!("Could not solve {}", s),
        Ok(b) => println!("\n{}", b),
    }
    println!("{:?}", elapsed);
}
