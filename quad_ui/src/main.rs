use ahash::RandomState;
use dt_lib::{
    battle::{army::*, battlefield::*, troop::Troop},
    items::item::*,
    locale::{parse_locale, Locale},
    map::{
        event::{execute_event, Event as GameEvent, Execute},
        map::*,
        object::ObjectInfo,
        tile::*,
    },
    network::net::*,
    parse::{collect_errors, parse_items, parse_objects, parse_settings, parse_story, parse_units},
    time::time::Data as TimeData,
    units::{
        unit::{ActionResult, Unit, UnitPos},
        unitstats::ModifyUnitStats,
    },
};
use macroquad::{
    prelude::*,
    ui::{
        hash, root_ui,
        widgets::{self, Window},
    },
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    ops::Index,
    sync::{Mutex, RwLock},
};
#[derive(Debug)]
struct Assets {
    inner: HashMap<String, Texture2D, RandomState>,
}
impl Assets {
    fn new(inner: HashMap<String, Texture2D, RandomState>) -> Self {
        Assets { inner: inner }
    }
    fn get(&self, key: &String) -> &Texture2D {
        self.inner.get(key).expect(&format!("No asset {key}"))
    }
}
impl Index<&String> for Assets {
    type Output = Texture2D;
    fn index(&self, index: &String) -> &Self::Output {
        self.get(index)
    }
}
static LOCALE: Lazy<RwLock<Locale>> =
    Lazy::new(|| RwLock::new(Locale::new("Rus".into(), "Eng".into())));
