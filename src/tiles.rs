use std::collections::HashMap;

pub enum Tileset {
    Town,
}

pub struct Tiledef(pub Tileset, pub u32, pub u32);

pub struct Tiles {
    pub tiles: HashMap<&'static str, Tiledef>,
}

impl Tiles {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::from_iter(vec![
                ("grass", Tiledef(Tileset::Town, 0, 0)),
                ("brick", Tiledef(Tileset::Town, 0, 1)),
                ("lamp", Tiledef(Tileset::Town, 0, 2)),
                ("sand", Tiledef(Tileset::Town, 2, 3)),
                ("well", Tiledef(Tileset::Town, 1, 3)),
            ]),
        }
    }
}
