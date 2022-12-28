use {
    serde::{Deserialize, Serialize},
    strum::{FromRepr},
    std::collections::HashMap,
};
pub type Pos = usize;
pub type Pos2D = [Pos; 2];
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
#[derive(Deserialize,Serialize,Clone,Debug)]
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
    pub pos: Pos2D,
    pub inventory: Vec<u8>,
}
/// requires attribute "opendoor"
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftNote {
    pub pos: Pos2D,
    pub content: String,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftDoor {
    pub here: Pos2D,
    pub there: Pos2D,
    pub exit_map: String,
    pub exit_direction: char,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftSwitch {
    pub state: bool,
    pub map: String,
    pub here: Pos2D,
    pub there: Pos2D,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftMap {
    pub tiles: Vec<Vec<u8>>,
    pub doors: Vec<ShvftDoor>,
    pub notes: Vec<ShvftNote>,
    pub containers: Vec<ShvftContainer>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
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
    pub h_v: Pos2D,
    pub inventory: Vec<u8>,
    pub rail: Vec<u8>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShvftTurtle {
    pub domain: String,
    pub info: ShvftInfo,
    pub map: ShvftMap,
}
