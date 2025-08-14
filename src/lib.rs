use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub value: TileValue,
    pub exposed: bool,
    pub flagged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TileValue {
    Bomb,
    Number(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    InProgress,
    Won,
    Lost,
}

#[derive(Debug)]
pub struct Minesweeper {
    board: Vec<Vec<Tile>>,
    game_state: GameState,
    size: usize,
    bomb_count: usize,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::new()
    }
}

impl Tile {
    pub fn new() -> Self {
        Tile {
            value: TileValue::Number(0),
            exposed: false,
            flagged: false,
        }
    }

    pub fn is_bomb(&self) -> bool {
        matches!(self.value, TileValue::Bomb)
    }

    pub fn set_bomb(&mut self) {
        self.value = TileValue::Bomb;
    }

    pub fn set_number(&mut self, count: u8) {
        self.value = TileValue::Number(count);
    }

    pub fn get_number(&self) -> Option<u8> {
        match self.value {
            TileValue::Number(n) => Some(n),
            TileValue::Bomb => None,
        }
    }
}

impl Minesweeper {
    pub fn new(size: usize, mine_locations: Vec<(usize, usize)>) -> Self {
        let mut board = Self::create_empty_board(size);
        let bomb_count = mine_locations.len();

        for (x, y) in &mine_locations {
            if *x < size && *y < size {
                board[*x][*y].set_bomb();
            }
        }

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

    /// Creates a new minesweeper game that generates the board after the first click
    /// to guarantee a good starting area (no bomb, ideally a zero for expansion)
    pub fn new_with_first_click(size: usize, bomb_count: usize, first_click: (usize, usize)) -> Self {
        use rand::Rng;
        
        let (first_x, first_y) = first_click;
        
        // Validate first click coordinates
        if first_x >= size || first_y >= size {
            panic!("First click coordinates out of bounds");
        }
        
        // Create list of all positions
        let mut all_positions = Vec::new();
        for x in 0..size {
            for y in 0..size {
                all_positions.push((x, y));
            }
        }
        
        // Remove the first click position and its neighbors from possible bomb locations
        // This ensures the first click will be a zero (or at least not a bomb with low numbers around)
        let forbidden_positions = Self::get_area_around(first_x, first_y, size);
        all_positions.retain(|pos| !forbidden_positions.contains(pos));
        
        // If we don't have enough positions left, just exclude the first click position
        if all_positions.len() < bomb_count {
            all_positions.clear();
            for x in 0..size {
                for y in 0..size {
                    if x != first_x || y != first_y {
                        all_positions.push((x, y));
                    }
                }
            }
        }
        
        // Randomly select bomb positions
        let mut rng = rand::thread_rng();
        let mut mine_locations = Vec::new();
        
        for _ in 0..bomb_count.min(all_positions.len()) {
            let index = rng.gen_range(0..all_positions.len());
            mine_locations.push(all_positions.remove(index));
        }
        
        // Create the game with the selected mine locations
        let mut game = Self::new(size, mine_locations);
        
        // Automatically perform the first click
        game.click_tile(first_x, first_y).expect("First click should always be safe");
        
        game
    }
    
    /// Get all positions around a given coordinate (including the coordinate itself)
    fn get_area_around(x: usize, y: usize, size: usize) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        
        for dx in -1..=1i32 {
            for dy in -1..=1i32 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && ny >= 0 && (nx as usize) < size && (ny as usize) < size {
                    positions.push((nx as usize, ny as usize));
                }
            }
        }
        
        positions
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

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
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

