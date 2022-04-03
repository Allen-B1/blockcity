#[macro_use] extern crate rocket;
#[macro_use] extern crate itertools;
use lazy_static::lazy_static;
use rand::Rng;
use rocket::response::content::{Html};
use rocket::serde::json::Json;
use std::{fs, collections::HashMap, sync::{RwLock,Mutex}};
use serde::Serialize;

type BlockID = u32;
type UserID = u32;

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
enum Block {
    User { user_id: UserID },
    Video { source: String }, // vimeo/YT
    Wall { width: u16, height: u16 }
}

#[derive(Clone, Serialize)]
struct User {
    name: String,
    block_id: BlockID
}

#[derive(Clone, Default, Serialize)]
struct City {
    pub users: HashMap<UserID, User>,
    pub blocks: HashMap<BlockID, Block>,
    pub positions: HashMap<BlockID, (i32, i32)>
}

impl City {
    pub fn insert_block(&mut self, block: Block, position: (i32, i32)) -> BlockID {
        let mut rng = rand::thread_rng();
        let block_id = rng.gen();
        self.blocks.insert(block_id, block);
        self.positions.insert(block_id, position);
        block_id
    }

    pub fn new() -> City {
        let mut city: City = Default::default();
        let mut rng = rand::thread_rng();
        city.insert_block(Block::Wall{ width: rng.gen_range(7..14), height: rng.gen_range(7..14) }, (rng.gen_range(2..5), rng.gen_range(2..5)));
        city
    }

    pub fn join_user(&mut self, name: String) -> UserID {
        let mut rng = rand::thread_rng();
        let id = rng.gen();
        let block_id = rng.gen();

        self.users.insert(id, User { name, block_id });
        self.blocks.insert(block_id, Block::User { user_id: id });

        id
    }
}

lazy_static! {
    static ref CITIES: RwLock<HashMap<u32, City>> = RwLock::new(HashMap::new());
}

#[post("/api/<city_id>/join?<name>")]
fn api_join(city_id: u32, name: String) -> Result<String, String> {
    let mut cities = CITIES.write().map_err(|e| format!("internal poison: {:?}", e))?;
    if let Some(city) = cities.get_mut(&city_id) {
        let user_id = city.join_user(name);
        Ok(format!("{}", user_id))
    } else {
        let mut city: City = City::new();
        let user_id = city.join_user(name);
        cities.insert(city_id, city);
        Ok(format!("{}", user_id))
    }
}

#[post("/api/<city_id>/leave?<user_id>")]
fn api_leave(city_id: u32, user_id: UserID) -> Result<String, String> {
    let mut cities = CITIES.write().map_err(|e| format!("internal poison: {:?}", e))?;
    if let Some(city) = cities.get_mut(&city_id) {
        let block_id = city.users.get(&user_id).map(|user| user.block_id);
        if let Some(block_id) = block_id {
            city.blocks.remove(&block_id);
            city.positions.remove(&block_id);
        }
        city.users.remove(&user_id);
        Ok(format!("user {} removed", user_id))
    } else {
        Err(format!("404 :("))
    }
}

#[get("/api/<city_id>/get")]
fn api_get(city_id: u32) -> Result<Json<City>, String> {
    let cities = CITIES.write().map_err(|e| format!("internal poison: {:?}", e))?;
    let city = cities.get(&city_id);
    if let Some(city) = city {
        Ok(Json(city.clone()))
    } else {
        Err("404 :(".to_string())
    }
}

#[post("/api/<city_id>/move?<block_id>&<x>&<y>")]
fn api_move(city_id: u32, block_id: BlockID, x: i32, y: i32) -> Result<&'static str, String> {
    let mut cities = CITIES.write().map_err(|e| format!("internal poison: {:?}", e))?;
    let city = cities.get_mut(&city_id);
    if let Some(city) = city {
        city.positions.insert(block_id, (x, y));
        Ok("")
    } else {
        Err("404 :(".to_string())
    }
}

#[get("/<_city_id>")]
fn city(_city_id: u32) -> Result<Html<String>, String> {
    fs::read_to_string("static/city.html").map_err(|e| format!("internal error: {:?}", e)).map(Html)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![city, api_join, api_get, api_move, api_leave])
}