use ::serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::f32::consts::PI;
use std::fmt;
use std::str::FromStr;

#[derive(Component, Clone, Deserialize, Serialize)]
pub struct Position {
    pub direction: Direction,
    pub x: i32,
    pub z: i32,
}

impl Position {
    pub fn go_forward(&mut self) {
        match self.direction {
            Direction::Right => self.x += 1,
            Direction::Up => self.z -= 1,
            Direction::Left => self.x -= 1,
            Direction::Down => self.z += 1,
        };
    }
    pub fn go_backward(&mut self) {
        match self.direction {
            Direction::Right => self.x -= 1,
            Direction::Up => self.z += 1,
            Direction::Left => self.x += 1,
            Direction::Down => self.z -= 1,
        };
    }
    pub fn rotate_right(&mut self) {
        self.direction = match self.direction {
            Direction::Right => Direction::Down,
            Direction::Up => Direction::Right,
            Direction::Left => Direction::Up,
            Direction::Down => Direction::Left,
        };
    }
    pub fn rotate_left(&mut self) {
        self.direction = match self.direction {
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
        };
    }
}

pub fn get_transform(direction: &Direction, x: f32, z: f32) -> Transform {
    Transform {
        translation: Vec3::new(x, 0.0, z),
        rotation: match direction {
            Direction::Up => Quat::from_rotation_y(0.0),
            Direction::Right => Quat::from_rotation_y(-PI * 0.5),
            Direction::Down => Quat::from_rotation_y(PI),
            Direction::Left => Quat::from_rotation_y(PI * 0.5),
        },
        ..default()
    }
}
#[derive(PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
pub enum Direction {
    Right,
    Up,
    Left,
    Down,
}
impl Direction {
    pub fn reverse(&self) -> Direction {
        match *self {
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
        }
    }
}
impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Direction::Right => "right",
                Direction::Up => "up",
                Direction::Left => "left",
                Direction::Down => "down",
            }
        )
    }
}
impl FromStr for Direction {
    type Err = ();
    fn from_str(input: &str) -> Result<Direction, Self::Err> {
        return match input.to_lowercase().as_str() {
            "right" => Ok(Direction::Right),
            "up" => Ok(Direction::Up),
            "left" => Ok(Direction::Left),
            "down" => Ok(Direction::Down),
            _ => Err(()),
        };
    }
}
