use std::collections::HashMap;

pub enum Tileset {
    Town,
}

pub struct Tiledef(pub Tileset, pub usize, pub usize);

pub struct Tiles {
    pub tiles: HashMap<&'static str, Tiledef>,
}

impl Tiles {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::from_iter(vec![
                ("grass", Tiledef(Tileset::Town, 0, 0)),
                ("well", Tiledef(Tileset::Town, 1, 3)),
            ]),
        }
    }
}
