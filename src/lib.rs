use std::mem::take;

use anyhow::ensure;

use {
    anyhow::{Context, Error, Result},
    image::{
        imageops::{resize, FilterType},
        ImageBuffer, Pixel, RgbImage,
    },
    serde::{Deserialize, Serialize},
    serde_json::{from_str, to_string, to_string_pretty},
    std::{
        //cell::RefCell,
        collections::HashMap,
        path::{Path, PathBuf},
    },
    strum::{FromRepr},
    tokio::fs::{create_dir_all, read_to_string, write, OpenOptions},
    //shvft_mapper::parsemap
};

/// Position type for the player (should be usize)
pub type Pos = usize;
#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum MvTileAttribute {
    Null,
    NoPassThrough,
    Push,
    Door,
    Locked,
    OpenDoor,
    StoreItems,
    Take,
    Drop,
    Equip,
    Read,
    Write,
    State,
}
pub type MvTileAttributes = Vec<MvTileAttribute>;

pub type MvRGB = [u8; 3];
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DbItem {
    pub description: String,
    pub rgb: MvRGB,
    pub attributes: MvTileAttributes,
}
// todo: necessary??? idk
pub type DB = HashMap<String,DbItem>;

fn get_attrs(db: &HashMap<String, DbItem>, item_id: u8) -> Result<&[MvTileAttribute]> {
    Ok(&db
        .get(&format!("{:?}", item_id))
        .context("NOT_VALID_ITEM")?
        .attributes)
}

/// ### MvRoom
/// 
/// the room typestruct has a designated list of access keys, 
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvRoom {
    /// ## Room keys
    /// #### Array assignment
    /// keys\[0] is the vector of room-denied (blacklisted) keys,
    /// 
    /// keys\[1] is the vector of room-accepted (whitelisted) keys
    ///
    /// #### use order
    /// rooms should check if a player is holding a blacklisted 
    /// key first, and then check if any of their keys are whitelisted after.
    pub keys: [Vec<String>;2],
    pub id: String,
    pub tiles: Vec<u8>,
    pub width: usize
}
//
/// creates a new instance of a turtle from default values at key
/// sets & returns the potential map index given a direction and an MvRoom reference
pub fn next_pos(dir: char, c_pos: &Pos, data: &MvRoom) -> Pos {
    //let height = ((data.width as f32 / data.tiles.len() as f32).ceil()) as usize;
    let potential_pos: Pos;
    match dir {
        'n' | 'N' => {
            potential_pos = if !(*c_pos <= data.width - 1) {
                *c_pos - data.width
            } else {
                *c_pos
            };
        }
        's' | 'S' => {
            potential_pos = if !(*c_pos >= (data.tiles.len() - 1) - data.width) {
                *c_pos + data.width
            } else {
                *c_pos
            };
        }
        'e' | 'E' => {
            potential_pos = if !(((*c_pos + 1) % data.width) == 0) {
                *c_pos + 1 as Pos
            } else {
                *c_pos
            };
        }
        'w' | 'W' => {
            potential_pos = if !(((*c_pos - 1) % data.width) == data.width - 1 as usize) {
                *c_pos - 1 as Pos
            } else {
                *c_pos
            };
        }
        '.' | 'h' | 'H' | _ => {
            potential_pos = *c_pos;
        }
    }
    potential_pos
}
/// ### MvPlayer
#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct MvPlayer {
    pub keys: Vec<String>,
    /// a string denoting the file name (without the file extension) 
    /// of the room the player is currently in
    pub room_id: String,
    /// a player's position in the room the player is currently in
    pub position: Pos,
    /// the inventory stores items a player has taken
    pub inventory: Vec<u8>,
    /// the rail holds up to 3 of a player's equippable items
    pub rail: Vec<u8>,
}
/*
!       BEGIN PLAYER + ROOM IMPL
*/
//
impl MvPlayer {
    /// should be called after checking whether or not a player exists for a given player key.
    /// 
    /// instantiates a new player with a given player key + default values
    pub fn new(default_key:String) -> Self {
        Self {
            // start keyring with user's key
            keys: vec![default_key.clone()],
            room_id: default_key.clone(),
            position: 24, // center of the default map
            inventory:vec![0,0,0,0,0,0,0], // empty
            rail: vec![0,0,0] // empty
        }
    }
    //
    /// loads playerdata from a file matching the current player's key
    pub async fn from_existing(default_key: String) -> MvPlayer {
        
        let player: MvPlayer =  from_str(
            &read_to_string(
                format!("players/{}/data.json",default_key.clone())
            ).await.unwrap()
        ).unwrap();

        player
    }
    //
    /// handles moving the palyer in a room
    pub async fn mv(&mut self, room: &mut MvRoom, db: &DB, dirs: Vec<char>) {
        let mut mv_out: Pos;
        for d in dirs.iter() {
            let attrs_next = get_attrs(
                db,
                room.tiles[next_pos(*d, &self.position,room)],
            )
            .unwrap();

            if attrs_next.contains(&MvTileAttribute::NoPassThrough) {
                if attrs_next.contains(&MvTileAttribute::Push) {
                    let next = next_pos(*d, &self.position, room);
                    let mut dummy: u8 = 0_u8;

                    // swap push tiles
                    dummy = room.tiles[self.position];
                    room.tiles[self.position] = room.tiles[next];
                    room.tiles[next] = dummy;

                    mv_out = next_pos(*d, &(self.position), room)
                } else {
                    mv_out = self.position;
                }
            } else {
                mv_out = next_pos(*d, &(self.position), &room);
            }

            self.position = mv_out;
        }
        let _ = write(
            format!(
                "keys/{}/maps/{}/data.json",
                self.keys[0], self.room_id
            ),
            to_string_pretty(room).unwrap(),
        )
        .await
        .unwrap();
    }
}
//
impl MvRoom {
    pub fn new(default_key: String) -> Self {
        Self {
            keys: [vec![],vec![]],
            id: default_key.clone(),
            tiles: vec![
                // default map, 7x7
                // empty room with solid walls
                2,2,2,2,2,2,2,
                2,1,1,1,1,1,2,
                2,1,1,1,1,1,2,
                2,1,1,1,1,1,2,
                2,1,1,1,1,1,2,
                2,2,2,2,2,2,2
            ],
            // define width for next_pos()
            width: 7
        }
    }
    pub async fn from_existing(default_key: String) -> MvRoom {
        let room: MvRoom = from_str(
            &read_to_string(format!("rooms/{}/data.json",default_key.clone())).await.unwrap()
        ).unwrap();
        room
    }
}
// blake put his blood sweat and tears into this, do not give 
// it the disrespect of commenting it out
/// returns the index of the first free slot in the player's inventory
pub fn ck_inv_empty(inventory: &mut Vec<u8>) -> Option<usize> {
    let out: Option<usize>;
    let mut free_slots: Vec<usize> = Vec::new();
    for (index, slot) in inventory.iter().enumerate() {
        if slot == &0 {
            println!("o  {} slot free.", &index);
            free_slots.push(index);
        } else {
            // dont push
            println!("x {} slot full.", &index)
        }
    }
    if free_slots.len() > 0 {
        out = Some(free_slots[0])
    } else {
        out = None
    }
    out
}

