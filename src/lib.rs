use {
    anyhow::{Context, Error, Result, ensure},
    image::{
        imageops::{resize, FilterType},
        ImageBuffer, Pixel, RgbImage,
    },
    serde::{Deserialize, Serialize},
    serde_json::{from_str, to_string, to_string_pretty},
    std::{
        //cell::RefCell,
        mem::{swap, take},
        collections::HashMap,
        path::{Path, PathBuf},
    },
    tokio::fs::{create_dir_all, read_to_string, write, OpenOptions},
    room::*,
    player::*,
};
pub mod player;
pub mod room;
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
pub type DB = HashMap<String, DbItem>;

fn get_attrs(db: &HashMap<String, DbItem>, item_id: u8) -> Result<&[MvTileAttribute]> {
    Ok(&db
        .get(&format!("{:?}", item_id))
        .context("NOT_VALID_ITEM")?
        .attributes)
}


//
/// creates a new instance of a turtle from default values at key
/// sets & returns the potential map index given a direction and an MvRoom reference
pub fn next_pos(dir: char, c_pos: &Pos, data: &MvRoom) -> Pos {
    
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
    if !(free_slots.len() > 0) {
        out = None
    } else {
        out = Some(free_slots[0])
    }
    out
}

/*
// requires strum::FromRepr
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
