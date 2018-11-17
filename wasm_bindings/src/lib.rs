extern crate ultimate_tic_tac_toe;
extern crate wasm_bindgen;
extern crate cfg_if;

#[macro_use]
extern crate serde_derive;

use ultimate_tic_tac_toe::*;
use wasm_bindgen::prelude::*;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum JsPiece {
    #[serde(rename = "x")] X,
    #[serde(rename = "o")] O,
}

impl From<Piece> for JsPiece {
    fn from(piece: Piece) -> Self {
        match piece {
            Piece::X => JsPiece::X,
            Piece::O => JsPiece::O,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum JsCompleteState {
    #[serde(rename = "x")] X,
    #[serde(rename = "o")] O,
    #[serde(rename = "draw")] Draw,
}

impl From<GameOverResult> for JsCompleteState {
    fn from(result: GameOverResult) -> Self {
        match result {
            GameOverResult::Draw => JsCompleteState::Draw,
            GameOverResult::Winner(Piece::X) => JsCompleteState::X,
            GameOverResult::Winner(Piece::O) => JsCompleteState::O,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct JsInnerGrid {
    pub grid: [[Option<JsPiece>; SIZE]; SIZE],
    pub completed: Option<JsCompleteState>,
}

#[derive(Serialize, Deserialize)]
pub struct JsGame {
    pub grid: [[JsInnerGrid; SIZE]; SIZE],
    pub turn: JsPiece,
    pub state: String,
    pub active_outer_square: Option<JsCoord>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct JsCoord {
    x: u8,
    y: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct JsMove {
    ox: u8,
    oy: u8,
    ix: u8,
    iy: u8,
    turn: JsPiece,
}

fn game_to_jsgame(game: &Game) -> JsValue {
    let mut grid = [[JsInnerGrid {
        grid: [[None; SIZE]; SIZE],
        completed: None,
    }; SIZE]; SIZE];

    for ox in 0..SIZE {
        for oy in 0..SIZE {
            for ix in 0..SIZE {
                for iy in 0..SIZE {
                    let coord = FullCoord::try_new((ox, oy), (ix, iy)).unwrap();
                    grid[ox][oy].grid[ix][iy] = match game.get_square(&coord) {
                        Square::Empty => None,
                        Square::Filled(piece) => Some(piece.into()),
                    }
                }
            }

            let coord = Coord::try_new(ox, oy).unwrap();

            grid[ox][oy].completed = match game.get_outer_square(&coord) {
                OuterSquare::Incomplete => None,
                OuterSquare::Complete(result) => Some(result.into()),
            };
        }
    }

    use GameState::*;
    use GameOverResult::*;
    let state = match game.state() {
        InProgress => "in-progress",
        GameOver(Draw) => "draw",
        GameOver(Winner(Piece::X)) => "x-wins",
        GameOver(Winner(Piece::O)) => "o-wins",
    }.into();

    let active_outer_square = game
        .active_outer_square()
        .map(|Coord { x, y }| JsCoord { x: x as u8, y: y as u8 });

    let game = JsGame {
        grid,
        turn: game.turn().into(),
        state,
        active_outer_square,
    };

    JsValue::from_serde(&game).unwrap()
}

fn jsgame_to_game(js_game: &JsGame) -> Game {
    use GameState::*;
    use GameOverResult::*;

    let state = match &js_game.state[..] {
        "in-progress" => InProgress,
        "draw" => GameOver(Draw),
        "x-wins" => GameOver(Winner(Piece::X)),
        "o-wins" => GameOver(Winner(Piece::O)),
        _ => InProgress,
    };

    let turn = match js_game.turn {
        JsPiece::X => Piece::X,
        JsPiece::O => Piece::O,
    };

    let active_outer_square = js_game.active_outer_square
        .map(|JsCoord { x, y }| Coord::try_new(x as usize, y as usize).unwrap());

    let mut board = [[[[Square::Empty; SIZE]; SIZE]; SIZE]; SIZE];
    let mut outer_board = [[OuterSquare::Incomplete; SIZE]; SIZE];

    for ox in 0..SIZE {
        for oy in 0..SIZE {
            let inner_grid = js_game.grid[ox][oy];

            outer_board[ox][oy] = match inner_grid.completed {
                None => OuterSquare::Incomplete,
                Some(JsCompleteState::Draw) => OuterSquare::Complete(Draw),
                Some(JsCompleteState::X) => OuterSquare::Complete(Winner(Piece::X)),
                Some(JsCompleteState::O) => OuterSquare::Complete(Winner(Piece::O)),
            };

            for ix in 0..SIZE {
                for iy in 0..SIZE {
                    board[ox][oy][ix][iy] = match inner_grid.grid[ix][iy] {
                        None => Square::Empty,
                        Some(JsPiece::X) => Square::Filled(Piece::X),
                        Some(JsPiece::O) => Square::Filled(Piece::O),
                    }
                }
            }
        }
    }

    Game {
        state,
        turn,
        board,
        outer_board,
        active_outer_square,
    }
}

#[wasm_bindgen]
pub fn initialize() -> JsValue {
    set_panic_hook();

    game_to_jsgame(&Game::new())
}

#[wasm_bindgen(js_name = getMoves)]
pub fn get_moves(game: JsValue) -> Result<JsValue, JsValue> {
    let js_game: JsGame = game.into_serde().unwrap();
    let moves = jsgame_to_game(&js_game).get_moves();

    let mut js_moves: Vec<JsMove> = Vec::with_capacity(moves.len());

    for move_ in moves {
        js_moves.push(JsMove {
            ix: move_.inner_x() as u8,
            iy: move_.inner_y() as u8,
            ox: move_.outer_x() as u8,
            oy: move_.outer_y() as u8,
            turn: js_game.turn,
        });
    }

    Ok(JsValue::from_serde(&js_moves).unwrap())
}

#[wasm_bindgen(js_name = playMove)]
pub fn play_move(game: JsValue, move_: JsValue) -> Result<JsValue, JsValue> {
    let js_game: JsGame = game.into_serde().unwrap();
    let js_move: JsMove = move_.into_serde().unwrap();

    let mut game = jsgame_to_game(&js_game);
    let coord = FullCoord::try_new(
        (js_move.ox as usize, js_move.oy as usize),
        (js_move.ix as usize, js_move.iy as usize),
    ).unwrap();

    game.play_move(&coord).unwrap();

    Ok(game_to_jsgame(&game))
}
