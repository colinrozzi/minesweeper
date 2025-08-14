use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
struct Tile {
    value: TileValue,
    exposed: bool,
    flagged: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum TileValue {
    Bomb,
    Number(u8),
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug)]
struct Minesweeper {
    board: Vec<Vec<Tile>>,
    game_state: GameState,
    size: usize,
    bomb_count: usize,
}

impl Tile {
    fn new() -> Self {
        Tile {
            value: TileValue::Number(0),
            exposed: false,
            flagged: false,
        }
    }

    fn is_bomb(&self) -> bool {
        matches!(self.value, TileValue::Bomb)
    }

    fn set_bomb(&mut self) {
        self.value = TileValue::Bomb;
    }

    fn set_number(&mut self, count: u8) {
        self.value = TileValue::Number(count);
    }

    fn get_number(&self) -> Option<u8> {
        match self.value {
            TileValue::Number(n) => Some(n),
            TileValue::Bomb => None,
        }
    }
}

impl Minesweeper {
    fn new(size: usize, mine_locations: Vec<(usize, usize)>) -> Self {
        let mut board = Self::create_empty_board(size);
        let bomb_count = mine_locations.len();
        
        // Place bombs
        for (x, y) in &mine_locations {
            if *x < size && *y < size {
                board[*x][*y].set_bomb();
            }
        }

        // Calculate adjacent bomb counts for non-bomb tiles
        for x in 0..size {
            for y in 0..size {
                if !board[x][y].is_bomb() {
                    let adjacent_bombs = Self::count_adjacent_bombs(&board, x, y, size);
                    board[x][y].set_number(adjacent_bombs);
                }
            }
        }

        Minesweeper {
            board,
            game_state: GameState::InProgress,
            size,
            bomb_count,
        }
    }

    fn create_empty_board(size: usize) -> Vec<Vec<Tile>> {
        vec![vec![Tile::new(); size]; size]
    }

    fn count_adjacent_bombs(board: &[Vec<Tile>], x: usize, y: usize, size: usize) -> u8 {
        let mut count = 0;
        
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 && (nx as usize) < size && (ny as usize) < size {
                    if board[nx as usize][ny as usize].is_bomb() {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }

    fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.size && y < self.size {
            Some(&self.board[x][y])
        } else {
            None
        }
    }

    fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < self.size && y < self.size {
            Some(&mut self.board[x][y])
        } else {
            None
        }
    }

    fn click_tile(&mut self, x: usize, y: usize) -> Result<(), String> {
        if self.game_state != GameState::InProgress {
            return Err("Game is already finished".to_string());
        }

        let tile = match self.get_tile(x, y) {
            Some(tile) => tile,
            None => return Err("Invalid coordinates".to_string()),
        };

        if tile.exposed || tile.flagged {
            return Err("Tile already exposed or flagged".to_string());
        }

        match tile.value {
            TileValue::Bomb => {
                self.game_state = GameState::Lost;
                // Expose all bombs when game is lost
                self.expose_all_bombs();
            }
            TileValue::Number(0) => {
                self.flood_fill(x, y);
                self.check_win_condition();
            }
            TileValue::Number(_) => {
                if let Some(tile) = self.get_tile_mut(x, y) {
                    tile.exposed = true;
                }
                self.check_win_condition();
            }
        }

        Ok(())
    }

    fn flood_fill(&mut self, start_x: usize, start_y: usize) {
        let mut queue = VecDeque::new();
        let mut visited = vec![vec![false; self.size]; self.size];
        
        queue.push_back((start_x, start_y));
        visited[start_x][start_y] = true;

        while let Some((x, y)) = queue.pop_front() {
            self.board[x][y].exposed = true;

            // If this is a numbered tile (not 0), don't continue flood fill from here
            if let TileValue::Number(n) = self.board[x][y].value {
                if n > 0 {
                    continue;
                }
            }

            // Check all 8 adjacent tiles
            for dx in -1..=1i32 {
                for dy in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if nx >= 0 && ny >= 0 && (nx as usize) < self.size && (ny as usize) < self.size {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        
                        if !visited[nx][ny] && !self.board[nx][ny].is_bomb() && !self.board[nx][ny].flagged {
                            visited[nx][ny] = true;
                            self.board[nx][ny].exposed = true;
                            
                            // Only add to queue if it's a 0 (to continue flood fill)
                            if let TileValue::Number(0) = self.board[nx][ny].value {
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }
            }
        }
    }

    fn expose_all_bombs(&mut self) {
        for row in &mut self.board {
            for tile in row {
                if tile.is_bomb() {
                    tile.exposed = true;
                }
            }
        }
    }

    fn check_win_condition(&mut self) {
        let mut unexposed_non_bombs = 0;
        
        for row in &self.board {
            for tile in row {
                if !tile.is_bomb() && !tile.exposed {
                    unexposed_non_bombs += 1;
                }
            }
        }
        
        if unexposed_non_bombs == 0 {
            self.game_state = GameState::Won;
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) -> Result<(), String> {
        if self.game_state != GameState::InProgress {
            return Err("Game is already finished".to_string());
        }

        let tile = match self.get_tile_mut(x, y) {
            Some(tile) => tile,
            None => return Err("Invalid coordinates".to_string()),
        };

        if tile.exposed {
            return Err("Cannot flag exposed tile".to_string());
        }

        tile.flagged = !tile.flagged;
        Ok(())
    }

    fn display(&self) -> String {
        let mut result = String::new();
        
        // Add column numbers
        result.push_str("   ");
        for i in 0..self.size {
            result.push_str(&format!("{:2} ", i));
        }
        result.push('\n');
        
        for (i, row) in self.board.iter().enumerate() {
            result.push_str(&format!("{:2} ", i));
            for tile in row {
                let display_char = if tile.flagged {
                    "🚩"
                } else if !tile.exposed {
                    "■ "
                } else {
                    match tile.value {
                        TileValue::Bomb => "💣",
                        TileValue::Number(0) => "  ",
                        TileValue::Number(n) => {
                            // Convert number to string and pad
                            &format!("{} ", n)
                        }
                    }
                };
                result.push_str(&format!("{} ", display_char));
            }
            result.push('\n');
        }
        
        result.push_str(&format!("\nGame State: {:?}\n", self.game_state));
        result
    }
}

fn main() {
    // Example usage
    let mine_locations = vec![(0, 0), (1, 1), (2, 2)];
    let mut game = Minesweeper::new(5, mine_locations);
    
    println!("Initial board:");
    println!("{}", game.display());
    
    // Make some moves
    match game.click_tile(0, 1) {
        Ok(_) => println!("Clicked (0, 1)"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nAfter clicking (0, 1):");
    println!("{}", game.display());
    
    // Try flagging a tile
    match game.toggle_flag(0, 0) {
        Ok(_) => println!("Flagged (0, 0)"),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\nAfter flagging (0, 0):");
    println!("{}", game.display());
}
