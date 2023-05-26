//! A 2d chess game made with bevy

use std::ops::ControlFlow;

use bevy::{log, prelude::*};
use bevy_mod_picking::prelude::*;

const BLACK_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
const WHITE_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const SELECTED_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const POSSIBLE_MOVE_COLOR: Color = Color::rgb(0.9, 0., 0.);

#[derive(Resource, Default)]
struct Board {
    pub state: [[Option<Entity>; Self::COLS]; Self::ROWS],
}

impl Board {
    const WIDTH: f32 = 500.0;
    const HEIGHT: f32 = 500.0;

    const COLS: usize = 8;
    const ROWS: usize = 8;

    const SIZE: Vec2 = Vec2::new(Self::WIDTH, Self::HEIGHT);

    const POSITIONS: [[Option<Piece>; Self::COLS]; Self::ROWS] = Self::init_positions();

    const fn init_positions() -> [[Option<Piece>; Self::COLS]; Self::ROWS] {
        let blacks = Self::init_side(PieceColor::Black);
        let whites = Self::init_side(PieceColor::White);

        [
            whites[0],
            whites[1],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            blacks[1],
            blacks[0],
        ]
    }

    const fn init_side(color: PieceColor) -> [[Option<Piece>; Self::COLS]; 2] {
        let (row, pawns_row) = match color {
            PieceColor::White => (0, 1),
            PieceColor::Black => (7, 6),
        };

        [
            [
                Some(Piece::rook(color, row, 0)),
                Some(Piece::bishop(color, row, 1)),
                Some(Piece::knight(color, row, 2)),
                Some(Piece::queen(color, row, 3)),
                Some(Piece::king(color, row, 4)),
                Some(Piece::knight(color, row, 5)),
                Some(Piece::bishop(color, row, 6)),
                Some(Piece::rook(color, row, 7)),
            ],
            [
                Some(Piece::pawn(color, pawns_row, 0)),
                Some(Piece::pawn(color, pawns_row, 1)),
                Some(Piece::pawn(color, pawns_row, 2)),
                Some(Piece::pawn(color, pawns_row, 3)),
                Some(Piece::pawn(color, pawns_row, 4)),
                Some(Piece::pawn(color, pawns_row, 5)),
                Some(Piece::pawn(color, pawns_row, 6)),
                Some(Piece::pawn(color, pawns_row, 7)),
            ],
        ]
    }
}

const TILE_GAP: f32 = 0.0;
const TILE_SIZE: Vec2 = Vec2::new(
    Board::SIZE.x / Board::ROWS as f32 - TILE_GAP * Board::ROWS as f32,
    Board::SIZE.y / Board::COLS as f32 - TILE_GAP * Board::COLS as f32,
);

const PIECE_SIZE: Vec2 = Vec2::new(TILE_SIZE.x / 2., TILE_SIZE.y);

#[derive(Component)]
struct Tile {
    pub x: usize,
    pub y: usize,
}

#[derive(Resource, Default)]
struct SelectedTile {
    pub tile: Option<Entity>,
}

#[derive(Resource, Default)]
struct SelectedPiece {
    pub piece: Option<(Vec<Move>, Entity)>,
}

#[derive(Component, Clone, Copy)]
struct Piece {
    pub piece_type: PieceType,
    pub piece_color: PieceColor,
    pub x: usize,
    pub y: usize,
}

impl Piece {
    const fn rook(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::Rook,
            piece_color: color,
            x,
            y,
        }
    }

    const fn knight(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::Knight,
            piece_color: color,
            x,
            y,
        }
    }

    const fn bishop(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::Bishop,
            piece_color: color,
            x,
            y,
        }
    }

    const fn queen(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::Queen,
            piece_color: color,
            x,
            y,
        }
    }

    const fn king(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::King,
            piece_color: color,
            x,
            y,
        }
    }

    const fn pawn(color: PieceColor, x: usize, y: usize) -> Self {
        Self {
            piece_type: PieceType::Pawn,
            piece_color: color,
            x,
            y,
        }
    }

    #[inline]
    pub fn possible_moves(&self, board: &Board) -> Vec<Move> {
        self.piece_type
            .possible_moves(self.piece_color, self.x, self.y, board)
    }
}

#[derive(Debug, Copy, Clone)]
enum PieceColor {
    White,
    Black,
}

#[derive(Debug, Copy, Clone)]
enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone)]
struct Move {
    pub x: usize,
    pub y: usize,
    pub move_type: MoveType,
}

