use {
    serde::{Deserialize, Serialize},
    hex::{decode,encode},
    strum::{FromRepr},
    std::{collections::HashMap,cell::RefCell,path::{Path, PathBuf},},
    serde_json::{from_str,to_string_pretty,to_string},
    anyhow::{Context,Error,Result},
    image::{imageops::{resize,FilterType},Pixel,RgbImage,ImageBuffer},
    tokio::fs::{create_dir_all, read_to_string, write, OpenOptions},
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
impl ShvftTurtle {
    pub async fn update_cam(&self) {
        let height_target:u32 = 600;
        // extract references to image-related data
        let Self {domain,info: ShvftInfo {db,current_map,h_v,..},map: ShvftMap {tiles,..},..} = self;
        let mut map_image_buf: RgbImage = ImageBuffer::new(tiles.len() as u32,tiles[0].len() as u32);
        for pix_y in 0..tiles[0].len() {
            for pix_x in 0..tiles.len() {
                map_image_buf.put_pixel(pix_x as u32, pix_y as u32, *Pixel::from_slice(&[
                    //r
                    db.get(&format!("{}",tiles[pix_x][pix_y])).unwrap().rgb.r,
                    //g
                    db.get(&format!("{}",tiles[pix_x][pix_y])).unwrap().rgb.g,
                    //b
                    db.get(&format!("{}",tiles[pix_x][pix_y])).unwrap().rgb.b
                ]));
            }
        }
        map_image_buf = resize(
            &map_image_buf,
            (map_image_buf.width() as f64 * (height_target as f64/map_image_buf.height() as f64)).ceil() as u32, 
            height_target, FilterType::Nearest
        );
        let cursor_colors = [
            [
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.r + 5,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.g + 5,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.b + 5
            ],/*[
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.r - 25,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.g - 25,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.b - 25
            ],*/
            [0,0,0],
            [
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.r,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.g,
                db.get(&format!("{}",tiles[h_v[0]][h_v[1]])).unwrap().rgb.b
            ],
        ];
        // do the cursor overlay thing
        let offset: [f64;2] = [
            (1 as f64/tiles.len() as f64) *
            (map_image_buf.width() as f64 * (height_target as f64/map_image_buf.height() as f64)).ceil(),
            ((1 as f64 / tiles[0].len() as f64) * height_target as f64).ceil()
        ]; // cursor data len = offset[0]*offset[1]
        
        
        for cursor_dither_y  in (((h_v[1])*offset[1] as usize) as u32..((1 + h_v[1])*offset[1] as usize) as u32) {
            for cursor_dither_x in (((h_v[0])*offset[0] as usize) as u32..((1 + h_v[0])*offset[0] as usize) as u32) {
                map_image_buf.put_pixel(
                    cursor_dither_x,
                    cursor_dither_y,
                    if (cursor_dither_x % 2 == 0) != (cursor_dither_y % 2 == 0){
                        *Pixel::from_slice(&[50,50,255])
                    } else {
                        //*Pixel::from_slice(&cursor_colors[1])
                        *Pixel::from_slice(&[255,255,0])
                    }
                    //
                   // *Pixel::from_slice(&cursor_colors[1])
                )
            }
        }

        println!(
            "dithered from ({},{}) to ({},{})",
            ((h_v[0] + 1)*offset[0] as usize) as u32,
            ((h_v[1]+ 1)*offset[1] as usize) as u32,
            (h_v[0]*offset[0] as usize) as u32,
            (h_v[1]*offset[1] as usize) as u32,
        );
        
        map_image_buf.save(&format!("domains/{domain}/info/current_map.png")).unwrap();
    }
    pub async fn post_cam(&self) -> String {
        let mut out = String::new();


        
        
        out
    }
    /// checks a tile at a given direction and position
    pub async fn ck(&mut self, direction: &char) -> String {
        let Self {
            info: ShvftInfo { db, h_v, .. },
            map:
                ShvftMap {
                    tiles,
                    notes,
                    containers,
                    ..
                },
            ..
        } = self;
        let mut peek_result: String = String::new();
        //
        // abstract to position
        let [h, v] = h_v;
        let fetch_notes = |hori: usize, vert: usize| {
            db.get(&format!("{}", tiles[hori][vert]))
                .unwrap()
                .attributes
                .contains(&TurtAttr::Read)
        };
        let fetch_doors = |hori: usize, vert: usize| {
            db.get(&format!("{}", tiles[hori][vert]))
                .unwrap()
                .attributes
                .contains(&TurtAttr::Door)
        };
        let fetch_containers = |hori: usize, vert: usize| {
            db.get(&format!("{}", tiles[hori][vert]))
                .unwrap()
                .attributes
                .contains(&TurtAttr::StoreItems)
        };
        let [potential_h, potential_v] =
            next_hv(direction.clone(), [&h, &v], [&tiles.len(), &tiles[0].len()]);
        match direction {
            'n' | 'N' | 'k' | 'K' | 's' | 'S' | 'j' | 'J' | 'e' | 'E' | 'l' | 'L' | 'w' | 'W'
            | 'h' | 'H' | '.' => {
                if fetch_doors(potential_h, potential_v) {
                    // CHECK DOOR THIS DIRECTION
                    tiles[potential_h][potential_v] = 5;
                    println!("{}", &tiles[potential_h][potential_v]);
                    peek_result = String::from(" -> 5");
                } else if fetch_notes(potential_h, potential_v) {
                    for note in notes {
                        if note.pos == [potential_h, potential_v] {
                            peek_result = note.content.to_owned();
                            println!("{}", &peek_result);
                        } else {
                            /* wrong note */
                        }
                    }
                } else if fetch_containers(potential_h, potential_v) {
                    for container in containers {
                        if container.pos == [potential_h, potential_v] {
                            peek_result = String::from(format!(
                                "container: {:?}",
                                container.inventory.to_owned()
                            ));
                            println!("{:?}", &peek_result);
                        } else {
                            /* wrong note */
                        }
                    }
                }
            }
            _ => { /* input already sanitized */ }
        }
        let result_tile_description = String::from(
            &db.get(&format!("{}", &tiles[potential_h][potential_v]))
                .unwrap()
                .description,
        );
        self.update_cam().await;
        format!("```fix\n{}```\n{}", result_tile_description, peek_result)
    }
    ///
    /// moves turtle based on a vec of cardinal directions
    pub async fn mv(&mut self, directions: Vec<char>) {
        use TurtAttr::*;
        let Self {
            domain,
            info:
                ShvftInfo {
                    h_v,
                    db,
                    current_map,
                    ..
                },
            map:
                ShvftMap {
                    tiles,
                    doors,
                    notes,
                    ..
                },
            ..
        } = self;
        //
        // load the item db
        let item_db = db;
        let doors_rc = RefCell::from(doors);
        let [h, v] = h_v;
        let [mut potential_h, mut potential_v]: Pos2D;
        //
        // hacky door update workaround
        let mut door_holder: ShvftDoor = ShvftDoor {
            here: [0, 0],
            there: [0, 0],
            exit_direction: 'x',
            exit_map: String::from("MapNone"),
        };
        //
        // check criteria for directions inside of passthrough tiles
        for direction in directions {
            [potential_h, potential_v] =
                next_hv(direction, [h, v], [&tiles.len(), &tiles[0].len()]);
            let attrs = &item_db
                .get(&String::from(&format!(
                    "{}",
                    &tiles[potential_h][potential_v].clone()
                )))
                .unwrap()
                .attributes;
            //println!("checking for wall");
            if !(attrs.contains(&NoPassThrough)) {
                //println!("checking for door");
                /* check if it's an door */
                if attrs.contains(&OpenDoor) {
                    let mut update = false;
                    for door in doors_rc.borrow_mut().iter_mut() {
                        if [*h, *v] == door.here && direction == door.exit_direction {
                            update = true;
                            door_holder = door.to_owned();
                        } else {
                            /* just move, no exit */
                            println!(
                                " move: {},{} over door: {:?}",
                                &potential_h, &potential_v, &door
                            );
                            [*h, *v] = [potential_h, potential_v];
                        }
                    }
                    if update {
                        println!("attempting exit to map {}", &door_holder.exit_map);
                        *current_map = door_holder.exit_map.clone();
                        
                        write(
                            &format!("domains/{}/info/current_map.txt", &domain),
                            current_map.clone().as_bytes(),
                        ).await
                        .unwrap();
                        let next_tiles: Vec<String> = from_str(
                            &read_to_string(&format!(
                                "domains/{}/maps/{}/tiles.json",
                                domain, &current_map
                            )).await
                            .unwrap()
                        ).unwrap();
                        *tiles = next_tiles
                        .into_iter()
                        .map(|s| decode(s).unwrap())
                        .collect::<Vec<_>>();
                        [*h, *v] = door_holder.there.to_owned();
                        println!("exit coords: {},{}", &h, &v);
                        **doors_rc.borrow_mut() = from_str(
                            &read_to_string(&format!(
                                "domains/{}/maps/{}/doors.json",
                                &domain, &current_map
                            )).await
                            .unwrap(),
                        )
                        .unwrap();
                    }
                } else {
                    /* carry on, move the thing */
                    [*h, *v] = [potential_h, potential_v];
                }
            } else {
                /* solid tile detected */
                for attr in attrs {
                    match attr {
                        Push => {
                            let [box_next_h, box_next_v] = next_hv(
                                direction,
                                [&potential_h, &potential_v],
                                [&tiles.len(), &tiles[0].len()],
                            );
                            println!("{:?}", [&box_next_h, &box_next_v]);
                            let attrs = &item_db
                                .get(&String::from(format!("{}", &tiles[box_next_h][box_next_v])))
                                .unwrap()
                                .attributes;
                            println!("{}: {:?}", &tiles[box_next_h][box_next_v], &attrs);
                            if attrs.contains(&NoPassThrough) |
                //attrs.contains(&Read) |
                attrs.contains(&OpenDoor)
                            {
                                /* box effectively hits a NoPassThrough */
                            } else {
                                /* otherwise, scoot the box forward 1 */
                                tiles[box_next_h][box_next_v] =
                                    tiles[potential_h][potential_v].clone();
                                let mut default_under = 1;
                                for door_hv in doors_rc.borrow_mut().iter_mut() {
                                    if [potential_h, potential_v] == door_hv.here {
                                        default_under = 5;
                                    }
                                }
                                for note_hv in notes.iter() {
                                    if [potential_h, potential_v] == note_hv.pos {
                                        default_under = 8;
                                    }
                                }

                                tiles[potential_h][potential_v] = default_under;
                            }

                            let tiles = tiles.iter().map(|s| encode(s)).collect::<Vec<_>>();
                            write(
                                &format!("domains/{}/maps/{}/tiles.json", &domain, &current_map),
                                to_string_pretty(&tiles).unwrap(),
                            ).await
                            .unwrap();
                        }
                        _ => {
                            println!("wall @ {},{}", &potential_h, &potential_v)
                        }
                    }
                }
            }
        }
        self.update_cam().await;
    }

    /// peeks at a given tile without interacting
    pub async fn pk(&mut self, direction: char) -> String {
        let Self {
            info: ShvftInfo { h_v, .. },
            map: ShvftMap { tiles, .. },
            ..
        } = self;

        let [h, v] = h_v;
        let [potential_h, potential_v]: Pos2D;
        [potential_h, potential_v] = next_hv(direction, [h, v], [&tiles.len(), &tiles[0].len()]);
        let peek_result: String = match direction {
            'n' | 'N' | 'k' | 'K' | 's' | 'S' | 'j' | 'J' | 'e' | 'E' | 'l' | 'L' | 'w' | 'W'
            | 'h' | 'H' | '.' => {
                format!("{}", tiles[(potential_h)][potential_v])
            }
            _ => String::from("```fix\nERR::ERROR(DIRECTION_NOT_VALID)```"),
        };
        peek_result
    }
    /// takes from the inventory of an adjacent tile
    pub async fn tk(&mut self, direction: &char, index: usize) -> String {
        let Self {
            domain,
            info:
                ShvftInfo {
                    h_v,
                    db,
                    inventory,
                    current_map,
                    ..
                },
            map: ShvftMap {
                tiles, containers, ..
            },
            ..
        } = self;

        //
        // set up next hv
        let [h, v] = h_v;
        let hv_max = [&tiles.len(), &tiles[0].len()];
        let [next_h, next_v] = next_hv(*direction, [h, v], hv_max);

        //
        // take the thing
        use TurtAttr::*;
        let mut take_result = String::from("");
        let mut taken_item: u8;
        let mut take_t_index: Option<usize>;
        take_t_index = ck_inv_empty(inventory);
        println!("storing at index {:?}", &take_t_index);
        if !(db
            .get(&format!("{}", &tiles[next_h][next_v]))
            .unwrap()
            .attributes
            .contains(&StoreItems))
        {
            /* not a takeable tile */
            take_result = String::from(format!(
                "`{}`,`{}`: ERR::ERROR(NO_CONTAINER)",
                &next_h, &next_v
            ));
        } else {
            println!("container found");
            for container in containers.iter_mut() {
                //println!("{} {} {:?}",container.pos[0],container.pos[1],[next_h, next_v]);
                if ([container.pos[0], container.pos[1]]) == ([next_h, next_v]) {
                    /* correct container */
                    // check & set inventory item on turtle
                    println!("{container:?}, taking index {index}");
                    if take_t_index == None {
                        // empty
                        take_result = String::from(format!(
                            "`{}`,`{}`: ERR::ERROR(NO_TAKEABLE_ITEM)",
                            &next_h, &next_v
                        ));
                    } else {
                        inventory[take_t_index.unwrap()] = container.inventory[index];
                        take_result = format!("took item: {}", container.inventory[index]);
                        container.inventory[index] = 0;
                        write(
                            &format!("domains/{}/info/inv.json", &domain),
                            &to_string(inventory).unwrap(),
                        ).await
                        .unwrap();
                    }
                } else {
                    // wrong container
                    /*take_result = String::from(format!(
                        "`{}`,`{}`: ERR::ERROR(INVALID_CONTAINER)",
                        &next_h, &next_v
                    ));*/
                }
            }
        }
        write(
            &format!("domains/{}/maps/{}/containers.json", &domain, &current_map),
            &(to_string(&containers).unwrap()),
        ).await
        .unwrap();
        take_result
    }

    pub async fn swap_with_floor(&mut self, direction: char, inv_index: usize) -> Result<String> {
        let target_tile: [usize; 2] = next_hv(
            direction,
            [&self.info.h_v[0], &self.info.h_v[1]],
            [&self.map.tiles.len(), &self.map.tiles[0].len()],
        );
        let inventory_item_id = &mut self.info.inventory[inv_index];
        let inventory_item_attrs = get_attrs(&self.info.db, *inventory_item_id)?;
        let floor_item_id = &mut self.map.tiles[target_tile[0]][target_tile[1]];
        let floor_item_attrs = get_attrs(&self.info.db, *floor_item_id)?;

        anyhow::ensure!(
            inventory_item_attrs.contains(&TurtAttr::Drop),
            "NOT_DROPPABLE_ITEM"
        );
        anyhow::ensure!(
            floor_item_attrs.contains(&TurtAttr::Take),
            "NOT_TAKEABLE_ITEM"
        );

        core::mem::swap(inventory_item_id, floor_item_id);

        write(
            PathBuf::from_iter(["domains", &self.domain, "info", "inv.json"]),
            to_string_pretty(&self.info.inventory)?,
        ).await?;
        write(
            PathBuf::from_iter([
                "domains",
                &self.domain,
                "maps",
                &self.info.current_map,
                "tiles.json",
            ]),
            to_string_pretty(
                &self
                    .map
                    .tiles
                    .iter()
                    .map(|s| encode(s))
                    .collect::<Vec<String>>(),
            )?,
        ).await?;

        Ok(format!(""))
    }
    ///
    /// perform a container swap and return the resulting swap info
    pub async fn swap_with_container(
        &mut self,
        direction: char,
        inv_index: usize,
        container_index: usize,
    ) -> Result<String> {
        let inventory_item_id = &mut self.info.inventory[inv_index];
        let inventory_item_attrs = get_attrs(&self.info.db, *inventory_item_id)?;
        let container_ref = get_container_mut(&self.map.containers, &self.info.h_v)?;
        let container_item_id = &mut container_ref.inventory[container_index];
        let container_item_attrs = get_attrs(&self.info.db, *container_item_id)?;

        anyhow::ensure!(
            inventory_item_attrs.contains(&TurtAttr::Drop),
            "NOT_DROPPABLE_ITEM"
        );
        anyhow::ensure!(
            container_item_attrs.contains(&TurtAttr::Take),
            "NOT_TAKEABLE_ITEM"
        );

        write(
            PathBuf::from_iter(["domains", &self.domain, "info", "inv.json"]),
            to_string_pretty(&self.info.inventory)?,
        ).await?;
        write(
            PathBuf::from_iter([
                "domains",
                &self.domain,
                "maps",
                &self.info.current_map,
                "containers.json",
            ]),
            to_string_pretty(&self.map.containers)?,
        ).await?;

        Ok(format!(""))
    }

    pub fn cs(&mut self, direction: char, index: usize) -> String {
        let Self {
            domain,
            map: ShvftMap { tiles, .. },
            ..
        } = self;
        String::from("")
    }
}
///
/// returns a mutable reference to a `ShvftContainer` via a set of hv coords
fn get_container_mut<'a>(
    containers: &'a Vec<ShvftContainer>,
    pos: &'a Pos2D,
) -> Result<&'a mut ShvftContainer> {
    todo!()
}
fn get_attrs(db: &HashMap<String, DbItem>, item_id: u8) -> Result<&[TurtAttr]> {
    Ok(&db
        .get(&format!("{}", item_id))
        .context("NOT_VALID_ITEM")?
        .attributes)
}
/// creates a new instance of a turtle from default values at domain
pub async fn from_domain(domain: String) -> ShvftTurtle {
    let current_map = read_to_string(format!("domains/{}/info/current_map.txt", &domain)).await.unwrap();
    let tiles: Vec<String> = from_str(
        &read_to_string(&format!(
            "domains/{}/maps/{}/tiles.json",
            domain, &current_map
        )).await
        .unwrap(),
    )
    .unwrap();
    let tiles = tiles
        .into_iter()
        .map(|s| decode(s).unwrap())
        .collect::<Vec<_>>();
    ShvftTurtle {
        domain: domain.clone(),
        info: ShvftInfo {
            current_map: current_map.clone(),
            nametag: String::from("turtle"),
            db: from_str(&read_to_string("domains/global/item_db.json").await.unwrap()).unwrap(),
            h_v: from_str(&read_to_string(format!("domains/{}/info/pos.json", &domain)).await.unwrap())
                .unwrap(),
            inventory: from_str(
                &read_to_string(format!("domains/{}/info/inv.json", &domain)).await.unwrap(),
            )
            .unwrap(),
            rail: from_str(&read_to_string(format!("domains/{}/info/rail.json", &domain)).await.unwrap())
                .unwrap(),
        },
        map: ShvftMap {
            tiles,
            doors: from_str(
                &read_to_string(&format!(
                    "domains/{}/maps/{}/doors.json",
                    &domain, &current_map
                )).await
                .unwrap(),
            )
            .unwrap(),
            notes: from_str(
                &read_to_string(&format!(
                    "domains/{}/maps/{}/notes.json",
                    &domain, &current_map
                )).await
                .unwrap(),
            )
            .unwrap(),
            containers: from_str(
                &read_to_string(&format!(
                    "domains/{}/maps/{}/containers.json",
                    &domain, &current_map
                )).await
                .unwrap(),
            )
            .unwrap(),
        },
    }
}
/// sets the potential horizontal and vertical value given a direction and tileset dimensions
pub fn next_hv(dir: char, c_pos: [&Pos; 2], max_hv: [&Pos; 2]) -> Pos2D {
    let [h, v] = c_pos;
    let [t_h, t_v] = max_hv;
    let potential_h: Pos;
    let potential_v: Pos;
    match dir {
        'n' | 'N' | 'k' | 'K' => {
            potential_h = *h;
            potential_v = if *v > 0 { *v - 1 } else { *v };
        }
        's' | 'S' | 'j' | 'J' => {
            potential_h = *h;
            potential_v = if *v < *t_v - 1 { *v + 1 } else { *v };
        }
        'e' | 'E' | 'l' | 'L' => {
            potential_h = if *h < *t_h - 1 { *h + 1 } else { *h };
            potential_v = *v;
        }
        'w' | 'W' | 'h' | 'H' => {
            potential_h = if *h > 0 { *h - 1 } else { *h };
            potential_v = *v;
        }
        '.' => {
            potential_h = *h;
            potential_v = *v;
        }
        _ => {
            potential_h = *h;
            potential_v = *v;
        }
    }
    [potential_h, potential_v]
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