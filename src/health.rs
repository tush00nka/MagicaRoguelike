use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max: i32,
    pub current: i32,
    pub extra_lives: u8,
    pub hit_queue: Vec<(i32, Vec3)>,
}

impl Health {
    pub fn new(value: i32) -> Self {
        Self {
            max: value,
            current: value,
            extra_lives: 0,
            hit_queue: vec![]
        }
    }

    pub fn heal(&mut self, value: i32) {
        if self.current + value >= self.max {
            self.current = self.max;
        }
        else {
            self.current += value;
        }
    }
    pub fn damage(&mut self, value: i32,) {
        self.current -= value;
    }
}