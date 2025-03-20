#![allow(dead_code)]
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use teleia::*;

mod tiles;

const TILE_DIM: f32 = 16.0;
const PLAYER_SPEED: f32 = 0.5;

fn rect_collide(
    x0: f32, y0: f32,
    x1: f32, y1: f32,
) -> bool {
    let maxx = x0.max(x1);
    let minx = x0.min(x1);
    let maxy = y0.max(y1);
    let miny = y0.min(y1);
    maxx < minx + TILE_DIM && maxy < miny + TILE_DIM
}

struct Tilesheet {
    texture: texture::Texture,
    dim: u32,
    w: u32, h: u32,
}

impl Tilesheet {
    fn new(ctx: &context::Context, dim: u32, w: u32, h: u32, bytes: &[u8]) -> Self {
        Self {
            dim, w, h,
            texture: texture::Texture::new(ctx, bytes),
        }
    }
}

struct Assets {
    font: font::Bitmap,
    shader_flat: shader::Shader,
    shader_sheet: shader::Shader,
    mesh_square: mesh::Mesh,
    tilesheet_player: Tilesheet,
    tilesheet_town: Tilesheet,
}

impl Assets {
    fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Bitmap::new(ctx),
            shader_flat: shader::Shader::new(
                ctx,
                include_str!("assets/shaders/flat/vert.glsl"),
                include_str!("assets/shaders/flat/frag.glsl"),
            ),
            shader_sheet: shader::Shader::new(
                ctx,
                include_str!("assets/shaders/sheet/vert.glsl"),
                include_str!("assets/shaders/sheet/frag.glsl"),
            ),
            mesh_square: mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/square.obj")),
            tilesheet_player: Tilesheet::new(ctx, 16, 8, 1, include_bytes!("assets/textures/player.png")),
            tilesheet_town: Tilesheet::new(ctx, 16, 13, 4, include_bytes!("assets/textures/fftown.png")),
        }
    }
}

#[derive(Debug)]
struct PageEntity {
    message: Option<String>,
}

impl PageEntity {
    fn new(_v: lexpr::Value) -> Self {
        Self {
            message: None,
        }
    }
}

#[derive(Debug)]
struct PageTile {
    image: Option<String>,
    color: Option<String>,
}

impl PageTile {
    fn new(_v: lexpr::Value) -> Self {
        Self {
            image: None,
            color: None,
        }
    }
}


fn value_plist_get<'a>(key: &str, m: &'a lexpr::Value) -> Option<&'a lexpr::Value> {
    let itr = m.list_iter()?;
    itr.skip_while(|x| x.as_symbol() != Some(key)).nth(1)
}

fn value_extract_coordinate_pair(m: &lexpr::Value) -> Option<((f32, f32), &lexpr::Value)> {
    let mut itr = m.list_iter()?;
    let coords = itr.next()?;
    let mut coords_itr = coords.list_iter()?;
    let x = coords_itr.next()?.as_f64()? as f32;
    let y = coords_itr.next()?.as_f64()? as f32;
    let v = itr.next()?;
    Some(((x, y), v))
}

fn value_as_coords(m: &lexpr::Value) -> Option<(f32, f32)> {
    let mut itr = m.list_iter()?;
    let x = itr.next()?.as_f64()? as f32;
    let y = itr.next()?.as_f64()? as f32;
    Some((x, y))
}

#[derive(Debug)]
struct Page {
    url: String,
    tree: lexpr::value::Value,

    spawn: (f32, f32),
    width: u32, height: u32,
    bg_tile: String,
    map: HashMap<(u32, u32), String>,
    links: HashMap<(u32, u32), String>,
    entities: HashMap<(f32, f32), PageEntity>,
    tiles: HashMap<String, PageTile>,
}

