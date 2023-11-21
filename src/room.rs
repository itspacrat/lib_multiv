use crate::*;
use image::{
    Rgb,
    imageops::{resize, FilterType},
    ImageBuffer, Pixel, RgbImage,
};
/// the width of the returned image for a rendered set of [MvRoom] tiles
///
/// todo: make this a part of a config 
/// file so it updates on every ran command
pub const MAP_RENDER_WIDTH: u32 = 500;
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
    pub keys: [Vec<String>; 2],
    //pub id: String, // todo remove, redundant
    pub width: usize,
    pub notes: HashMap<Pos,String>,
    pub tiles: Vec<u8>,
}
/*
!       BEGIN ROOM IMPL
*/
//
impl MvRoom {
    pub fn new(default_key: String) -> Self {
        Self {
            keys: [vec![], vec![default_key.clone()]],
            //id: default_key.clone(),
            width: 7,
            notes: HashMap::new(),
            tiles: vec![
                // default map, 7x7
                // empty room with solid walls
                2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2,
                2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
            ],
            // define width for next_pos()
            
        }
    }
    pub async fn from_existing(default_key: String) -> MvRoom {
        let room: MvRoom = from_str(
            &read_to_string(format!("rooms/{}/data.json", default_key.clone()))
                .await
                .unwrap(),
        )
        .unwrap();
        room
    }
    pub fn height(&self) -> Pos {
        (self.tiles.len() / self.width) as Pos
    }
    /// returns the path of an image generated based on the current player's map
    pub async fn render_map(&self, player: String, position: Pos, database: DB) -> String {
        let p = player;
        //
        let Self {tiles,width,..} = self;
        //
        let mut out_img = RgbImage::new(*width as u32, self.height() as u32);
        //
        for x in 0..*width {
            for y in 0..self.height() {
                
                let rgb = database.get(&format!("{}",tiles[(y*(width))+x])).unwrap().rgb;
                out_img.put_pixel(
                    x as u32, y as u32,
                    *Rgb::from_slice(&rgb)
                );
                // Overlay player pos on map, currently light cyan
                if (y*width)+x == position {
                    out_img.put_pixel(
                        x as u32, y as u32,
                        *Rgb::from_slice(&[0,200,150])
                    );
                }
            }
        }
        out_img = resize(&out_img,MAP_RENDER_WIDTH,(MAP_RENDER_WIDTH/&out_img.width())*out_img.height(),FilterType::Nearest);
        let out_path = format!("players/{}/current_map.jpg",&p);
        out_img.save(&out_path).unwrap();
        out_path
    }
}
