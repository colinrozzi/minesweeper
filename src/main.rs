use std::ops::IndexMut;

#[derive(Debug, Clone)]
struct Tile {
    value: TileValue,
    exposed: bool,
}

#[derive(Debug, Clone)]
enum TileValue {
    Bomb,
    Number(u8),
}

struct Minesweeper {
    board: Vec<Vec<Tile>>,
    game_state: GameState,
}

enum GameState {
    InProgress,
    Won,
    Lost,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            value: TileValue::Number(0),
            exposed: false,
        }
    }

    fn is_bomb(&self) -> bool {
        match self {
            Tile {
                value: TileValue::Bomb,
                ..
            } => true,
            _ => false,
        }
    }

    fn put_bomb(&mut self) {
        self.value = TileValue::Bomb;
    }
}

impl Minesweeper {
    fn new(size: u64, mine_locations: Vec<(u64, u64)>) -> Self {
        // create a new empty board
        // place the bombs
        // iterate through the board, getting the value for each tile - store it

        // create a new board of size `size` x `size` of Tiles
        let board = Minesweeper::create_empty_board(size);

        // for each of the mine_locations, place a bomb
        let mut minesweeper = Minesweeper {
            board,
            game_state: GameState::InProgress,
        };

        for (x, y) in mine_locations {
            if x < size && y < size {
                minesweeper
                    .board
                    .index_mut(x as usize)
                    .index_mut(y as usize)
                    .put_bomb();
            }
        }

        // for each of the tiles, calculate the number of adjacent bombs
        for x in 0..size {
            for y in 0..size {
                if minesweeper.board[x as usize][y as usize].is_bomb() {
                    continue;
                }

                let mut adjacent_bombs = 0;

                // check all adjacent tiles
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue; // skip the tile itself
                        }
                        let nx = x as i64 + dx as i64;
                        let ny = y as i64 + dy as i64;

                        if nx >= 0 && ny >= 0 && nx < size as i64 && ny < size as i64 {
                            if minesweeper.board[nx as usize][ny as usize].is_bomb() {
                                adjacent_bombs += 1;
                            }
                        }
                    }
                }

                if adjacent_bombs > 0 {
                    minesweeper
                        .board
                        .index_mut(x as usize)
                        .index_mut(y as usize)
                        .value = TileValue::Number(adjacent_bombs);
                }
            }
        }

        return minesweeper;
    }

    fn create_empty_board(size: u64) -> Vec<Vec<Tile>> {
        let mut board = Vec::new();
        for _ in 0..size {
            let row = vec![Tile::new(); size as usize];
            board.push(row);
        }
        return board;
    }

    fn get_tile(&self, location: (u64, u64)) -> &Tile {
        let (x, y) = location;
        if x < self.board.len() as u64 && y < self.board[0].len() as u64 {
            &self.board[x as usize][y as usize]
        } else {
            panic!("Location out of bounds");
        }
    }

    fn next_state(&mut self, click_location: (u64, u64)) -> &mut Minesweeper {
        // take in the click, get the tile at that loc,

        let clicked_tile = self.get_tile(click_location);

        match clicked_tile.value {
            TileValue::Bomb => {
                // if the tile is a bomb, fail
                // return new Minsweeper with game state set to Lost
            }
            TileValue::Number(num) => {
                // if the tile is a number, walk from where that click was out in a BFS, rendering
                // any Tiles with a value of 0 as exposed and all of their neighbors
                //
                // check if we won by just checking the board for any unexposed tiles
                // or just checking the number of exposed tiles against the number of bombs

                // return new Minesweeper with game state set to InProgress or Won

                match clicked_tile.value {
                    TileValue::Bomb => {
                        self.game_state = GameState::Lost;
                    }
                    TileValue::Number(0) => {
                        // BFS to expose all connected 0s and their neighbors
                        let mut queue = vec![click_location];
                        let mut visited = vec![click_location];

                        while let Some((x, y)) = queue.pop() {
                            self.board[x as usize][y as usize].exposed = true;

                            for dx in -1..=1 {
                                for dy in -1..=1 {
                                    if dx == 0 && dy == 0 {
                                        continue; // skip the tile itself
                                    }
                                    let nx = x as i64 + dx as i64;
                                    let ny = y as i64 + dy as i64;

                                    if nx >= 0
                                        && ny >= 0
                                        && nx < self.board.len() as i64
                                        && ny < self.board[0].len() as i64
                                    {
                                        let neighbor = (nx as u64, ny as u64);
                                        if !visited.contains(&neighbor) {
                                            visited.push(neighbor);
                                            self.board[nx as usize][ny as usize].exposed = true;
                                            if let TileValue::Number(0) =
                                                self.board[nx as usize][ny as usize].value
                                            {
                                                queue.push(neighbor);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    TileValue::Number(_) => {
                        self.board[click_location.0 as usize][click_location.1 as usize].exposed =
                            true;
                    }
                }
            }
        }

        self
    }
}

fn main() {
    println!("Hello, world!");
}