impl Page {
    fn new(url: String, tree: lexpr::value::Value) -> Self {
        let spawn = value_plist_get(":spawn", &tree).and_then(|x| value_as_coords(x)).unwrap_or((0.0, 0.0));
        let width = value_plist_get(":width", &tree).and_then(|x| x.as_i64()).unwrap_or(0) as u32;
        let height = value_plist_get(":height", &tree).and_then(|x| x.as_i64()).unwrap_or(0) as u32;
        let bg_tile = value_plist_get(":bg-tile", &tree).and_then(|x| x.as_symbol()).unwrap_or("grass").to_owned();
        let mut map = HashMap::new();
        if let Some(m) = value_plist_get(":map", &tree).and_then(|m| m.list_iter()) {
            for p in m {
                if let Some(((x, y), v)) = value_extract_coordinate_pair(p) {
                    if let Some(s) = v.as_symbol() {
                        map.insert((x as _, y as _), String::from(s));
                    }
                }
            }
        }
        log::info!("map: {:?}", map);
        let mut links = HashMap::new();
        if let Some(m) = value_plist_get(":links", &tree).and_then(|m| m.list_iter()) {
            for p in m {
                if let Some(((x, y), v)) = value_extract_coordinate_pair(p) {
                    if let Some(s) = v.as_str() {
                        links.insert((x as _, y as _), String::from(s));
                    }
                }
            }
        }
        log::info!("links: {:?}", links);
        Self {
            url,
            tree,
            spawn,
            width, height,
            bg_tile,
            map,
            links,
            entities: HashMap::new(),
            tiles: HashMap::new(),
        }
    }
}
   
enum Facing {
    North, South, West, East,
}

impl Facing {
    fn tile_offset_and_flip(&self, walk: u32) -> (u32, bool) {
        match self {
            Self::North => (1, walk == 1),
            Self::South => (0, walk == 1),
            Self::West => (2 + walk, false),
            Self::East => (2 + walk, true),
        }
    }
}