impl Move {
    #[inline]
    const fn new(x: usize, y: usize, move_type: MoveType) -> Self {
        Self { x, y, move_type }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MoveType {
    Move,
    Capture,
}

impl PieceType {
    /// Return a list of possiblet tiles to which the piece can move.
    #[inline]
    pub fn possible_moves(
        &self,
        color: PieceColor,
        x: usize,
        y: usize,
        board: &Board,
    ) -> Vec<Move> {
        match self {
            Self::Pawn => Self::pawn_moves(color, x, y, board),
            Self::Rook => Self::rook_moves(color, x, y, board),
            Self::Knight => Self::knight_moves(color, x, y, board),
            Self::Bishop => Self::bishop_moves(color, x, y, board),
            Self::Queen => Self::queen_moves(color, x, y, board),
            Self::King => Self::king_moves(color, x, y, board),
        }
    }

    #[inline]
    fn pawn_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        let direction: isize = match color {
            PieceColor::White => 1,
            PieceColor::Black => -1,
        };

        let move_y = ((y as isize) + direction) as usize;

        // Check if the pawn can move forward
        if let ControlFlow::Break(_) = add_move(color, board, x, move_y, &mut moves) {
            // Check if the pawn can move two tiles forward
            if y == 1 || y == 6 {
                add_move(
                    color,
                    board,
                    x,
                    ((y as isize) + direction * 2) as usize,
                    &mut moves,
                );
            }
        }

        // Check if the pawn can capture a piece
        if let ControlFlow::Break(_) = add_move(color, board, x + 1, move_y, &mut moves) {
            add_move(color, board, x - 1, move_y, &mut moves);
        }

        moves
    }

    #[inline]
    fn rook_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        // Check is there is a piece to the right of the rook
        for i in x + 1..Board::COLS {
            if let ControlFlow::Break(_) = add_move(color, board, i, y, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the left of the rook
        for i in (0..x).rev() {
            if let ControlFlow::Break(_) = add_move(color, board, i, y, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the top of the rook
        for i in y + 1..Board::ROWS {
            if let ControlFlow::Break(_) = add_move(color, board, x, i, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the bottom of the rook
        for i in (0..y).rev() {
            if let ControlFlow::Break(_) = add_move(color, board, x, i, &mut moves) {
                break;
            }
        }

        moves
    }

    #[inline]
    fn bishop_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        // Check is there is a piece to the top right of the bishop
        for i in 1..Board::COLS {
            if x + i >= Board::COLS || y + i >= Board::ROWS {
                break;
            }
            if let ControlFlow::Break(_) = add_move(color, board, x + i, y + i, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the top left of the bishop
        for i in 1..Board::COLS {
            if x < i || y + i >= Board::ROWS {
                break;
            }
            if let ControlFlow::Break(_) = add_move(color, board, x - i, y + i, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the bottom right of the bishop
        for i in 1..Board::COLS {
            if x + i >= Board::COLS || y < i {
                break;
            }
            if let ControlFlow::Break(_) = add_move(color, board, x + i, y - i, &mut moves) {
                break;
            }
        }

        // Check is there is a piece to the bottom left of the bishop
        for i in 1..Board::COLS {
            if x < i || y < i {
                break;
            }
            if let ControlFlow::Break(_) = add_move(color, board, x - i, y - i, &mut moves) {
                break;
            }
        }

        moves
    }

    #[inline]
    fn knight_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        // Check is there is a piece to the top right of the knight
        if x + 1 < Board::COLS && y + 2 < Board::ROWS {
            add_move(color, board, x + 1, y + 2, &mut moves);
        }

        // Check is there is a piece to the top left of the knight
        if x > 0 && y + 2 < Board::ROWS {
            add_move(color, board, x - 1, y + 2, &mut moves);
        }

        // Check is there is a piece to the bottom right of the knight
        if x + 1 < Board::COLS && y > 1 {
            add_move(color, board, x + 1, y - 2, &mut moves);
        }

        // Check is there is a piece to the bottom left of the knight
        if x > 0 && y > 1 {
            add_move(color, board, x - 1, y - 2, &mut moves);
        }

        // Check is there is a piece to the right top of the knight
        if x + 2 < Board::COLS && y + 1 < Board::ROWS {
            add_move(color, board, x + 2, y + 1, &mut moves);
        }

        // Check is there is a piece to the right bottom of the knight
        if x + 2 < Board::COLS && y > 0 {
            add_move(color, board, x + 2, y - 1, &mut moves);
        }

        // Check is there is a piece to the left top of the knight
        if x > 1 && y + 1 < Board::ROWS {
            add_move(color, board, x - 2, y + 1, &mut moves);
        }

        // Check is there is a piece to the left bottom of the knight
        if x > 1 && y > 0 {
            add_move(color, board, x - 2, y - 1, &mut moves);
        }

        moves
    }

    #[inline]
    fn queen_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        moves.append(&mut Self::rook_moves(color, x, y, board));
        moves.append(&mut Self::bishop_moves(color, x, y, board));

        moves
    }

    #[inline]
    fn king_moves(color: PieceColor, x: usize, y: usize, board: &Board) -> Vec<Move> {
        let mut moves = vec![];

        // Check is there is a piece to the right of the king
        if x + 1 < Board::COLS {
            add_move(color, board, x + 1, y, &mut moves);
        }

        // Check is there is a piece to the left of the king
        if x > 0 {
            add_move(color, board, x - 1, y, &mut moves);
        }

        // Check is there is a piece to the top of the king
        if y + 1 < Board::ROWS {
            add_move(color, board, x, y + 1, &mut moves);
        }

        // Check is there is a piece to the bottom of the king
        if y > 0 {
            add_move(color, board, x, y - 1, &mut moves);
        }

        // Check is there is a piece to the top right of the king
        if x + 1 < Board::COLS && y + 1 < Board::ROWS {
            add_move(color, board, x + 1, y + 1, &mut moves);
        }

        // Check is there is a piece to the top left of the king
        if x > 0 && y + 1 < Board::ROWS {
            add_move(color, board, x - 1, y + 1, &mut moves);
        }

        // Check is there is a piece to the bottom right of the king
        if x + 1 < Board::COLS && y > 0 {
            add_move(color, board, x + 1, y - 1, &mut moves);
        }

        // Check is there is a piece to the bottom left of the king
        if x > 0 && y > 0 {
            add_move(color, board, x - 1, y - 1, &mut moves);
        }

        moves
    }
}

#[inline]
fn add_move(
    color: PieceColor,
    board: &Board,
    row: usize,
    col: usize,
    moves: &mut Vec<Move>,
) -> ControlFlow<()> {
    if let Some(piece) = board.state[row][col] {
        // if piece == color {
        //     return ControlFlow::Break(());
        // }
        moves.push(Move::new(row, col, MoveType::Capture));
        return ControlFlow::Break(());
    }
    moves.push(Move::new(row, col, MoveType::Move));
    ControlFlow::Continue(())
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            // This sets image filtering to nearest
            // This is done to prevent textures with low resolution (e.g. pixel art) from being blurred
            // by linear filtering.
            ImagePlugin::default_nearest(),
        ))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .insert_resource(Board::default())
        .insert_resource(SelectedTile::default())
        .insert_resource(SelectedPiece { piece: None })
        .add_system(bevy::window::close_on_esc)
        .add_system(move_pieces)
        .run();
}

/// Startup system to create the board
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut board: ResMut<Board>) {
    commands.spawn(Camera2dBundle::default());

    // Draw tiles of the board
    for row in 0..Board::ROWS {
        for col in 0..Board::COLS {
            let x = (col as f32 * TILE_SIZE.x) + TILE_SIZE.x / 2. + (col as f32 * TILE_GAP)
                - Board::SIZE.x / 2.;
            let y = (row as f32 * TILE_SIZE.y) + TILE_SIZE.y / 2. + (row as f32 * TILE_GAP)
                - Board::SIZE.y / 2.;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if (row + col) % 2 == 0 {
                            Color::rgb(0.9, 0.9, 0.9)
                        } else {
                            Color::rgb(0.1, 0.1, 0.1)
                        },
                        custom_size: Some(TILE_SIZE),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(x, y, 0.0)),
                    ..default()
                },
                PickableBundle::default(),
                OnPointer::<Click>::run_callback(select_tile),
                Tile { x: row, y: col },
            ));

            if let Some(piece) = Board::POSITIONS[row][col] {
                let entity = spawn_piece(&mut commands, &asset_server, piece, Vec3::new(x, y, 0.0));
                board.state[row][col] = Some(entity);
            }
        }
    }
}

