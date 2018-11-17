pub const SIZE: usize = 3;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    fn new(x: usize, y: usize) -> Self {
        Self { x: x, y: y }
    }

    pub fn try_new(x: usize, y:usize) -> Result<Self, String> {
        let c = Self::new(x, y);
        if c.is_valid() { Ok(c) } else { Err("Out of bounds".into()) }
    }

    pub fn is_valid(&self) -> bool {
        self.x < SIZE && self.y < SIZE
    }

    pub fn x(&self) -> usize { self.x }
    pub fn y(&self) -> usize { self.y }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FullCoord {
    outer: Coord,
    inner: Coord,
}

impl FullCoord {
    fn new(outer: Coord, inner: Coord) -> Self {
        Self {
            outer: outer,
            inner: inner,
        }
    }

    pub fn try_new((ox, oy): (usize, usize), (ix, iy): (usize, usize)) -> Result<Self, String> {
        let c = Self::new(Coord::new(ox, oy), Coord::new(ix, iy));
        if c.is_valid() { Ok(c) } else { Err("Out of bounds".into()) }
    }

    pub fn is_valid(&self) -> bool {
        self.outer.is_valid() && self.inner.is_valid()
    }

    pub fn inner(&self) -> Coord { self.inner }
    pub fn outer(&self) -> Coord { self.outer }

