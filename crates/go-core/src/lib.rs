use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use thiserror::Error;

mod readboard;
pub use readboard::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Black,
    White,
}
impl Color {
    pub fn opponent(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vertex {
    Point(Point),
    Pass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOutcome {
    pub played: Vertex,
    pub captured: Vec<Point>,
    pub ko: Option<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreTally {
    pub black_stones: u32,
    pub white_stones: u32,
    pub black_territory: u32,
    pub white_territory: u32,
    pub black_dead: u32,
    pub white_dead: u32,
    pub ownership: Vec<Option<Color>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    width: u8,
    height: u8,
    stones: Vec<Option<Color>>,
    ko: Option<Point>,
    consecutive_passes: u8,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuleError {
    #[error("board size must be between 2 and 25")]
    InvalidBoardSize,
    #[error("point is outside board")]
    OutOfBounds,
    #[error("point is already occupied")]
    Occupied,
    #[error("move violates simple ko")]
    Ko,
    #[error("suicide move is not allowed")]
    Suicide,
    #[error("point is invalid for scoring")]
    InvalidScoringPoint,
}

impl Board {
    pub fn new(width: u8, height: u8) -> Result<Self, RuleError> {
        if !(2..=25).contains(&width) || !(2..=25).contains(&height) {
            return Err(RuleError::InvalidBoardSize);
        }
        Ok(Self {
            width,
            height,
            stones: vec![None; width as usize * height as usize],
            ko: None,
            consecutive_passes: 0,
        })
    }
    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn height(&self) -> u8 {
        self.height
    }
    pub fn ko(&self) -> Option<Point> {
        self.ko
    }
    pub fn get(&self, point: Point) -> Result<Option<Color>, RuleError> {
        Ok(self.stones[self.index(point)?])
    }
    pub fn stones_snapshot(&self) -> Vec<Option<Color>> {
        self.stones.clone()
    }

    pub fn connected_group(&self, point: Point) -> Result<Vec<Point>, RuleError> {
        let color = self.get(point)?.ok_or(RuleError::InvalidScoringPoint)?;
        let mut seen = HashSet::new();
        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        seen.insert(point);
        group.push(point);
        queue.push_back(point);
        while let Some(current) = queue.pop_front() {
            for n in self.neighbors(current) {
                if !seen.contains(&n) && self.get(n)? == Some(color) {
                    seen.insert(n);
                    group.push(n);
                    queue.push_back(n);
                }
            }
        }
        Ok(group)
    }

    pub fn score(&self, dead: &[Point], neutral: &[Point]) -> Result<ScoreTally, RuleError> {
        let mut dead_set = HashSet::new();
        let mut black_dead = 0u32;
        let mut white_dead = 0u32;

        for &pt in dead {
            match self.get(pt)? {
                Some(Color::Black) => {
                    if dead_set.insert(pt) {
                        black_dead += 1;
                    }
                }
                Some(Color::White) => {
                    if dead_set.insert(pt) {
                        white_dead += 1;
                    }
                }
                None => return Err(RuleError::InvalidScoringPoint),
            }
        }

        let mut neutral_set = HashSet::new();
        for &pt in neutral {
            if self.get(pt)?.is_some() {
                return Err(RuleError::InvalidScoringPoint);
            }
            neutral_set.insert(pt);
        }

        let mut black_stones = 0u32;
        let mut white_stones = 0u32;
        for y in 0..self.height {
            for x in 0..self.width {
                let pt = Point { x, y };
                if dead_set.contains(&pt) {
                    continue;
                }
                match self.get(pt)? {
                    Some(Color::Black) => black_stones += 1,
                    Some(Color::White) => white_stones += 1,
                    None => {}
                }
            }
        }

        let total_points = self.width as usize * self.height as usize;
        let mut visited = vec![false; total_points];
        let mut ownership = vec![None; total_points];
        let mut black_territory = 0u32;
        let mut white_territory = 0u32;

        for y in 0..self.height {
            for x in 0..self.width {
                let pt = Point { x, y };
                let idx = self.index(pt)?;
                if visited[idx] {
                    continue;
                }
                let is_live_stone = self.get(pt)?.is_some() && !dead_set.contains(&pt);
                if is_live_stone {
                    continue;
                }

                let mut region = Vec::new();
                let mut queue = VecDeque::new();
                visited[idx] = true;
                region.push(pt);
                queue.push_back(pt);

                let mut bordering_black = false;
                let mut bordering_white = false;

                while let Some(current) = queue.pop_front() {
                    for n in self.neighbors(current) {
                        let n_idx = self.index(n)?;
                        let n_is_live = self.get(n)?.is_some() && !dead_set.contains(&n);
                        if n_is_live {
                            match self.get(n)? {
                                Some(Color::Black) => bordering_black = true,
                                Some(Color::White) => bordering_white = true,
                                None => unreachable!(),
                            }
                        } else if !visited[n_idx] {
                            visited[n_idx] = true;
                            region.push(n);
                            queue.push_back(n);
                        }
                    }
                }

                let region_owner = match (bordering_black, bordering_white) {
                    (true, false) => Some(Color::Black),
                    (false, true) => Some(Color::White),
                    _ => None,
                };

                if let Some(color) = region_owner {
                    for p in region {
                        if !neutral_set.contains(&p) {
                            let p_idx = self.index(p)?;
                            ownership[p_idx] = Some(color);
                            match color {
                                Color::Black => black_territory += 1,
                                Color::White => white_territory += 1,
                            }
                        }
                    }
                }
            }
        }

        Ok(ScoreTally {
            black_stones,
            white_stones,
            black_territory,
            white_territory,
            black_dead,
            white_dead,
            ownership,
        })
    }

    pub fn set_stone(&mut self, point: Point, color: Option<Color>) -> Result<(), RuleError> {
        let idx = self.index(point)?;
        self.stones[idx] = color;
        self.ko = None;
        self.consecutive_passes = 0;
        Ok(())
    }

    pub fn play(&mut self, color: Color, vertex: Vertex) -> Result<MoveOutcome, RuleError> {
        match vertex {
            Vertex::Pass => {
                self.ko = None;
                self.consecutive_passes = self.consecutive_passes.saturating_add(1);
                Ok(MoveOutcome {
                    played: vertex,
                    captured: vec![],
                    ko: None,
                })
            }
            Vertex::Point(point) => self.play_point(color, point),
        }
    }

    fn play_point(&mut self, color: Color, point: Point) -> Result<MoveOutcome, RuleError> {
        let idx = self.index(point)?;
        if self.stones[idx].is_some() {
            return Err(RuleError::Occupied);
        }
        if self.ko == Some(point) {
            return Err(RuleError::Ko);
        }
        let previous = self.clone();
        self.stones[idx] = Some(color);
        self.consecutive_passes = 0;
        let mut captured = Vec::new();
        for neighbor in self.neighbors(point) {
            if self.get(neighbor)? == Some(color.opponent()) {
                let group = self.group_at(neighbor)?;
                if self.liberty_count(&group) == 0 {
                    for stone in &group {
                        let stone_idx = self.index(*stone)?;
                        self.stones[stone_idx] = None;
                    }
                    captured.extend(group);
                }
            }
        }
        let own_group = self.group_at(point)?;
        if self.liberty_count(&own_group) == 0 {
            *self = previous;
            return Err(RuleError::Suicide);
        }
        self.ko = if captured.len() == 1 && own_group.len() == 1 {
            captured.first().copied()
        } else {
            None
        };
        Ok(MoveOutcome {
            played: Vertex::Point(point),
            captured,
            ko: self.ko,
        })
    }

    fn index(&self, point: Point) -> Result<usize, RuleError> {
        if point.x >= self.width || point.y >= self.height {
            return Err(RuleError::OutOfBounds);
        }
        Ok(point.y as usize * self.width as usize + point.x as usize)
    }
    fn neighbors(&self, point: Point) -> Vec<Point> {
        let mut result = Vec::with_capacity(4);
        if point.x > 0 {
            result.push(Point {
                x: point.x - 1,
                y: point.y,
            });
        }
        if point.y > 0 {
            result.push(Point {
                x: point.x,
                y: point.y - 1,
            });
        }
        if point.x + 1 < self.width {
            result.push(Point {
                x: point.x + 1,
                y: point.y,
            });
        }
        if point.y + 1 < self.height {
            result.push(Point {
                x: point.x,
                y: point.y + 1,
            });
        }
        result
    }
    fn group_at(&self, start: Point) -> Result<Vec<Point>, RuleError> {
        self.connected_group(start)
    }
    fn liberty_count(&self, group: &[Point]) -> usize {
        let mut liberties = HashSet::new();
        for point in group {
            for n in self.neighbors(*point) {
                if self.get(n).ok().flatten().is_none() {
                    liberties.insert(n);
                }
            }
        }
        liberties.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: u8, y: u8) -> Vertex {
        Vertex::Point(Point { x, y })
    }
    #[test]
    fn captures_single_stone() {
        let mut b = Board::new(5, 5).unwrap();
        b.play(Color::Black, p(1, 1)).unwrap();
        b.play(Color::White, p(0, 1)).unwrap();
        b.play(Color::White, p(1, 0)).unwrap();
        b.play(Color::White, p(2, 1)).unwrap();
        let out = b.play(Color::White, p(1, 2)).unwrap();
        assert_eq!(out.captured, vec![Point { x: 1, y: 1 }]);
    }
    #[test]
    fn rejects_suicide() {
        let mut b = Board::new(5, 5).unwrap();
        b.play(Color::White, p(0, 1)).unwrap();
        b.play(Color::White, p(1, 0)).unwrap();
        b.play(Color::White, p(2, 1)).unwrap();
        b.play(Color::White, p(1, 2)).unwrap();
        assert_eq!(b.play(Color::Black, p(1, 1)).unwrap_err(), RuleError::Suicide);
    }

    #[test]
    fn setup_can_add_replace_and_clear_stones() {
        let mut b = Board::new(5, 5).unwrap();
        let point = Point { x: 2, y: 3 };

        b.set_stone(point, Some(Color::Black)).unwrap();
        assert_eq!(b.get(point).unwrap(), Some(Color::Black));

        b.set_stone(point, Some(Color::White)).unwrap();
        assert_eq!(b.get(point).unwrap(), Some(Color::White));

        b.set_stone(point, None).unwrap();
        assert_eq!(b.get(point).unwrap(), None);
        assert_eq!(
            b.set_stone(Point { x: 5, y: 0 }, Some(Color::Black)).unwrap_err(),
            RuleError::OutOfBounds
        );
    }

    #[test]
    fn asymmetric_board_bounds_and_capture_near_long_axis_edge() {
        let mut b = Board::new(4, 9).unwrap();
        assert_eq!(b.width(), 4);
        assert_eq!(b.height(), 9);

        assert_eq!(Board::new(1, 9).unwrap_err(), RuleError::InvalidBoardSize);
        assert_eq!(Board::new(4, 26).unwrap_err(), RuleError::InvalidBoardSize);

        assert_eq!(b.get(Point { x: 4, y: 0 }).unwrap_err(), RuleError::OutOfBounds);
        assert_eq!(b.get(Point { x: 0, y: 9 }).unwrap_err(), RuleError::OutOfBounds);
        assert_eq!(b.get(Point { x: 8, y: 2 }).unwrap_err(), RuleError::OutOfBounds);

        b.play(Color::Black, p(1, 8)).unwrap();
        b.play(Color::White, p(0, 8)).unwrap();
        b.play(Color::White, p(2, 8)).unwrap();
        let out = b.play(Color::White, p(1, 7)).unwrap();
        assert_eq!(out.captured, vec![Point { x: 1, y: 8 }]);
        assert_eq!(b.get(Point { x: 1, y: 8 }).unwrap(), None);
    }

    #[test]
    fn test_connected_group() {
        let mut b = Board::new(5, 5).unwrap();
        let p = |x, y| Point { x, y };

        // Empty point rejects with InvalidScoringPoint
        assert_eq!(
            b.connected_group(p(2, 2)).unwrap_err(),
            RuleError::InvalidScoringPoint
        );
        // Out of bounds rejects with OutOfBounds
        assert_eq!(b.connected_group(p(5, 2)).unwrap_err(), RuleError::OutOfBounds);

        // Single stone
        b.set_stone(p(2, 2), Some(Color::Black)).unwrap();
        let group = b.connected_group(p(2, 2)).unwrap();
        assert_eq!(group, vec![p(2, 2)]);

        // L-shaped connected group plus an adjacent opponent stone and a disconnected same-color stone
        b.set_stone(p(2, 3), Some(Color::Black)).unwrap();
        b.set_stone(p(3, 3), Some(Color::Black)).unwrap();
        b.set_stone(p(1, 2), Some(Color::White)).unwrap();
        b.set_stone(p(4, 4), Some(Color::Black)).unwrap();

        let group = b.connected_group(p(2, 2)).unwrap();
        assert_eq!(group.len(), 3);
        assert!(group.contains(&p(2, 2)));
        assert!(group.contains(&p(2, 3)));
        assert!(group.contains(&p(3, 3)));
        assert!(!group.contains(&p(1, 2)));
        assert!(!group.contains(&p(4, 4)));

        // Opponent single stone
        let white_group = b.connected_group(p(1, 2)).unwrap();
        assert_eq!(white_group, vec![p(1, 2)]);
    }

    #[test]
    fn test_scoring_edge_and_rectangular_mixed_neighbors() {
        // 4x3 rectangular board with edge territory and mixed-neighbor dame
        // y=0: B . . W
        // y=1: B B W W
        // y=2: . B W .
        let mut b = Board::new(4, 3).unwrap();
        let p = |x, y| Point { x, y };

        b.set_stone(p(0, 0), Some(Color::Black)).unwrap();
        b.set_stone(p(0, 1), Some(Color::Black)).unwrap();
        b.set_stone(p(1, 1), Some(Color::Black)).unwrap();
        b.set_stone(p(1, 2), Some(Color::Black)).unwrap();

        b.set_stone(p(3, 0), Some(Color::White)).unwrap();
        b.set_stone(p(2, 1), Some(Color::White)).unwrap();
        b.set_stone(p(3, 1), Some(Color::White)).unwrap();
        b.set_stone(p(2, 2), Some(Color::White)).unwrap();

        let tally = b.score(&[], &[]).unwrap();
        assert_eq!(tally.black_stones, 4);
        assert_eq!(tally.white_stones, 4);
        assert_eq!(tally.black_dead, 0);
        assert_eq!(tally.white_dead, 0);
        // (0, 2) is bounded exclusively by Black living stones
        assert_eq!(tally.black_territory, 1);
        // (3, 2) is bounded exclusively by White living stones
        assert_eq!(tally.white_territory, 1);

        // Ownership row-major width stride: index = y * 4 + x
        let idx = |x, y| (y * 4 + x) as usize;
        assert_eq!(tally.ownership[idx(0, 2)], Some(Color::Black));
        assert_eq!(tally.ownership[idx(3, 2)], Some(Color::White));
        // Dame region {(1, 0), (2, 0)} touches both Black and White -> None
        assert_eq!(tally.ownership[idx(1, 0)], None);
        assert_eq!(tally.ownership[idx(2, 0)], None);
        // Stone intersections must have None ownership
        assert_eq!(tally.ownership[idx(0, 0)], None);
        assert_eq!(tally.ownership[idx(3, 0)], None);
    }

    #[test]
    fn test_scoring_dead_stone_removal_catches_recomputing_before_removal_and_double_count() {
        // 5x3 board:
        // y=0: B B B B B
        // y=1: B . W . B
        // y=2: B B B B B
        // where (2, 1) is a White stone marked dead.
        let mut b = Board::new(5, 3).unwrap();
        let p = |x, y| Point { x, y };

        for x in 0..5 {
            b.set_stone(p(x, 0), Some(Color::Black)).unwrap();
            b.set_stone(p(x, 2), Some(Color::Black)).unwrap();
        }
        b.set_stone(p(0, 1), Some(Color::Black)).unwrap();
        b.set_stone(p(2, 1), Some(Color::White)).unwrap(); // Dead stone
        b.set_stone(p(4, 1), Some(Color::Black)).unwrap();

        let dead = vec![p(2, 1)];
        let tally = b.score(&dead, &[]).unwrap();

        assert_eq!(tally.black_stones, 12);
        assert_eq!(tally.white_stones, 0);
        assert_eq!(tally.white_dead, 1);
        assert_eq!(tally.black_dead, 0);

        // After removing (2, 1), points (1, 1), (2, 1), (3, 1) form a connected empty region
        // surrounded solely by Black living stones.
        // Territory must be exactly 3.
        // - If recomputed before removal, (2, 1) is not empty -> territory would be 2.
        // - If dead stone was double counted into territory -> territory would be 4.
        assert_eq!(tally.black_territory, 3);
        assert_eq!(tally.white_territory, 0);

        let idx = |x, y| (y * 5 + x) as usize;
        assert_eq!(tally.ownership[idx(1, 1)], Some(Color::Black));
        assert_eq!(tally.ownership[idx(2, 1)], Some(Color::Black));
        assert_eq!(tally.ownership[idx(3, 1)], Some(Color::Black));
        assert_eq!(tally.ownership[idx(0, 1)], None);
    }

    #[test]
    fn test_scoring_neutral_point_exclusion_and_corrections() {
        // 5x3 board as above, with (1, 1) marked neutral (seki eye correction)
        let mut b = Board::new(5, 3).unwrap();
        let p = |x, y| Point { x, y };

        for x in 0..5 {
            b.set_stone(p(x, 0), Some(Color::Black)).unwrap();
            b.set_stone(p(x, 2), Some(Color::Black)).unwrap();
        }
        b.set_stone(p(0, 1), Some(Color::Black)).unwrap();
        b.set_stone(p(2, 1), Some(Color::White)).unwrap();
        b.set_stone(p(4, 1), Some(Color::Black)).unwrap();

        let dead = vec![p(2, 1)];
        let neutral = vec![p(1, 1)];
        let tally = b.score(&dead, &neutral).unwrap();

        // (1, 1) is excluded from claimed territory
        assert_eq!(tally.black_territory, 2);
        assert_eq!(tally.white_dead, 1);

        let idx = |x, y| (y * 5 + x) as usize;
        assert_eq!(tally.ownership[idx(1, 1)], None);
        assert_eq!(tally.ownership[idx(2, 1)], Some(Color::Black));
        assert_eq!(tally.ownership[idx(3, 1)], Some(Color::Black));
    }

    #[test]
    fn test_scoring_validation_and_deduplication() {
        let mut b = Board::new(4, 4).unwrap();
        let p = |x, y| Point { x, y };
        b.set_stone(p(1, 1), Some(Color::Black)).unwrap();
        b.set_stone(p(2, 2), Some(Color::White)).unwrap();

        // Out of bounds in dead or neutral
        assert_eq!(b.score(&[p(4, 0)], &[]).unwrap_err(), RuleError::OutOfBounds);
        assert_eq!(b.score(&[], &[p(0, 4)]).unwrap_err(), RuleError::OutOfBounds);

        // Dead point must be occupied in original
        assert_eq!(
            b.score(&[p(0, 0)], &[]).unwrap_err(),
            RuleError::InvalidScoringPoint
        );

        // Neutral point must be empty in original
        assert_eq!(
            b.score(&[], &[p(1, 1)]).unwrap_err(),
            RuleError::InvalidScoringPoint
        );

        // Duplicated dead points count dead stones only once
        let tally = b.score(&[p(2, 2), p(2, 2)], &[]).unwrap();
        assert_eq!(tally.white_dead, 1);
        assert_eq!(tally.white_stones, 0);

        // Duplicated neutral points handled gracefully
        let tally2 = b.score(&[], &[p(0, 0), p(0, 0)]).unwrap();
        assert_eq!(tally2.ownership[0], None);
    }

    #[test]
    fn test_scoring_preserves_board_state() {
        let mut b = Board::new(5, 5).unwrap();
        let p = |x, y| Point { x, y };
        b.set_stone(p(2, 2), Some(Color::Black)).unwrap();
        b.set_stone(p(2, 3), Some(Color::White)).unwrap();

        let snapshot_before = b.stones_snapshot();
        let ko_before = b.ko();

        let _ = b.connected_group(p(2, 2)).unwrap();
        let _ = b.score(&[p(2, 3)], &[p(0, 0)]).unwrap();

        assert_eq!(b.stones_snapshot(), snapshot_before);
        assert_eq!(b.ko(), ko_before);
    }
}
