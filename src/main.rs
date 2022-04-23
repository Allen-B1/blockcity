#[macro_use] extern crate rocket;
#[macro_use] extern crate itertools;
use lazy_static::lazy_static;
use rand::Rng;
use rocket::response::content::{Html, JavaScript};
use rocket::serde::json::Json;
use std::{fs, collections::HashMap, sync::{RwLock,Mutex}};
use serde::Serialize;

type BlockID = u32;
type UserID = u32;

#[derive(Clone, Serialize)]
#[serde(tag = "type")]
enum Block {
    User,
    Video { source: String }, // vimeo/YT
    Wall
}

#[derive(Clone, Serialize)]
struct User {
    name: String,
    block_id: BlockID
}

#[derive(Clone, Default, Serialize)]
struct City {
    pub owner: Option<UserID>,
    pub users: HashMap<UserID, User>,
    pub blocks: HashMap<BlockID, Block>,
    pub block_owners: HashMap<BlockID, UserID>, 
    pub positions: HashMap<BlockID, (i32, i32)>

    /* 
constraints:
 - owner: may be None; cannot refer to invalid user
 - users: entry for each user
 - blocks: must have entry for each block
 - block_owners: may not have entry; may point to invalid user
 - positions: if no entry, assume (0, 0); may have invalid entries
    */
}

impl City {
    pub fn insert_block(&mut self, block: Block, position: (i32, i32), owner: Option<UserID>) -> BlockID {
        let mut rng = rand::thread_rng();
        let block_id = rng.gen();
        self.blocks.insert(block_id, block);
        self.positions.insert(block_id, position);
        if let Some(owner) = owner {
            self.block_owners.insert(block_id, owner);
        }
        block_id
    }

    pub fn new() -> City {
        let mut city: City = Default::default();
        let mut rng = rand::thread_rng();
        city.insert_block(
            Block::Wall, 
            (rng.gen_range(2..5), rng.gen_range(2..5)),
            None);
        city
    }

    pub fn join_user(&mut self, name: String) -> UserID {
        let mut rng = rand::thread_rng();
        let id = rng.gen();
        let block_id = rng.gen();

        self.users.insert(id, User { name, block_id });
        self.blocks.insert(block_id, Block::User);
        self.block_owners.insert(block_id, id);

        if self.owner == None {
            self.owner = Some(id);
        }

        id
    }

    pub fn leave_user(&mut self, user_id: UserID) {
        let block_id = self.users.get(&user_id).map(|user| user.block_id);
        if let Some(block_id) = block_id {
            self.blocks.remove(&block_id);
            self.positions.remove(&block_id);
        }
        self.users.remove(&user_id);

        // if owner, give owner to random person
        if Some(user_id) == self.owner {
            self.owner = self.users.iter().next().map(|(id, data)| *id);
        }
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
        city.leave_user(user_id);
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

#[post("/api/<city_id>/move?<user_id>&<block_id>&<x>&<y>")]
fn api_move(user_id: UserID, city_id: u32, block_id: BlockID, x: i32, y: i32) -> Result<&'static str, String> {
    let mut cities = CITIES.write().map_err(|e| format!("internal poison: {:?}", e))?;
    let city = cities.get_mut(&city_id);
    if let Some(city) = city {
        let mut has_perms = false;
        if Some(user_id) == city.owner {
            has_perms = true;
        }
        if let Some(owner) = city.block_owners.get(&block_id) {
            if *owner == user_id {
                has_perms = true;
            }
        }

        if has_perms {
            city.positions.insert(block_id, (x, y));
            Ok("")
        } else {
            Err("insufficient permissions".to_string())
        }
    } else {
        Err("city doesn't exist".to_string())
    }
}

#[post("/api/<city_id>/create?<user_id>&<block_type>")]
fn api_create(city_id: u32, user_id: UserID, block_type: String) -> Result<String, String> {
    unimplemented!()
}

#[get("/<_city_id>")]
fn city(_city_id: u32) -> Result<Html<String>, String> {
    fs::read_to_string("public/index.html").map_err(|e| format!("internal error: {:?}", e)).map(Html)
}

#[get("/build/<file>")]
async fn bundle(file: String) -> Result<rocket::fs::NamedFile, String> {
    rocket::fs::NamedFile::open(format!("public/build/{}", file) ).await.map_err(|e| format!("internal error: {:?}", e))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![bundle, city, api_join, api_get, api_move, api_leave])
}