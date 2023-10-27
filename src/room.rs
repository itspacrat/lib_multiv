use crate::*;

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
            tiles: vec![
                // default map, 7x7
                // empty room with solid walls
                2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2,
                2, 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
            ],
            // define width for next_pos()
            width: 7,
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
}