static UNITS: Lazy<RwLock<Vec<Unit>>> = Lazy::new(|| RwLock::new(vec![]));
static OBJECTS: Lazy<RwLock<Vec<ObjectInfo>>> = Lazy::new(|| RwLock::new(vec![]));
macro_rules! units {
    () => {
        &UNITS.read().unwrap()
    };
}
macro_rules! units_mut {
    () => {
        &mut UNITS.write().unwrap()
    };
}
macro_rules! objects {
    () => {
        &OBJECTS.read().unwrap()
    };
}
macro_rules! objects_mut {
    () => {
        &mut OBJECTS.write().unwrap()
    };
}
#[derive(Debug)]
struct State {
    pub assets: Assets,
    pub game: Game,
    pub ui: Ui,
}
impl State {
    fn new() -> Self {
        todo!()
    }
}
async fn load_assets(req_assets_list: &[(&str, Vec<String>)]) -> Assets {
    let mut asset_names = Vec::new();
    let mut assets = Vec::new();
    let mut error_collector: Vec<String> = Vec::new();
    for req_assets in req_assets_list {
        for asset in &req_assets.1 {
            assets.push(collect_errors(
                load_texture(&format!("{}/{}", req_assets.0, &asset)).await,
                &mut error_collector,
                "Failed to load image asset",
            ));
            asset_names.push(asset.clone());
        }
    }
    let Ok(assets) = assets
        .into_iter()
        .map(|v| v.ok_or(()))
        .collect::<Result<Vec<Texture2D>, ()>>()
    else {
        panic!("No asssets!")
    };
    println!("{}", error_collector.join("\n"));
    Assets::new(asset_names.into_iter().zip(assets).collect())
}
async fn game_init() -> State {
    let settings = parse_settings();
    {
        let locale = &mut LOCALE.write().unwrap();
        locale.set_lang((&settings.locale, &settings.additional_locale));
        parse_locale(&[&settings.locale, &settings.additional_locale], locale);
    }
    let assets = {
        let req_assets_items = parse_items(None, &settings.locale);
        let res = parse_units(None);
        if let Err(err) = res {
            error!("{}", err);
            panic!("{}", err);
        }
        let Ok((mut units, req_assets_units)) = res else {
            panic!("Unit parsing error")
        };
        units_mut!().append(&mut units);
        let (mut objects, req_assets_objects) = parse_objects();
        objects_mut!().append(&mut objects);
        dbg!(units!().len(), objects!().len());
        let req_assets_tiles = (
            "assets/Terrain",
            TILES.iter().map(|tile| tile.sprite().to_string()).collect(),
        );
        let req_assets_list = [
            req_assets_objects,
            req_assets_items,
            req_assets_units,
            req_assets_tiles,
        ];
        load_assets(&req_assets_list).await
    };
    let (mut gamemap, events) = parse_story(
        units!(),
        objects!(),
        &settings.locale,
        &settings.additional_locale,
    );
    gamemap.calc_hitboxes(objects!());
    State {
        assets,
        ui: Ui {
            main: Menu::Main,
            stack: Vec::new(),
        },
        game: Game::Single(Scenario {
            gamemap,
            battle: None,
            events,
        }),
    }
}
#[derive(Debug)]
struct Scenario {
    pub gamemap: GameMap,
    pub battle: Option<BattleInfo>,
    pub events: Vec<GameEvent>,
}
#[derive(Debug)]
enum Game {
    Single(Scenario),
    Online(ConnectionManager),
}
fn window_conf() -> Conf {
    Conf {
        high_dpi: true,
        window_title: "DT REMASTERED".into(),
        ..Default::default()
    }
}
#[derive(Debug)]
enum Menu {
    Main,
    Map(Camera2D),
    Atlas,
    Battle,
    EventMessage,
}
#[derive(Debug)]
struct Ui {
    pub main: Menu,
    pub stack: Vec<Menu>,
}
const SIZE: (f32, f32) = (256., 242.);
fn draw_map(assets: &Assets, camera: &mut Camera2D, game: &mut Game) {
    let gamemap = match game {
        Game::Single(Scenario {
            gamemap,
            battle,
            events,
        }) => gamemap,
        Game::Online(conn) => &mut conn.gamemap,
    };
    set_camera(camera);
    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            let tile = TILES[gamemap.tilemap[(i, j)]];
            draw_texture(
                assets.get(&tile.sprite().to_string()),
                i as f32 * SIZE.0,
                j as f32 * SIZE.1,
                WHITE,
            );
        }
    }
    draw_text("Loading game map...", 0., 0., 0.5, BLACK);
    if is_key_down(KeyCode::W) {
        camera.offset += Vec2::new(0., -0.01)
    }
    if is_key_down(KeyCode::D) {
        camera.offset += Vec2::new(-0.01, 0.);
    }
    if is_key_down(KeyCode::A) {
        camera.offset += Vec2::new(0.01, 0.);
    }
    if is_key_down(KeyCode::S) {
        camera.offset += Vec2::new(0., 0.01);
    }
    if is_key_down(KeyCode::Equal) {
        camera.zoom += Vec2::new(0.001, 0.001);
    }
    if is_key_down(KeyCode::Minus) {
        camera.zoom -= Vec2::new(0.001, 0.001);
    }
}
fn draw_menu(state: &mut State) {
    match &mut state.ui.main {
        Menu::Main => {
            Window::new(hash!(), vec2(0., 0.), vec2(screen_height(), screen_width()))
                .titlebar(false)
                .ui(&mut *root_ui(), |ui| {
                    ui.label(Some((50., 50.).into()), "Discord Times");
                    if ui.button(Some((50., 100.).into()), "Start") {
                        state.ui.main = Menu::Map(Camera2D::from_display_rect(Rect::new(
                            0.,
                            0.,
                            screen_width(),
                            screen_height(),
                        )));
                    }
                    if ui.button(Some((50., 150.).into()), "Atlas") {
                        state.ui.main = Menu::Atlas;
                    }
                });
        }
        Menu::Map(camera) => {
            draw_map(&state.assets, camera, &mut state.game);
        }
        Menu::Atlas => draw_texture(
            state
                .assets
                .inner
                .values()
                .skip(macroquad::time::get_time() as usize % state.assets.inner.len())
                .next()
                .unwrap(),
            0.,
            0.,
            WHITE,
        ),
        _ => {}
    }
}
fn draw_ui(state: &mut State) {
    draw_menu(state);
    //for menu in state.ui.stack.clone() {
    //draw_menu(menu, state);
    //}
}
#[macroquad::main(window_conf)]
async fn main() {
    clear_background(WHITE);
    draw_text(
        "Loading game assets...",
        0.,
        screen_height() / 2.,
        20.,
        BLACK,
    );
    next_frame().await;
    let mut state = game_init().await;
    loop {
        clear_background(WHITE);
        draw_ui(&mut state);
        next_frame().await
    }
}