    pub fn inner_x(&self) -> usize { self.inner.x() }
    pub fn inner_y(&self) -> usize { self.inner.y() }
    pub fn outer_x(&self) -> usize { self.outer.x() }
    pub fn outer_y(&self) -> usize { self.outer.y() }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Piece {
    X,
    O,
}

impl Piece {
    pub fn opposite(&self) -> Self {
        match self {
            Piece::X => Piece::O,
            Piece::O => Piece::X,
        }
    }
}

trait FilledSquare {
    fn is_filled(&self) -> bool;
    fn filling_piece(&self) -> Option<Piece>;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Square {
    Empty,
    Filled(Piece),
}

impl FilledSquare for Square {
    fn is_filled(&self) -> bool {
        if let Square::Filled(_) = self { true } else { false }
    }
    fn filling_piece(&self) -> Option<Piece> {
        if let Square::Filled(piece) = self { Some(*piece) } else { None }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum OuterSquare {
    Incomplete,
    Complete(GameOverResult),
}

impl FilledSquare for OuterSquare {
    fn is_filled(&self) -> bool {
        if let OuterSquare::Complete(_) = self { true } else { false }
    }
    fn filling_piece(&self) -> Option<Piece> {
        if let OuterSquare::Complete(GameOverResult::Winner(piece)) = self { Some(*piece) } else { None }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameOverResult {
    Draw,
    Winner(Piece),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameState {
    InProgress,
    GameOver(GameOverResult),
}

pub struct Game {
    pub state: GameState,
    pub board: [[[[Square; SIZE]; SIZE]; SIZE]; SIZE],
    pub outer_board: [[OuterSquare; SIZE]; SIZE],
    pub turn: Piece,
    pub active_outer_square: Option<Coord>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            state: GameState::InProgress,
            board: [[[[Square::Empty; SIZE]; SIZE]; SIZE]; SIZE],
            outer_board: [[OuterSquare::Incomplete; SIZE]; SIZE],
            turn: Piece::X,
            active_outer_square: None,
        }
    }

    pub fn state(&self) -> GameState {
        self.state
    }

    pub fn turn(&self) -> Piece {
        self.turn
    }

    pub fn active_outer_square(&self) -> Option<Coord> {
        self.active_outer_square
    }

    pub fn get_outer_square(&self, coord: &Coord) -> OuterSquare {
        self.outer_board[coord.x()][coord.y()]
    }

    pub fn get_square(&self, full_coord: &FullCoord) -> Square {
        let Coord { x: ox, y: oy } = full_coord.outer;
        let Coord { x: ix, y: iy } = full_coord.inner;

        self.board[ox][oy][ix][iy]
    }

    fn set_square(&mut self, full_coord: &FullCoord, piece: Square) {
        let Coord { x: ox, y: oy } = full_coord.outer;
        let Coord { x: ix, y: iy } = full_coord.inner;

        self.board[ox][oy][ix][iy] = piece;
    }

    fn get_moves_inner_board(&self, outer: &Coord, moves: &mut Vec<FullCoord>) {
        let mut full_coord = FullCoord::new(
            *outer,
            Coord::new(0, 0),
        );
        for ix in 0..SIZE {
            full_coord.inner.x = ix;
            for iy in 0..SIZE {
                full_coord.inner.y = iy;

                match self.get_square(&full_coord) {
                    Square::Empty => {
                        moves.push(full_coord)
                    },
                    _ => (),
                }
            }
        }
    }

    pub fn get_moves(&self) -> Vec<FullCoord> {
        if let GameState::GameOver(_) = self.state {
            vec![]
        } else if let Some(active) = self.active_outer_square {
            let mut moves = Vec::with_capacity(SIZE * SIZE);
            self.get_moves_inner_board(&active, &mut moves);
            moves

        } else {
            let mut moves = Vec::with_capacity(SIZE * SIZE * SIZE * SIZE);
            for x in 0..SIZE {
                for y in 0..SIZE {
                    if let OuterSquare::Incomplete = self.outer_board[x][y] {
                        self.get_moves_inner_board(&Coord::new(x, y), &mut moves);
                    }
                }
            }
            moves
        }
    }

    pub fn play_move(&mut self, full_coord: &FullCoord) -> Result<(), String> {
        if let GameState::GameOver(_) = self.state {
            return Err("Attempt to play on a finished game".into());
        }

        if let Square::Filled(_) = self.get_square(full_coord) {
            return Err("Attempt to play on a non-empty square".into());
        }
        if let Some(active) = self.active_outer_square {
            if active != full_coord.outer {
                return Err("Attempt to play outside active outer square".into());
            }
        }

        // TODO: why doesn't this work if the variable is inlined?
        let square = Square::Filled(self.turn);
        self.set_square(full_coord, square);

        self.turn = self.turn.opposite();

        // check for game over conditions on the inner board
        // and update the outer board if necessary
        let Coord { x, y } = full_coord.outer();
        if let Some(result) = check_result(&self.board[x][y]) {
            self.outer_board[x][y] = OuterSquare::Complete(result);
        }

        // update the active outer square
        let Coord { x, y } = full_coord.inner;
        self.active_outer_square = match self.outer_board[x][y] {
            OuterSquare::Incomplete => Some(full_coord.inner),
            OuterSquare::Complete(_) => None,
        };

        // check for game over conditions of the full board
        if let Some(result) = check_result(&self.outer_board) {
            self.state = GameState::GameOver(result);
            self.active_outer_square = None;
        }

        Ok(())
    }
}

fn check_result<T>(board: &[[T; SIZE]; SIZE]) -> Option<GameOverResult>
    where T: FilledSquare + PartialEq
{
    const WINNING_TRIPLES: [(usize, usize, usize, usize, usize, usize); 8] = [
        // Columns
        (0, 0,  0, 1,  0, 2),
        (1, 0,  1, 1,  1, 2),
        (2, 0,  2, 1,  2, 2),
        // Row
        (0, 0,  1, 0,  2, 0),
        (0, 1,  1, 1,  2, 1),
        (0, 2,  1, 2,  2, 2),
        // Diagonals
        (0, 0,  1, 1,  2, 2),
        (0, 2,  1, 1,  2, 0),
    ];

    for (x0, y0,  x1, y1,  x2, y2) in WINNING_TRIPLES.iter() {
        let a = &board[*x0][*y0];
        let b = &board[*x1][*y1];
        let c = &board[*x2][*y2];

        if a == b && b == c {
            if let Some(winner) = a.filling_piece() {
                return Some(GameOverResult::Winner(winner));
            }
        }
    };

    // Draw
    for x in 0..SIZE {
        for y in 0..SIZE {
            if !&board[x][y].is_filled() {
                return None;
            }
        }
    }

    return Some(GameOverResult::Draw);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(x: usize, y: usize) -> Coord {
        Coord::try_new(x, y).unwrap()
    }

    fn ct((x, y): (usize, usize)) -> Coord {
        Coord::try_new(x, y).unwrap()
    }

    fn fc(outer: (usize, usize), inner: (usize, usize)) -> FullCoord {
        FullCoord::try_new(outer, inner).unwrap()
    }

    fn game_with_moves(moves: Vec<((usize, usize), (usize, usize))>) -> Game{
        let mut game = Game::new();

        for (outer, inner) in moves {
            game.play_move(&fc(outer, inner)).unwrap();
        }

        game
    }

    #[test]
    fn a_fresh_game_should_return_all_squares_as_possible_moves() {
        let game = Game::new();
        let moves = game.get_moves();

        assert_eq!(moves.len(), SIZE * SIZE * SIZE * SIZE);
    }

    #[test]
    fn for_a_fresh_game_all_outer_squares_should_be_incomplete() {
        let game = Game::new();

        for x in 0..SIZE {
            for y in 0..SIZE {
                assert_eq!(game.get_outer_square(&c(x, y)), OuterSquare::Incomplete);
            }
        }
    }

    #[test]
    fn playing_a_move_should_change_the_appropriate_square() {
        let mut game = Game::new();
        let center = fc((1, 1), (1, 1));
        game.play_move(&center).unwrap();

        assert_eq!(game.get_square(&center), Square::Filled(Piece::X));
    }

    #[test]
    fn playing_a_move_should_change_the_turn() {
        let mut game = Game::new();
        assert_eq!(game.turn, Piece::X);

        game.play_move(&fc((1, 1), (1, 1))).unwrap();
        assert_eq!(game.turn, Piece::O);

        game.play_move(&fc((1, 1), (0, 0))).unwrap();
        assert_eq!(game.turn, Piece::X);
    }

    #[test]
    fn playing_a_move_should_update_the_active_outer_square() {
        let mut game = Game::new();

        assert_eq!(game.active_outer_square(), None);

        game.play_move(&fc((0, 0), (1, 1))).unwrap();
        assert_eq!(game.active_outer_square(), Some(c(1, 1)));
    }

    #[test]
    fn the_active_outer_square_should_limit_which_moves_are_returned() {
        let mut game = Game::new();
        game.play_move(&fc((0, 0), (1, 1))).unwrap();

        let moves = game.get_moves();
        assert_eq!(moves.len(), SIZE * SIZE);

        for move_ in moves {
            assert_eq!(move_.outer(), c(1, 1));
        }
    }

    #[test]
    fn playing_a_move_should_complete_outer_square_when_appropriate() {
        let target = (0, 0);
        let game = game_with_moves(vec![
            (target, (1, 1)), // first move in target square
            ((1, 1), target), // back to target square
            (target, (2, 0)), // second move in target square
            ((2, 0), target), // back to target square
            (target, (0, 2)), // winning target square
        ]);

        assert_eq!(game.get_outer_square(&ct(target)),
            OuterSquare::Complete(GameOverResult::Winner(Piece::X)));
    }

    #[test]
    fn playing_a_move_that_sets_the_active_square_to_a_completed_square_sets_active_square_to_none() {
        let target = (1, 1);
        let game = game_with_moves(vec![
            (target, (0, 0)),
            ((0, 0), target),
            (target, (2, 2)),
            ((2, 2), (1, 1)),
            // Note: this also makes sure the check is done with the updated outer board
            (target, target),
        ]);

        assert_eq!(game.active_outer_square(), None);
    }

    #[test]
    fn get_moves_should_ignore_completed_outer_squares() {
        let target = (0, 0);
        let game = game_with_moves(vec![
            (target, (1, 1)),
            ((1, 1), target), // -1 move outside target square
            (target, (2, 0)),
            ((2, 0), target), // -1 another move outside target square
            (target, (0, 2)), // winning target square
            ((0, 2), target), // -1 move outside, and back to the target square
            // active outer square is now None
        ]);

        let moves = game.get_moves();
        let expected_move_count = (
            SIZE * SIZE * SIZE * SIZE
            - 3 // minus the three filled moves outside the completed square
            - SIZE * SIZE // minus the completed square
        );
        assert_eq!(moves.len(), expected_move_count);
    }

}