// struct Game {
//     assets: Assets,
//     tiles: tiles::Tiles,
//     page: Option<Page>,
//     player_facing: Facing,
//     player_pos: glam::Vec2,
//     link_target_pos: Option<(f32, f32)>,
// }
// 
// impl Game {
//     async fn new(ctx: &context::Context) -> Self {
//         log::info!("{:?}", level2d::tiled::Level::new(include_str!("../levels/test.json")));
//         Self {
//             assets: Assets::new(ctx),
//             tiles: tiles::Tiles::new(),
//             page: None,
//             player_facing: Facing::South,
//             player_pos: glam::Vec2::new(0.0, 0.0),
//             link_target_pos: None,
//         }
//     }
// 
//     fn load<U>(&mut self, _ctx: &context::Context, st: &mut state::State, url: U) where U: reqwest::IntoUrl {
//         self.page = None;
//         let u = url.into_url().expect("failed to convert to URL");
//         log::info!("loading: {}", u);
//         st.request(|c| {
//             c.get(u.clone())
//         });
//     }
// 
//     fn tile_at(&self, x: f32, y: f32) -> Option<String> {
//         let tx = (x / TILE_DIM) as u32;
//         let ty = (y / TILE_DIM) as u32;
//         if let Some(p) = &self.page {
//             p.map.get(&(tx, ty)).cloned()
//         } else {
//             None
//         }
//     }
// 
//     fn player_collides_at(&self, pos: glam::Vec2) -> bool {
//         let p = if let Some(p) = &self.page {p} else { return false };
//         for ((ix, iy), _) in p.map.iter() {
//             if rect_collide(pos.x, pos.y, *ix as f32 * TILE_DIM, *iy as f32 * TILE_DIM) {
//                 return true
//             }
//         }
//         false
//     }
//     
//     fn player_links_at(&self, pos: glam::Vec2) -> Option<String> {
//         let p = if let Some(p) = &self.page {p} else { return None };
//         for ((ix, iy), l) in p.links.iter() {
//             if rect_collide(pos.x, pos.y, *ix as f32 * TILE_DIM, *iy as f32 * TILE_DIM) {
//                 return Some(l.clone())
//             }
//         }
//         None
//     }
// 
//     fn render_tile_index(
//         &self, ctx: &context::Context, st: &mut state::State,
//         ts: &Tilesheet, tx: u32, ty: u32, flip: bool,
//         pos: glam::Vec2,
//     ) {
//         let dim = ts.dim as f32;
//         let w = 1.0 / ts.w as f32;
//         let h = 1.0 / ts.h as f32;
//         st.bind_2d(ctx, &self.assets.shader_sheet);
//         self.assets.shader_sheet.set_f32(ctx, "tile_w", w);
//         self.assets.shader_sheet.set_f32(ctx, "tile_h", h);
//         self.assets.shader_sheet.set_f32(ctx, "tile_x", tx as f32 * w);
//         self.assets.shader_sheet.set_f32(ctx, "tile_y", ty as f32 * h);
//         self.assets.shader_sheet.set_f32(ctx, "inv_x", if flip { 1.0 } else { 0.0 });
//         self.assets.shader_sheet.set_position_2d(ctx, &pos, &glam::Vec2::new(dim, dim));
//         ts.texture.bind(ctx);
//         self.assets.mesh_square.render(ctx);
//     }
// 
//     fn get_tilesheet(&self, t: &tiles::Tileset) -> &Tilesheet {
//         match t {
//             tiles::Tileset::Town => &self.assets.tilesheet_town,
//         }
//     }
//     fn render_tile(&self, ctx: &context::Context, st: &mut state::State, tile: &str, flip: bool, pos: glam::Vec2) {
//         if let Some(tiles::Tiledef(t, x, y)) = self.tiles.tiles.get(tile) {
//             self.render_tile_index(ctx, st, self.get_tilesheet(&t), *x, *y, flip, pos)
//         } else {
//             log::warn!("could not find tile: {}", tile);
//         }
//     }
//     fn render_map(&self, ctx: &context::Context, st: &mut state::State) {
//         if let Some(p) = &self.page {
//             // log::info!("rendering: {} {}", p.width, p.height);
//             for x in 0..p.width {
//                 for y in 0..p.height {
//                     let t = p.map.get(&(x, y)).unwrap_or(&p.bg_tile);
//                     let fx = x as f32;
//                     let fy = y as f32;
//                     self.render_tile(ctx, st, t, true, glam::Vec2::new(fx * TILE_DIM, fy * TILE_DIM));
//                 }
//             }
//         }
//     }
//     fn render_player(&self, ctx: &context::Context, st: &mut state::State) {
//         let walkcycle = if st.keys.up() || st.keys.down() || st.keys.left() || st.keys.right() {
//             ((st.tick / 15) % 2) as u32
//         } else {
//             0
//         };
//         let (offset, flip) = self.player_facing.tile_offset_and_flip(walkcycle);
//         self.render_tile_index(
//             ctx, st, &self.assets.tilesheet_player,
//             offset, 0, flip,
//             glam::Vec2::new(self.player_pos.x.floor(), self.player_pos.y.floor()),
//         );
//     }
// }
// 
// impl teleia::state::Game for Game {
//     fn initialize_audio(&self, _ctx: &context::Context, _st: &state::State, actx: &audio::Context) -> HashMap<String, audio::Audio> {
//         HashMap::from_iter(vec![
//             ("test".to_owned(), audio::Audio::new(&actx, include_bytes!("assets/audio/test.wav"))),
//         ])
//     }
//     fn finish_title(&mut self, _st: &mut state::State) {
//     }
//     fn request_return(&mut self, _ctx: &context::Context, _st: &mut state::State, res: state::Response) {
//         let str = String::from_utf8_lossy(&res.body).into_owned();
//         let url = res.url;
//         if let Ok(tree) = lexpr::from_str(&str) {
//             let p = Page::new(url, tree);
//             let spawn = if let Some(sp) = self.link_target_pos {
//                 sp
//             } else {
//                 p.spawn
//             };
//             self.player_pos = glam::Vec2::new(spawn.0 * TILE_DIM, spawn.1 * TILE_DIM);
//             self.link_target_pos = None;
//             self.page = Some(p);
//             log::info!("loaded page: {:?}", self.page);
//         } else {
//             log::warn!("failed to decode page: {}", url);
//         }
//     }
//     fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
//         if self.page.is_none() && !st.requesting() {
//             self.load(ctx, st, "https://pub.colonq.computer/~llll/spider/test.sp");
//         }
// 
//         let mut offset = glam::Vec2::new(0.0, 0.0);
//         if st.keys.left() {
//             self.player_facing = Facing::West;
//             offset += PLAYER_SPEED * glam::Vec2::new(-1.0, 0.0);
//         }
//         if st.keys.right() {
//             self.player_facing = Facing::East;
//             offset += PLAYER_SPEED * glam::Vec2::new(1.0, 0.0);
//         }
//         if st.keys.up() {
//             self.player_facing = Facing::North;
//             offset += PLAYER_SPEED * glam::Vec2::new(0.0, -1.0);
//         }
//         if st.keys.down() {
//             self.player_facing = Facing::South;
//             offset += PLAYER_SPEED * glam::Vec2::new(0.0, 1.0);
//         }
//         let offset = offset.normalize_or_zero();
//         if let Some(lf) = self.player_links_at(self.player_pos + offset) {
//             log::info!("lf: {}", lf);
//             let mut itr = lf.split('#'); 
//             if let Some(l) = itr.next() {
//                 log::info!("l: {}", l);
//                 if let Some(frag) = itr.next() {
//                     log::info!("frag: {}", frag);
//                     let mut fitr = frag.split(',');
//                     if let (Some(x), Some(y)) = (
//                         fitr.next().and_then(|x| x.parse::<f32>().ok()),
//                         fitr.next().and_then(|y| y.parse::<f32>().ok()),
//                     ) {
//                         log::info!("x, y: {:?}", (x, y));
//                         self.link_target_pos = Some((x, y));
//                     }
//                 }
//                 self.load(ctx, st, l);
//             }
//         } else {
//             if !self.player_collides_at(self.player_pos + glam::Vec2::new(offset.x, 0.0)) {
//                 self.player_pos = self.player_pos + glam::Vec2::new(offset.x, 0.0);
//             }
//             if !self.player_collides_at(self.player_pos + glam::Vec2::new(0.0, offset.y)) {
//                 self.player_pos = self.player_pos + glam::Vec2::new(0.0, offset.y);
//             }
//         }
//         Ok(())
//     }
//     fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
//         ctx.clear();
//         self.render_map(ctx, st);
//         self.render_player(ctx, st);
//         Ok(())
//     }
// }

