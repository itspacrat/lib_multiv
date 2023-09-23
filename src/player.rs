use crate::*;

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
impl MvPlayer {
    /// should be called after checking whether or not a player exists for a given player key.
    ///
    /// instantiates a new player with a given player key + default values
    pub fn new(default_key: String) -> Self {
        Self {
            // start keyring with user's key
            keys: vec![default_key.clone()],
            room_id: default_key.clone(),
            position: 24,                         // center of the default map
            inventory: vec![0, 0, 0, 0, 0, 0, 0], // empty
            rail: vec![0, 0, 0],                  // empty
        }
    }
    //
    /// loads playerdata from a file matching the current player's key
    pub async fn from_existing(default_key: String) -> MvPlayer {
        let player: MvPlayer = from_str(
            &read_to_string(format!("players/{}/data.json", default_key.clone()))
                .await
                .unwrap(),
        )
        .unwrap();

        player
    }
    /*
!       BEGIN PLAYER IMPL
*/
    //
    //*                         MOVEMENT
    /// handles moving the player in a room
    pub async fn mv(&mut self, room: &mut MvRoom, db: &DB, directions: Vec<char>) {
        let mut mv_out: Pos;
        //
        // march over every direction and process n times for n directions
        for d in directions.iter() {
            let attrs_next = get_attrs(db, room.tiles[next_pos(*d, &self.position, room)]).unwrap();
            //
            // !            attribute checks begin here
            //
            // check if solid
            if attrs_next.contains(&MvTileAttribute::NoPassThrough) {
                //
                // check if it has push
                if attrs_next.contains(&MvTileAttribute::Push) {
                    let next = next_pos(*d, &self.position, room);
                    let mut dummy: u8 = 0_u8;
                    //
                    // swap push tiles
                    let tiles_mut = &mut room.tiles;
                    let mut here = tiles_mut[self.position];
                    let mut there = tiles_mut[next];

                    swap(&mut here, &mut there);

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
            format!("rooms/{}/data.json", self.keys[0]),
            to_string_pretty(room).unwrap(),
        )
        .await
        .unwrap();
    }
}