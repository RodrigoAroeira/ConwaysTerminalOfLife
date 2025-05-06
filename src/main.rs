mod conway;

use anyhow as ah;
use conway::Grid;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers as KM},
    terminal,
};
use std::{thread, time::Duration};

/// FPS limit
const FPS: u64 = 25;

fn main() -> ah::Result<()> {
    let (y, x) = terminal::size().expect("Unable to get size");

    // provided is a bool showing if filename was given in the command line
    let (filename, provided) = match std::env::args().nth(1) {
        Some(filename) => (filename, true),
        None => (String::from("grid.data"), false),
    };

    // if provided, get from_file, else create a new grid
    let mut grid = Grid::from_file(&filename)
        .map_err(|e| {
            eprintln!("Error while loading from file: {e}. Creating default grid.");
            thread::sleep(Duration::from_secs(3));
        })
        .ok()
        .filter(|_| provided) // Returns None if provided is false
        .unwrap_or(Grid::new(x as usize, y as usize)); // Creates new if value is none

    grid.prepare_terminal()?;

    loop {
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(event) = event::read()? {
                if !handle_key_event(event, &mut grid, &filename)? {
                    break;
                }
            }
        }

        if grid.paused() {
            continue;
        }

        grid.update_grid();
        grid.draw();

        thread::sleep(Duration::from_millis(1000 / FPS));
    }

    Ok(())
}

fn handle_key_event(event: event::KeyEvent, grid: &mut Grid, filename: &str) -> ah::Result<bool> {
    use KeyCode::{Char, Esc};

    let b = match event.code {
        Char(r) if r.eq_ignore_ascii_case(&'r') => {
            grid.restart();
            true
        }

        Char(c_s) if event.modifiers.contains(KM::CONTROL) && c_s.eq_ignore_ascii_case(&'s') => {
            grid.save_to_file(filename)?;
            true
        }

        Char(s) if s.eq_ignore_ascii_case(&'s') => {
            grid.save_state();
            true
        }

        Char(l) if l.eq_ignore_ascii_case(&'l') => {
            grid.load_state();
            true
        }

        Char(p) if p.eq_ignore_ascii_case(&'p') => {
            grid.toggle_pause();
            true
        }

        // Break conditions
        Esc => false,
        Char(q) if q.eq_ignore_ascii_case(&'q') => false,
        Char(c_c) if event.modifiers.contains(KM::CONTROL) && c_c.eq_ignore_ascii_case(&'c') => {
            false
        }
        _ => true,
    };

    Ok(b)
}
