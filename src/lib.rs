use {
    serde::{Deserialize, Serialize},
    strum::{FromRepr},
    std::{collections::HashMap,cell::RefCell,path::{Path, PathBuf},},
    serde_json::{from_str,to_string_pretty,to_string},
    anyhow::{Context,Error,Result},
    image::{imageops::{resize,FilterType},Pixel,RgbImage,ImageBuffer},
    tokio::fs::{create_dir_all, read_to_string, write, OpenOptions},
};

/// Position type for the turtle (should be usize)
pub type Pos = usize;
//pub type Pos = Pos;
pub type TurtAttrs = Vec<TurtAttr>;
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, FromRepr)]
#[repr(u8)]
enum TileDatum {
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
    EndpointTile = 15
}
#[derive(PartialEq,Deserialize,Serialize,Clone,Debug)]
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
    pub inventory: Vec<u8>,
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
    pub width:Pos,
    pub tiles: Vec<u8>,
    pub doors: Vec<ShvftDoor>,
    pub notes: Vec<ShvftNote>,
    pub containers: Vec<ShvftContainer>,
}
//#[derive(Deserialize, Serialize, Debug, Clone)]
pub type ShvftRgb = [u8;3];
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
    pub inventory: Vec<u8>,
    pub rail: Vec<u8>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftTurtle {
    pub domain: String,
    pub info: ShvftInfo,
    pub map: ShvftMap,
}
impl ShvftTurtle {
    
}
//
/// returns a mutable reference to a `ShvftContainer` via an index
fn get_container_mut<'a>(
    containers: &'a Vec<ShvftContainer>,
    pos: &'a Pos,
) -> Result<&'a mut ShvftContainer> {
    todo!()
}
fn get_attrs(db: &HashMap<String, DbItem>, item_id: u8) -> Result<&[TurtAttr]> {
    Ok(&db
        .get(&format!("{}", item_id))
        .context("NOT_VALID_ITEM")?
        .attributes)
}
//
/// creates a new instance of a turtle from default values at domain
pub async fn new(domain: String) -> ShvftTurtle {
    let current_map = read_to_string(format!("domains/{}/info/current_map.txt", &domain)).await.unwrap();
    let map: ShvftMap = from_str(
        &read_to_string(&format!(
            "domains/{}/maps/{}/map.json",
            domain, &current_map
        )).await
        .unwrap(),
    )
    .unwrap();

    ShvftTurtle {
        domain: domain.clone(),
        info: ShvftInfo {
            current_map: current_map.to_owned(),
            nametag: String::from("turtle"),
            db: from_str(&read_to_string("domains/global/item_db.json").await.unwrap()).unwrap(),
            pos: from_str(&read_to_string(format!("domains/{}/info/pos.json", &domain)).await.unwrap())
                .unwrap(),
            inventory: from_str(
                &read_to_string(format!("domains/{}/info/inv.json", &domain)).await.unwrap(),
            )
            .unwrap(),
            rail: from_str(&read_to_string(format!("domains/{}/info/rail.json", &domain)).await.unwrap())
                .unwrap(),
        },
        map: map.to_owned(),
    }
}
/// sets the potential positional value given a direction and a ShvftMap reference
pub fn next_pos(dir: char, c_pos: &Pos, data: &ShvftMap) -> Pos {
    let height = (
        (data.width as f32 / data.tiles.len() as f32).ceil()
    ) as usize;
    let potential_pos: Pos;
    match dir {
        'n' | 'N'  => {
            potential_pos = if!(*c_pos <= data.width - 1) {*c_pos - data.width} else {*c_pos};
        }
        's' | 'S' => {
            potential_pos = if!(*c_pos  >= (data.tiles.len() - 1) - data.width) {*c_pos + data.width} else {*c_pos};
        }
        'e' | 'E' => {
            potential_pos = if !(((*c_pos + 1) % data.width) == 0) {*c_pos + 1 as Pos} else {*c_pos};
        }
        'w' | 'W' => {
            potential_pos = if !(((*c_pos - 1) % 0) == data.width - 1 as usize) {*c_pos - 1 as Pos} else {*c_pos};
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