fn spawn_piece(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    piece: Piece,
    position: Vec3,
) -> Entity {
    let texture = piece_texture(asset_server, piece.piece_color, piece.piece_type);

    let piece = commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                custom_size: Some(PIECE_SIZE),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        PickableBundle::default(),
        piece,
    ));

    piece.id()
}

const WHITE_PREFIX: &str = "W";
const BLACK_PREFIX: &str = "B";

fn piece_texture(
    asset_server: &Res<AssetServer>,
    color: PieceColor,
    piece_type: PieceType,
) -> Handle<Image> {
    let prefix = match color {
        PieceColor::Black => WHITE_PREFIX,
        PieceColor::White => BLACK_PREFIX,
    };

    let file_path = match piece_type {
        PieceType::Pawn => "Pawn.png",
        PieceType::Rook => "Rook.png",
        PieceType::Knight => "Knight.png",
        PieceType::Bishop => "Bishop.png",
        PieceType::Queen => "Queen.png",
        PieceType::King => "King.png",
    };

    let path = format!("{}_{}", prefix, file_path);

    asset_server.load(path.as_str())
}

/// Mark current tile as selected, and add that one to [`SelectedTile`]
/// resource. If `Tile` is not empty, also select the piece on that tile and add
/// it to [`SelectedPiece`], also show possible moves. If there is already a
/// piece selected, move it to the selected tile. If the move is valid, update
/// the board state.
fn select_tile(
    In(event): In<ListenedEvent<Click>>,
    mut commands: Commands,
    mut tiles: Query<(&mut Sprite, &Tile)>,
    mut pieces: Query<&mut Piece>,
    mut selected_tile: ResMut<SelectedTile>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut board: ResMut<Board>,
) -> Bubble {
    deselect_tile(&selected_tile, &mut tiles);
    if let Some((moves, _)) = selected_piece.piece.clone() {
        dehighlight_possible_moves(&moves, &mut tiles);
    }

    let selected_tile_entity = event.target;

    {
        // Select new tile
        let Ok((mut sprite, tile)) = tiles.get_mut(selected_tile_entity) else {
            return Bubble::Burst;
        };
        sprite.color = SELECTED_COLOR;
        selected_tile.tile = Some(selected_tile_entity);

        // If there is a piece on the tile, select it
        if let Some(piece_entity) = board.state[tile.x][tile.y] {
            let piece = pieces.get(piece_entity).expect("Piece not found");

            let moves = piece.possible_moves(&board);

            highlight_possible_moves(&moves, &mut tiles);

            selected_piece.piece = Some((moves, piece_entity));

            return Bubble::Up;
        }
    }

    // If there is a piece selected, move it to the selected tile
    if let Some((moves, piece_entity)) = selected_piece.piece.clone() {
        let piece = pieces.get_mut(piece_entity).expect("Piece not found");

        log::info!("Possible moves: {:?}", moves);

        let (_, tile) = tiles.get_mut(selected_tile_entity).expect("Tile not found");

        move_piece(&mut commands, moves, tile, board, piece, piece_entity);
    }

    Bubble::Up
}