/* 
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, FromRepr)]
#[repr(u8)]
pub enum MvTile {
    NullTile = 0,
    FloorTile = 1,
    WallTile = 2,
    WorkBenchTile = 3,
    ClosedDoorTile = 4,
    OpenDoorTile = 5,
    WoodCrateTile = 6,
    CardboardBoxTile = 7,
    NoteTile = 8,
    ComputerTile = 9,
    ScannerUpgrade = 10,
    ScannerIIUpgrade = 11,
    LockedDoorTile = 12,
    LockedIIDoorTile = 13,
    SwitchTile = 14,
    EndpointTile = 15,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvBox {
    pub pos: Pos,
    pub inventory: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvNote {
    pub pos: Pos,
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvDoor {
    pub here: Pos,
    pub there: Pos,
    pub exit_map: String,
    pub exit_direction: char,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvPlayerInfo {
    pub current_map: String,
    pub nametag: String,
    pub db: HashMap<String, DbItem>,
    pub pos: Pos,
    pub inventory: Vec<u8>,
    pub rail: Vec<u8>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MvPlayer {
    pub key: String,
    pub info: MvPlayerInfo,
    pub map: MvRoom,
}
impl MvPlayer {

    pub fn container_swap(&mut self, box_dir: char, give_index: usize, take_index: usize) {
        

        let Self {
            info: MvPlayerInfo { pos: refpos, .. },
            map: refmap,
            ..
        } = self;
        let mut refcontainers = refmap.containers.clone();
        let np = next_pos(box_dir, &refpos.clone(), refmap);

        let container_ref = get_container_mut(
            &mut refcontainers,
            refpos,
            &np,
        )
        .context("no container found :(")
        .unwrap();

        std::mem::swap(
            &mut container_ref.inventory[take_index],
            &mut self.info.inventory[give_index],
        );

        let _ = write(
            format!(
                "keys/{}/maps/{}/map.json",
                self.key, self.info.current_map
            ),
            to_string_pretty(&refmap).unwrap(),
        );
        let _ = write(
            format!(
                "keys/{}/info/inv.json",
                self.key
            ),
            to_string_pretty(&self.info.inventory).unwrap(),
        );

    }
}
//
/// returns a mutable reference to a `ShvftContainer` via an index
fn get_container_mut<'a>(
    containers: &'a mut Vec<ShvftContainer>,
    pos: &'a Pos,
    destination_pos: &'a Pos,
) -> Option<&'a mut ShvftContainer> {

    let mut count: usize = 0;
    let mut out_idx: usize = 0;

    for (i, c) in containers.iter_mut().enumerate() {
        if c.pos == *destination_pos {
            count += 1;
            out_idx = i;
        } else {
        }
    }

    if count == 1 {
        Some(&mut containers[out_idx])
    } else {
        None
    }
}
*/