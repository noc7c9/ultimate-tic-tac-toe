extern crate ultimate_tic_tac_toe;
use ultimate_tic_tac_toe::*;

mod ui;
use ui::{UI, Style, Color, Direction};

const X_COLOR: Color = Color::Red;
const O_COLOR: Color = Color::Blue;
const SQUARE_HIGHTLIGHT: Color = Color::Cyan;

type CursorPos = (u16, u16);

fn cursorpos_to_game_fullcoord(cursor: &CursorPos) -> FullCoord {
    let (outer, inner) = cursorpos_to_ui_coord_pair(&cursor);
    FullCoord::try_new(
        (outer.0 as usize, outer.1 as usize),
        (inner.0 as usize, inner.1 as usize),
    ).unwrap()
}

fn cursorpos_to_ui_coord_pair(cursor: &CursorPos) -> (ui::Coord, ui::Coord) {
    let ox = cursor.0 / 3;
    let ix = cursor.0 % 3;
    let oy = cursor.1 / 3;
    let iy = cursor.1 % 3;
    ((ox, oy), (ix, iy))
}

fn cursor_highlight(ui: &mut UI, cursor: &CursorPos, player: Piece) {
    let coords = cursorpos_to_ui_coord_pair(cursor);
    let style = Style::new().bg(match player {
        Piece::X => X_COLOR,
        Piece::O => O_COLOR,
    });
    ui.color_inner_square(coords, style);
}

fn cursor_highlight_clear(ui: &mut UI, cursor: &CursorPos) {
    let coords = cursorpos_to_ui_coord_pair(cursor);
    ui.color_inner_square(coords, Style::new().clear_bg());
}

fn move_cursor(game: &Game, cursor: &mut CursorPos, dir: Direction) {
    let original_pos = *cursor;
    let active_outer_square = game.active_outer_square();
    loop {
        match dir {
            Direction::Up => {
                cursor.1 = if cursor.1 == 0 { 8 } else { cursor.1 - 1 }
            },
            Direction::Down => {
                cursor.1 = if cursor.1 == 8 { 0 } else { cursor.1 + 1 }
            },
            Direction::Left => {
                cursor.0 = if cursor.0 == 0 { 8 } else { cursor.0 - 1 }
            },
            Direction::Right => {
                cursor.0 = if cursor.0 == 8 { 0 } else { cursor.0 + 1 }
            },
        }

        // We're back at our starting spot,
        // which means there are no valid square in this direction.
        // So give up.
        if *cursor == original_pos {
            break;
        }

        let coord = cursorpos_to_game_fullcoord(cursor);

        // This square is in a completed outer square, move one more square
        if let OuterSquare::Complete(_) = game.get_outer_square(&coord.outer()) {
            continue;
        }

        // This square is outside the active outer square, move one more square
        if active_outer_square.is_some() && Some(coord.outer()) != active_outer_square {
            continue;
        }

        // This square is occupied, move one more square
        if let Square::Filled(_) = game.get_square(&coord) {
            continue;
        }

        // Reached this point so we must be in a valid spot
        break;
    }
}

fn main() {
    let mut game = Game::new();
    let mut ui = UI::new();

    let mut cursor: CursorPos = (0, 0);
    let mut active_highlight: Option<ui::Coord> = None;

    ui.full_render();

    cursor_highlight(&mut ui, &cursor, game.turn());

    loop {
        match ui.read_input() {
            ui::Input::Exit => break,
            ui::Input::Move(dir) => {
                cursor_highlight_clear(&mut ui, &cursor);
                move_cursor(&game, &mut cursor, dir);
                cursor_highlight(&mut ui, &cursor, game.turn());
            },
            ui::Input::Select => {
                let (outer, inner) = cursorpos_to_ui_coord_pair(&cursor);
                let coord = cursorpos_to_game_fullcoord(&cursor);

                if let Ok(_) = game.play_move(&coord) {
                    // Update the square
                    let player = game.turn().opposite();
                    let value = match player {
                        Piece::X => 'X',
                        Piece::O => 'O',
                    };
                    let style = match player {
                        Piece::X => Style::new().fg(X_COLOR),
                        Piece::O => Style::new().fg(O_COLOR),
                    };
                    ui.inner_square(value, (outer, inner), style);

                    // Clear current active outer square highlight
                    if let Some(coord) = active_highlight {
                        ui.outer_square_grid(coord, Style::new().bg(Color::Black));
                    }
                    active_highlight = None;

                    // Highlight next active outer square
                    if let Some(game_coord) = game.active_outer_square() {
                        let coord = (game_coord.x() as u16, game_coord.y() as u16);
                        ui.outer_square_grid(coord, Style::new().bg(SQUARE_HIGHTLIGHT));
                        active_highlight = Some(coord);
                    }

                    // Move cursor to first playable move
                    cursor_highlight_clear(&mut ui, &cursor);
                    if let Some(m) = game.get_moves().first() {
                        cursor.0 = (m.outer_x() * 3 + m.inner_x()) as u16;
                        cursor.1 = (m.outer_y() * 3 + m.inner_y()) as u16;
                        cursor_highlight(&mut ui, &cursor, game.turn());
                    }

                    // Update completed outer squares
                    if let OuterSquare::Complete(result) = game.get_outer_square(&coord.outer()) {
                        let style = Style::new().bg(Color::Black);
                        match result {
                            GameOverResult::Draw =>
                                ui.outer_square_draw(outer, style),
                            GameOverResult::Winner(Piece::X) =>
                                ui.outer_square_x(outer, style.fg(X_COLOR)),
                            GameOverResult::Winner(Piece::O) =>
                                ui.outer_square_o(outer, style.fg(O_COLOR)),
                        }
                    }

                    // Check for game over
                    if let GameState::GameOver(result) = game.state() {
                        ui.reset_cursor_position();
                        println!("Game Over: {}", match result {
                            GameOverResult::Draw => "It was a draw!".into(),
                            GameOverResult::Winner(winner) => format!("{:?} Won", winner),
                        });
                        break;
                    }
                }
            },
            _ => (),
        }
    }
}
