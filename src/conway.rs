use anyhow as ah;
use crossterm::{
    QueueableCommand,
    cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
    execute,
    style::Print,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, SetSize, disable_raw_mode, enable_raw_mode,
    },
};
use rand::Rng;
use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

pub struct Grid {
    saved: Vec<Vec<bool>>,
    grid: Vec<Vec<bool>>,
    rows: usize,
    cols: usize,
    paused: bool,
}

/// Creates a random two dimensional vector of booleans
fn create_random_vec(rows: usize, cols: usize) -> Vec<Vec<bool>> {
    let mut rng = rand::rng();

    (0..rows)
        .map(|_| (0..cols).map(|_| rng.random_bool(0.5)).collect())
        .collect()
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = create_random_vec(rows, cols);
        Self {
            saved: grid.clone(),
            grid,
            rows,
            cols,
            paused: false,
        }
    }

    /// Creates a grid from file data
    ///
    /// File must contain only 0s and 1s.
    /// Each line must have the same length
    ///
    /// Changes terminal size to match the grid size
    pub fn from_file(filename: &str) -> Result<Self, GridError> {
        let str = std::fs::read_to_string(filename)?;

        let mut grid = Vec::new();
        let mut prev_len: Option<usize> = None;
        for line in str.lines() {
            let mut row = Vec::new();
            if let Some(len) = prev_len {
                if line.len() != len {
                    return Err(GridError::InconsistentWidth);
                }
            } else {
                prev_len = Some(line.len())
            }
            for c in line.chars() {
                match c {
                    '0' => row.push(false),
                    '1' => row.push(true),
                    invalid => {
                        return Err(GridError::Parse(invalid));
                    }
                }
            }
            grid.push(row);
        }

        let rows = str.lines().count();
        let cols = str.lines().next().unwrap().len();

        // Resize terminal to fit the grid
        execute!(io::stdout(), SetSize(cols as u16, rows as u16))?;

        let paused = false;
        Ok(Self {
            saved: grid.clone(),
            grid,
            rows,
            cols,
            paused,
        })
    }

    /// Randomizes the grid
    pub fn restart(&mut self) {
        self.grid = create_random_vec(self.rows, self.cols);
    }

    /// Internally saves current grid state
    pub fn save_state(&mut self) {
        self.saved = self.grid.clone();
    }

    /// Loads saved grid state
    pub fn load_state(&mut self) {
        self.grid = self.saved.clone();
    }

    /// Saves current grid state to a file
    pub fn save_to_file(&self, filename: &str) -> ah::Result<()> {
        let mut file = std::fs::File::create(filename)?;
        for row in &self.grid {
            for &cell in row {
                let c = if cell { '1' } else { '0' };
                write!(file, "{}", c)?;
            }
            writeln!(file)?;
        }
        Ok(())
    }

    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;

        // 3x3 grid centered in x, y while avoiding out-of-bounds
        for i in x.saturating_sub(1)..=(x + 1).min(self.rows - 1) {
            for j in y.saturating_sub(1)..=(y + 1).min(self.cols - 1) {
                if (i, j) != (x, y) && self.grid[i][j] {
                    count += 1;
                }
            }
        }

        count
    }

    /// Updates the grid according to the rules of Conway's Game of Life
    pub fn update_grid(&mut self) {
        let mut new = vec![vec![false; self.cols]; self.rows];

        for (i, row) in self.grid.iter().enumerate() {
            for (j, &cell) in row.iter().enumerate() {
                let neighbors = self.count_neighbors(i, j);

                new[i][j] = matches!((cell, neighbors), (true, 2..=3) | (false, 3));
            }
        }

        self.grid = new;
    }

    /// Prints the grid to the terminal
    pub fn draw(&mut self) {
        let mut stdout = io::stdout();

        stdout.queue(SavePosition).unwrap();

        for (i, row) in self.grid.iter().enumerate() {
            stdout.queue(MoveTo(0, i as u16)).unwrap();
            for &cell in row {
                let c = if cell { '\u{2588}' } else { ' ' };
                stdout.queue(Print(c)).unwrap();
            }

            if i < self.rows - 1 {}
        }

        stdout.queue(RestorePosition).unwrap();
        stdout.flush().unwrap();
    }

    /// Change terminal to raw mode and enter alternate screen
    ///
    /// Optional if button capture is not desired, and alternate screen is not needed
    pub fn prepare_terminal(&self) -> ah::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(())
    }

    /// Change terminal back to normal mode and leave alternate screen
    pub fn restore_terminal(&self) -> ah::Result<()> {
        execute!(io::stdout(), Show, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }
}

impl Drop for Grid {
    fn drop(&mut self) {
        if let Err(e) = self.restore_terminal() {
            eprintln!("Error restoring terminal: {}", e)
        }
    }
}

#[derive(Debug)]
pub enum GridError {
    Io(io::Error),
    Parse(char),
    InconsistentWidth,
    // SaveWithoutLoad,
}

impl From<io::Error> for GridError {
    fn from(err: io::Error) -> Self {
        GridError::Io(err)
    }
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridError::Io(e) => write!(f, "I/O error: {}", e),
            GridError::Parse(c) => write!(f, "Invalid character: '{}' (expected 0/1)", c),
            GridError::InconsistentWidth => write!(f, "Inconsistent row widths in file"),
            // GridError::SaveWithoutLoad => {
            //     write!(f, "Cannot save grid that wasn't properly loaded")
            // }
        }
    }
}

impl Error for GridError {}
