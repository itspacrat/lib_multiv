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
        cell::RefCell,
        collections::HashMap,
        path::{Path, PathBuf},
    },
    strum::{FromRepr},
    tokio::fs::{create_dir_all, read_to_string, write, OpenOptions},
    //shvft_mapper::parsemap
};

/// Position type for the turtle (should be usize)
pub type Pos = usize;
pub type TurtAttrs = Vec<TurtAttr>;
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, FromRepr)]
#[repr(u8)]
pub enum TileDatum {
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
#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub enum TurtAttr {
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
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftContainer {
    pub pos: Pos,
    pub inventory: Vec<TileDatum>,
}
/// requires attribute "opendoor"
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftNote {
    pub pos: Pos,
    pub content: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftDoor {
    pub here: Pos,
    pub there: Pos,
    pub exit_map: String,
    pub exit_direction: char,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftSwitch {
    pub state: bool,
    pub map: String,
    pub here: Pos,
    pub there: Pos,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftMap {
    pub width: Pos,
    pub tiles: Vec<TileDatum>,
    pub doors: Vec<ShvftDoor>,
    pub notes: Vec<ShvftNote>,
    pub containers: Vec<ShvftContainer>,
}
pub type ShvftRgb = [u8; 3];
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DbItem {
    pub description: String,
    pub rgb: ShvftRgb,
    pub attributes: TurtAttrs,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftInfo {
    pub current_map: String,
    pub nametag: String,
    pub db: HashMap<String, DbItem>,
    pub pos: Pos,
    pub inventory: Vec<TileDatum>,
    pub rail: Vec<u8>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftTurtle {
    pub domain: String,
    pub info: ShvftInfo,
    pub map: ShvftMap,
}
impl ShvftTurtle {
    pub async fn mv(&mut self, dirs: Vec<char>) {
        let Self {
            info: refinfo,
            map: refmap,
            ..
        } = self;
        let mut mv_out: Pos;
        for d in dirs.iter() {
            let attrs_next = get_attrs(
                &refinfo.db,
                refmap.tiles[next_pos(*d, &refinfo.pos, &refmap)],
            )
            .unwrap();

            if attrs_next.contains(&TurtAttr::NoPassThrough) {
                if attrs_next.contains(&TurtAttr::Push) {
                    let next = next_pos(*d, &refinfo.pos, refmap);
                    let mut dummy: TileDatum = TileDatum::from_repr(0).unwrap();

                    // swap push tiles
                    dummy = refmap.tiles[refinfo.pos];
                    refmap.tiles[refinfo.pos] = refmap.tiles[next];
                    refmap.tiles[next] = dummy;

                    mv_out = next_pos(*d, &(refinfo.pos), &refmap)
                } else {
                    mv_out = refinfo.pos;
                }
            } else {
                mv_out = next_pos(*d, &(refinfo.pos), &refmap);
            }

            refinfo.pos = mv_out;
        }
        let _ = write(
            format!(
                "domains/{}/maps/{}/map.json",
                self.domain, self.info.current_map
            ),
            to_string_pretty(&refmap).unwrap(),
        )
        .await
        .unwrap();
    }

    pub fn container_swap(&mut self, box_dir: char, give_index: usize, take_index: usize) {
        

        let Self {
            info: ShvftInfo { pos: refpos, .. },
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
        ()
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
fn get_attrs(db: &HashMap<String, DbItem>, item_id: TileDatum) -> Result<&[TurtAttr]> {
    Ok(&db
        .get(&format!("{:?}", item_id))
        .context("NOT_VALID_ITEM")?
        .attributes)
}
//
/// creates a new instance of a turtle from default values at domain
pub async fn new(domain: String) -> ShvftTurtle {
    let current_map = read_to_string(format!("domains/{}/info/current_map.txt", &domain))
        .await
        .unwrap();
    let map: ShvftMap = from_str(
        &read_to_string(&format!(
            "domains/{}/maps/{}/map.json",
            domain, &current_map
        ))
        .await
        .unwrap(),
    )
    .unwrap();

    ShvftTurtle {
        domain: domain.clone(),
        info: ShvftInfo {
            current_map: current_map.to_owned(),
            nametag: String::from("turtle"),
            db: from_str(&read_to_string("domains/global/item_db.json").await.unwrap()).unwrap(),
            pos: from_str(
                &read_to_string(format!("domains/{}/info/pos.json", &domain))
                    .await
                    .unwrap(),
            )
            .unwrap(),
            inventory: from_str(
                &read_to_string(format!("domains/{}/info/inv.json", &domain))
                    .await
                    .unwrap(),
            )
            .unwrap(),
            rail: from_str(
                &read_to_string(format!("domains/{}/info/rail.json", &domain))
                    .await
                    .unwrap(),
            )
            .unwrap(),
        },
        map: map.to_owned(),
    }
}
/// sets & returns the potential map index given a direction and a ShvftMap reference
pub fn next_pos(dir: char, c_pos: &Pos, data: &ShvftMap) -> Pos {
    let height = ((data.width as f32 / data.tiles.len() as f32).ceil()) as usize;
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
