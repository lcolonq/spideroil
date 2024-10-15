#![allow(dead_code, unused_variables)]
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use teleia::*;

mod tiles;

struct Tilesheet {
    texture: texture::Texture,
    dim: usize,
    w: usize, h: usize,
}

impl Tilesheet {
    fn new(ctx: &context::Context, dim: usize, w: usize, h: usize, bytes: &[u8]) -> Self {
        Self {
            dim, w, h,
            texture: texture::Texture::new(ctx, bytes),
        }
    }
}

struct Assets {
    font: font::Font,
    shader_flat: shader::Shader,
    shader_sheet: shader::Shader,
    mesh_square: mesh::Mesh,
    tilesheet_town: Tilesheet,
}

impl Assets {
    fn new(ctx: &context::Context) -> Self {
        Self {
            font: font::Font::new(ctx),
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
            mesh_square: mesh::Mesh::new(ctx, include_bytes!("assets/meshes/square.obj")),
            tilesheet_town: Tilesheet::new(ctx, 16, 13, 4, include_bytes!("assets/textures/fftown.png")),
        }
    }
}

#[derive(Debug)]
struct PageEntity {
    message: Option<String>,
}

impl PageEntity {
    fn new(v: lexpr::Value) -> Self {
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
    fn new(v: lexpr::Value) -> Self {
        Self {
            image: None,
            color: None,
        }
    }
}

#[derive(Debug)]
struct Page {
    url: String,
    tree: lexpr::value::Value,
    width: usize, height: usize,
    map: HashMap<(usize, usize), String>,
    links: HashMap<(usize, usize), String>,
    entities: HashMap<(f32, f32), PageEntity>,
    tiles: HashMap<String, PageTile>,
}

impl Page {
    fn new(url: String, tree: lexpr::value::Value) -> Self {
        Self {
            url,
            tree,
            width: 0, height: 0,
            map: HashMap::new(),
            links: HashMap::new(),
            entities: HashMap::new(),
            tiles: HashMap::new(),
        }
    }
}

struct Game {
    assets: Assets,
    tiles: tiles::Tiles,
    page: Option<Page>,
}

impl Game {
    async fn new(ctx: &context::Context) -> Self {
        Self {
            assets: Assets::new(ctx),
            tiles: tiles::Tiles::new(),
            page: None,
        }
    }

    fn load<U>(&self, ctx: &context::Context, st: &mut state::State, url: U) where U: reqwest::IntoUrl {
        let u = url.into_url().expect("failed to convert to URL");
        st.request(|c| {
            c.get(u.clone())
        });
    }

    fn render_tile_index(
        &self, ctx: &context::Context, st: &mut state::State,
        ts: &Tilesheet, tx: usize, ty: usize, pos: glam::Vec2
    ) {
        let dim = ts.dim as f32;
        let w = 1.0 / ts.w as f32;
        let h = 1.0 / ts.h as f32;
        st.bind_2d(ctx, &self.assets.shader_sheet);
        self.assets.shader_sheet.set_f32(ctx, "tile_w", w);
        self.assets.shader_sheet.set_f32(ctx, "tile_h", h);
        self.assets.shader_sheet.set_f32(ctx, "tile_x", tx as f32 * w);
        self.assets.shader_sheet.set_f32(ctx, "tile_y", ty as f32 * h);
        self.assets.shader_sheet.set_position_2d(ctx, &pos, &glam::Vec2::new(dim, dim));
        ts.texture.bind(ctx);
        self.assets.mesh_square.render(ctx);
    }

    fn get_tilesheet(&self, t: &tiles::Tileset) -> &Tilesheet {
        match t {
            tiles::Tileset::Town => &self.assets.tilesheet_town,
        }
    }
    fn render_tile(&self, ctx: &context::Context, st: &mut state::State, tile: &str, pos: glam::Vec2) {
        if let Some(tiles::Tiledef(t, x, y)) = self.tiles.tiles.get(tile) {
            self.render_tile_index(ctx, st, self.get_tilesheet(&t), *x, *y, pos)
        } else {
            log::warn!("could not find tile: {}", tile);
        }
    }
}

impl teleia::state::Game for Game {
    fn initialize_audio(&self, ctx: &context::Context, st: &state::State, actx: &audio::Context) -> HashMap<String, audio::Audio> {
        HashMap::from_iter(vec![
            ("test".to_owned(), audio::Audio::new(&actx, include_bytes!("assets/audio/test.wav"))),
        ])
    }
    fn finish_title(&mut self, _st: &mut state::State) {
    }
    fn request_return(&mut self, ctx: &context::Context, st: &mut state::State, res: state::Response) {
        let str = String::from_utf8_lossy(&res.body).into_owned();
        let url = res.url;
        if let Ok(tree) = lexpr::from_str(&str) {
            self.page = Some(Page::new(url, tree));
            log::info!("loaded page: {:?}", self.page);
        } else {
            log::warn!("failed to decode page: {}", url);
        }
    }
    fn update(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        if self.page.is_none() && !st.requesting() {
            self.load(ctx, st, "https://pub.colonq.computer/~llll/spider/test.sp");
        }
        Some(())
    }
    fn render(&mut self, ctx: &context::Context, st: &mut state::State) -> Option<()> {
        ctx.clear();
        self.assets.font.render_text(
            ctx,
            &glam::Vec2::new(0.0, 0.0),
            "hello spider oil",
        );
        self.render_tile(ctx, st, "grass", glam::Vec2::new(0.0, 0.0));
        self.render_tile(ctx, st, "well", glam::Vec2::new(40.0, 0.0));
        Some(())
    }
}

#[wasm_bindgen]
pub async fn main_js() {
    teleia::run(Game::new).await;
}