fn move_piece(
    commands: &mut Commands,
    moves: Vec<Move>,
    tile: &Tile,
    mut board: ResMut<Board>,
    mut piece: Mut<Piece>,
    piece_entity: Entity,
) {
    // Check if the selected tile is a valid move
    let valid_move = moves.iter().find(|m| m.x == tile.x && m.y == tile.y);

    if let Some(m) = valid_move {
        if m.move_type == MoveType::Capture {
            let captured_piece = board.state[m.x][m.y].expect("Piece not found");

            commands.entity(captured_piece).despawn();
        }

        // Move the piece
        board.state[piece.x][piece.y] = None;
        board.state[tile.x][tile.y] = Some(piece_entity);

        piece.x = tile.x;
        piece.y = tile.y;
    } else {
        log::info!("Invalid move");
    }
}

fn highlight_possible_moves(moves: &Vec<Move>, tiles: &mut Query<(&mut Sprite, &Tile)>) {
    moves.iter().for_each(|m| {
        for (mut sprite, tile) in tiles.iter_mut() {
            if tile.x == m.x && tile.y == m.y {
                sprite.color = POSSIBLE_MOVE_COLOR;
            }
        }
    });
}

// Dehighlight the tile that was previously selected
fn dehighlight_possible_moves(moves: &Vec<Move>, tiles: &mut Query<(&mut Sprite, &Tile)>) {
    moves.iter().for_each(|m| {
        for (mut sprite, tile) in tiles.iter_mut() {
            if tile.x == m.x && tile.y == m.y {
                sprite.color = if (tile.x + tile.y) % 2 == 0 {
                    WHITE_COLOR
                } else {
                    BLACK_COLOR
                };
            }
        }
    });
}

fn deselect_tile(selected_tile: &ResMut<SelectedTile>, tiles: &mut Query<(&mut Sprite, &Tile)>) {
    if let Some(prev_tile_entity) = selected_tile.tile {
        if let Ok((mut sprite, tile)) = tiles.get_mut(prev_tile_entity) {
            sprite.color = if (tile.x + tile.y) % 2 == 0 {
                WHITE_COLOR
            } else {
                BLACK_COLOR
            };
        }
    }
}

fn move_pieces(mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let x = (piece.y as f32 * TILE_SIZE.x) + TILE_SIZE.x / 2. + (piece.y as f32 * TILE_GAP)
            - Board::SIZE.x / 2.;
        let y = (piece.x as f32 * TILE_SIZE.y) + TILE_SIZE.x / 2. + (piece.y as f32 * TILE_GAP)
            - Board::SIZE.y / 2.;

        transform.translation = Vec3::new(x, y, 0.0);
    }
}
