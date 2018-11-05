extern crate crossterm;

use std::io;

use self::crossterm::Screen;
use self::crossterm::cursor::cursor;
use self::crossterm::input::input;
use self::crossterm::style::style;
use self::crossterm::terminal::{terminal, ClearType};

pub use self::crossterm::style::Color;

const OUTER_GRID_OFFSET: (u16, u16) = (2, 1);
const OUTER_GRID_SQUARE_OFFSET: (u16, u16) = (2, 1);

const FULL_RENDER_HEIGHT: u16 = 24;

const INITIAL_RENDER: &'static str = "
                 ██               ██
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
                 ██               ██
  █████████████████████████████████████████████████
                 ██               ██
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
                 ██               ██
  █████████████████████████████████████████████████
                 ██               ██
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
    ───┼───┼───  ██  ───┼───┼───  ██  ───┼───┼───
       │   │     ██     │   │     ██     │   │
                 ██               ██";

type InnerRender = [&'static str; 7];
const INNER_GRID_RENDER: InnerRender = [
    "               ",
    "     │   │     ",
    "  ───┼───┼───  ",
    "     │   │     ",
    "  ───┼───┼───  ",
    "     │   │     ",
    "               ",
];
const INNER_X_RENDER: InnerRender = [
    "               ",
    "   ██     ██   ",
    "     ██ ██     ",
    "      ███      ",
    "     ██ ██     ",
    "   ██     ██   ",
    "               ",
];
const INNER_O_RENDER: InnerRender = [
    "               ",
    "     █████     ",
    "   ██     ██   ",
    "   ██     ██   ",
    "   ██     ██   ",
    "     █████     ",
    "               ",
];
const INNER_DRAW_RENDER: InnerRender = [
    "               ",
    "               ",
    "               ",
    "   █████████   ",
    "               ",
    "               ",
    "               ",
];

pub type Coord = (u16, u16);

fn outer_square_coord_to_pos((x, y): Coord) -> Coord {
    (x * 17 + OUTER_GRID_OFFSET.0, y * 8 + OUTER_GRID_OFFSET.1)
}

fn inner_square_coord_to_pos(((ox, oy), (ix, iy)): (Coord, Coord)) -> Coord {
    let (ox, oy) = outer_square_coord_to_pos((ox, oy));
    (ox + ix * 4 + 1 + OUTER_GRID_SQUARE_OFFSET.0,
        oy + iy * 2 + OUTER_GRID_SQUARE_OFFSET.1)
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum Input {
    Exit,
    Invalid,
    Select,
    Move(Direction),
    Error(io::Error),
}

pub struct UI {
    screen: Screen,
    square_vals: [[[[char; 3]; 3]; 3]; 3],
    square_fgs: [[[[Color; 3]; 3]; 3]; 3],
    square_bgs: [[[[Color; 3]; 3]; 3]; 3],
    outer_square_bgs: [[Color; 3]; 3],
}

impl UI {
    pub fn new() -> Self {
        return Self {
            screen: Screen::new(false),
            square_vals: [[[[' '; 3]; 3]; 3]; 3],
            square_fgs: [[[[Color::White; 3]; 3]; 3]; 3],
            square_bgs: [[[[Color::Black; 3]; 3]; 3]; 3],
            outer_square_bgs: [[Color::Black; 3]; 3],
        }
    }

    pub fn full_render(&self) {
        let terminal = terminal(&self.screen);
        let cursor = cursor(&self.screen);

        cursor.hide();

        terminal.clear(ClearType::All);
        cursor.goto(0, 0);

        style(INITIAL_RENDER).paint(&self.screen);
    }

    pub fn inner_square(&mut self,
                        value: char,
                        coords: (Coord, Coord),
                        style: Style) {
        // Figure out colors
        let fg = style.fg_or(get_4d_arr(&mut self.square_fgs, coords));
        let bg = if style.clear_bg {
            get_2d_arr(&mut self.outer_square_bgs, coords.0)
        } else {
            style.bg_or(get_4d_arr(&mut self.square_bgs, coords))
        };

        // Save options
        set_4d_arr(&mut self.square_vals, coords, value);
        set_4d_arr(&mut self.square_fgs, coords, fg);
        set_4d_arr(&mut self.square_bgs, coords, bg);

        // Update ui
        let pos = inner_square_coord_to_pos(coords);
        cursor(&self.screen).goto(pos.0, pos.1);
        crossterm::style(value).with(fg).on(bg).paint(&self.screen);
        self.reset_cursor_position();
    }

    pub fn color_inner_square(&mut self,
                        coords: (Coord, Coord),
                        style: Style) {
        let value = get_4d_arr(&self.square_vals, coords);
        self.inner_square(value, coords, style);
    }

    pub fn outer_square_grid(&mut self, outer: Coord, style: Style) {
        self.draw_outer_square(&INNER_GRID_RENDER, outer, style.clone());

        // Draw square contents
        for ix in 0..3 {
            for iy in 0..3 {
                self.color_inner_square((outer, (ix, iy)), style.clone());
            }
        }
    }

    pub fn outer_square_x(&mut self, outer: Coord, style: Style) {
        self.draw_outer_square(&INNER_X_RENDER, outer, style.clone());
    }

    pub fn outer_square_o(&mut self, outer: Coord, style: Style) {
        self.draw_outer_square(&INNER_O_RENDER, outer, style.clone());
    }

    pub fn outer_square_draw(&mut self, outer: Coord, style: Style) {
        self.draw_outer_square(&INNER_DRAW_RENDER, outer, style.clone());
    }

    fn draw_outer_square(&mut self, render: &InnerRender, outer: Coord, style: Style) {
        // Figure out colors
        let fg = style.fg_or(Color::White);
        let bg = style.bg_or(Color::Black);

        set_2d_arr(&mut self.outer_square_bgs, outer, bg);

        let cursor = cursor(&self.screen);
        let (initial_x, initial_y) = outer_square_coord_to_pos(outer);

        for (i, line) in render.iter().enumerate() {
            cursor.goto(initial_x, initial_y + i as u16);
            crossterm::style(line).with(fg).on(bg).paint(&self.screen);
        }
    }

    pub fn read_input(&self) -> Input {
        let input = input(&self.screen);

        self.reset_cursor_position();
        match input.read_char() {
            Ok('x') | Ok('q') => Input::Exit,
            Ok('w') | Ok('k') => Input::Move(Direction::Up),
            Ok('s') | Ok('j') => Input::Move(Direction::Down),
            Ok('a') | Ok('h') => Input::Move(Direction::Left),
            Ok('d') | Ok('l') => Input::Move(Direction::Right),
            Ok(' ') => Input::Select,
            Ok(_) => Input::Invalid,
            Err(err) => Input::Error(err),
        }
    }

    pub fn reset_cursor_position(&self) {
        let cursor = cursor(&self.screen);
        cursor.goto(0, FULL_RENDER_HEIGHT);
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let cursor = cursor(&self.screen);

        cursor.goto(0, FULL_RENDER_HEIGHT);
        cursor.show();
    }
}

fn get_2d_arr<T: Copy>(arr: &[[T; 3]; 3], (x, y): Coord) -> T {
    arr[x as usize][y as usize]
}

fn set_2d_arr<T>(arr: &mut [[T; 3]; 3], (x, y): Coord, val: T) {
    arr[x as usize][y as usize] = val;
}

fn get_4d_arr<T: Copy>(arr: &[[[[T; 3]; 3]; 3]; 3], ((ox, oy), (ix, iy)): (Coord, Coord)) -> T {
    arr[ox as usize][oy as usize][ix as usize][iy as usize]
}

fn set_4d_arr<T>(arr: &mut [[[[T; 3]; 3]; 3]; 3], ((ox, oy), (ix, iy)): (Coord, Coord), val: T) {
    arr[ox as usize][oy as usize][ix as usize][iy as usize] = val;
}

#[derive(Clone)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    clear_fg: bool,
    clear_bg: bool,
}

impl Style {
    pub fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            clear_fg: false,
            clear_bg: false,
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }
    pub fn fg_or(&self, default: Color) -> Color {
        self.fg.unwrap_or(default)
    }
    pub fn clear_fg(mut self) -> Self {
        self.clear_fg = true;
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }
    pub fn bg_or(&self, default: Color) -> Color {
        self.bg.unwrap_or(default)
    }
    pub fn clear_bg(mut self) -> Self {
        self.clear_bg = true;
        self
    }
}