    pub fn click_tile(&mut self, x: usize, y: usize) -> Result<(), String> {
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

            if let TileValue::Number(n) = self.board[x][y].value {
                if n > 0 {
                    continue;
                }
            }

            for dx in -1..=1i32 {
                for dy in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && ny >= 0 && (nx as usize) < self.size && (ny as usize) < self.size
                    {
                        let nx = nx as usize;
                        let ny = ny as usize;

                        if !visited[nx][ny]
                            && !self.board[nx][ny].is_bomb()
                            && !self.board[nx][ny].flagged
                        {
                            visited[nx][ny] = true;
                            self.board[nx][ny].exposed = true;

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

    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_bomb_count(&self) -> usize {
        self.bomb_count
    }

    pub fn count_flagged_tiles(&self) -> usize {
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .filter(|tile| tile.flagged)
            .count()
    }

    pub fn count_exposed_tiles(&self) -> usize {
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .filter(|tile| tile.exposed)
            .count()
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) -> Result<(), String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let mine_locations = vec![(0, 0), (1, 1)];
        let game = Minesweeper::new(3, mine_locations);

        assert_eq!(game.get_size(), 3);
        assert_eq!(game.get_bomb_count(), 2);
        assert_eq!(*game.get_game_state(), GameState::InProgress);
    }

    #[test]
    fn test_mine_placement() {
        let mine_locations = vec![(0, 0), (2, 2)];
        let game = Minesweeper::new(3, mine_locations);

        assert!(game.get_tile(0, 0).unwrap().is_bomb());
        assert!(game.get_tile(2, 2).unwrap().is_bomb());
        assert!(!game.get_tile(1, 1).unwrap().is_bomb());
    }

    #[test]
    fn test_adjacent_bomb_counting() {
        let mine_locations = vec![(0, 0)];
        let game = Minesweeper::new(3, mine_locations);

        let tile = game.get_tile(1, 1).unwrap();
        assert_eq!(tile.get_number(), Some(1));

        let tile = game.get_tile(2, 2).unwrap();
        assert_eq!(tile.get_number(), Some(0));
    }

    #[test]
    fn test_clicking_bomb() {
        let mine_locations = vec![(0, 0)];
        let mut game = Minesweeper::new(2, mine_locations);

        let result = game.click_tile(0, 0);
        assert!(result.is_ok());
        assert_eq!(*game.get_game_state(), GameState::Lost);
    }

    #[test]
    fn test_flagging() {
        let mine_locations = vec![(0, 0)];
        let mut game = Minesweeper::new(2, mine_locations);

        assert!(game.toggle_flag(0, 0).is_ok());
        assert!(game.get_tile(0, 0).unwrap().flagged);

        // Toggle again to unflag
        assert!(game.toggle_flag(0, 0).is_ok());
        assert!(!game.get_tile(0, 0).unwrap().flagged);
    }

    #[test]
    fn test_win_condition() {
        let mine_locations = vec![(0, 0)];
        let mut game = Minesweeper::new(2, mine_locations);

        // Click all non-bomb tiles
        game.click_tile(0, 1).unwrap();
        game.click_tile(1, 0).unwrap();
        game.click_tile(1, 1).unwrap();

        assert_eq!(*game.get_game_state(), GameState::Won);
    }

    #[test]
    fn test_new_with_first_click() {
        // Test that first click never hits a bomb
        for _ in 0..10 {
            let game = Minesweeper::new_with_first_click(10, 15, (5, 5));
            
            // First click should be exposed and not be a bomb
            let first_tile = game.get_tile(5, 5).unwrap();
            assert!(first_tile.exposed);
            assert!(!first_tile.is_bomb());
            
            // Game should still be in progress (not lost)
            assert_eq!(*game.get_game_state(), GameState::InProgress);
            
            // Should have correct bomb count
            assert_eq!(game.get_bomb_count(), 15);
        }
    }

    #[test]
    fn test_first_click_creates_opening() {
        // Test that first click often creates a nice opening (zero tile)
        let mut zero_count = 0;
        for _ in 0..20 {
            let game = Minesweeper::new_with_first_click(10, 10, (5, 5));
            let first_tile = game.get_tile(5, 5).unwrap();
            
            if let Some(0) = first_tile.get_number() {
                zero_count += 1;
            }
        }
        
        // Should get zeros fairly often (this is probabilistic, but should usually work)
        assert!(zero_count > 5, "Should get some zero tiles as first clicks");
    }
}