struct Game {
    test: texture::Texture,
    shader: shader::Shader,
    mesh: mesh::Mesh,
    tassets: level2d::tiled::Assets,
    level: level2d::tiled::Level,
    renderer: level2d::tiled::LevelRenderer,
}
impl Game {
    async fn new(ctx: &context::Context) -> Self {
        let mut tassets = level2d::tiled::Assets::new();
        tassets.load(ctx, "cave", include_str!("assets/tilesets/cave.tsj"), include_bytes!("assets/tilesets/cave.png")).unwrap();
        tassets.load(ctx, "mrgreen", include_str!("assets/tilesets/mrgreen.tsj"), include_bytes!("assets/tilesets/mrgreen.png")).unwrap();
        tassets.load(ctx, "onewilliamdollars", include_str!("assets/tilesets/onewilliamdollars.tsj"), include_bytes!("assets/tilesets/onewilliamdollars.jpg")).unwrap();
        let level = level2d::tiled::Level::new(include_str!("assets/levels/test.tmj")).unwrap();
        let mut renderer = level2d::tiled::LevelRenderer::new(ctx, &level).unwrap();
        renderer.populate(ctx, &tassets, &level).unwrap();
        Self {
            test: texture::Texture::new(ctx, include_bytes!("assets/textures/chocojdog.png")),
            shader: shader::Shader::new(
                ctx,
                include_str!("assets/shaders/flat/vert.glsl"),
                include_str!("assets/shaders/flat/frag.glsl"),
            ),
            mesh: mesh::Mesh::from_obj(ctx, include_bytes!("assets/meshes/square.obj")),
            tassets,
            level,
            renderer,
        }
    }
}
impl teleia::state::Game for Game {
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Erm<()> {
        ctx.clear();
        st.bind_2d(ctx, &self.shader);
        self.shader.set_position_2d(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            &glam::Vec2::new(100.0, 100.0),
        );
        self.test.bind(ctx);
        self.mesh.render(ctx);
        self.renderer.render(ctx, &self.tassets, &self.level).unwrap();
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn main_js() {
    teleia::run(240, 160, teleia::Options::empty(), Game::new).await;
}
