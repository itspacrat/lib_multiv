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
    /// the feature list the player currently has access to;
    /// helps limit the abilities of those solving puzzles :^)
    pub features: Vec<String>,
}

/// compare a list of strings to another list of strings. unsure if this is necessary to have.
pub fn host_has_key(guest_keys: Vec<String>, host_keys: Vec<String>) -> bool {
    let mut out: bool = false;
    for k in guest_keys {
        if host_keys.contains(&k) {
            out = true;
        }
    }
    out
}

impl MvPlayer {
    /// should be called after checking whether or not a player exists for a given player key.
    ///
    /// instantiates a new player with a given player key + default values
    pub fn new(default_key: String) -> Self {
        Self {
            // start keyring with user's key
            keys: vec![default_key.clone()], // initialize w player key
            room_id: default_key.clone(),    // default player room
            position: 24,                    // center of the default map
            inventory: vec![0, 0, 0, 0, 0, 0, 0], // empty
            rail: vec![0, 0, 0],             // empty
            features: vec![String::from("mv")], // default
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
    pub async fn mv(&mut self, mut room: &mut MvRoom, db: &DB, directions: Vec<char>) {
        let mut mv_out: Pos;
        //
        // march over every direction and process n times for n directions
        for d in directions.iter() {
            let attrs_next = get_attrs(db, room.tiles[next_pos(*d, &self.position, room)]).unwrap();
            let next = next_pos(*d, &self.position, room);
            //
            // !            attribute checks begin here
            //
            // check if solid
            //println!("attrs next pos: {:?}",attrs_next);
            if attrs_next.contains(&MvTileAttribute::NoPassThrough) {
                //
                // check if it has push
                if attrs_next.contains(&MvTileAttribute::Push) {
                    let expensive_room_clone = room.clone().to_owned();
                    let next_2 = next_pos(*d, &next, &expensive_room_clone);

                    if get_attrs(db, expensive_room_clone.tiles[next_2])
                        .unwrap()
                        .contains(&MvTileAttribute::NoPassThrough)
                    {
                        mv_out = self.position;
                    } else {
                        // swap push tiles
                        let dummy: u8;
                        dummy = room.tiles[next];
                        room.tiles[next] = room.tiles[next_2];
                        room.tiles[next_2] = dummy;

                        mv_out = next_pos(*d, &(self.position), room)
                    }
                } else {
                    // ! CHECK DOORS

                    println!("attrs_next\n{:?}", &attrs_next);
                    println!("attrs_here {:?}", get_attrs(db, room.tiles[self.position]));
                    mv_out = self.position;
                }
            } else {
                if get_attrs(db, room.tiles[self.position])
                    .unwrap()
                    .contains(&MvTileAttribute::OpenDoor)
                {
                    println!("this tile has OpenDoor attr");

                    //
                    // check if actually walking through the door
                    if *d == room.doors.get(&format!("{}",self.position)).unwrap().exit_direction {
                        //
                        // preload next room
                        let next_room = MvRoom::from_existing(
                            room.doors.get(&format!("{next}")).unwrap().exit_map.clone(),
                        )
                        .await;
                        // println!("preloaded {}", &next_room.keys[1][0]);
                        println!("{} eq; saving current room", d);
                        //
                        // save current room before swapping
                        let _ = write(
                            format!("rooms/{}/data.json", room.keys[1][0]),
                            to_string_pretty(room).unwrap(),
                        )
                        .await
                        .unwrap();
                        //
                        // set player pos for new map
                        mv_out = room.doors.get(&format!("{}",self.position)).unwrap().there;
                        //
                        // hacky stitch together/move room vars
                        room.tiles = next_room.tiles.to_owned();
                        room.keys = next_room.keys.to_owned();
                        room.doors = next_room.doors.to_owned();
                        room.notes = next_room.notes.to_owned();
                        room.width = next_room.width.to_owned();
                        println!("{:?}", room);
                        //
                        // set new map for player
                        self.room_id = ((&room).keys).clone()[1].to_owned()[0].to_owned();
                        //
                        // re-save room after swap
                        let _ = write(
                            format!("rooms/{}/data.json", room.keys[1][0]),
                            to_string_pretty(room).unwrap(),
                        )
                        .await
                        .unwrap();
                        //
                        // save player key
                        let _ = write(
                            format!("players/{}/data.json", self.keys[0]),
                            to_string_pretty(&self).unwrap(),).await
                        .unwrap();

                        
                    } else {
                        mv_out = self.position;
                    }
                } else {
                    mv_out = next_pos(*d, &(self.position), &room);
                }
                
            }

            self.position = mv_out;
        }
        let _ = write(
            format!("rooms/{}/data.json", room.keys[1][0]),
            to_string_pretty(room).unwrap(),
        )
        .await
        .unwrap();
    }
    /// with interact set to `false`, checks the tile [DB] id in a given direction
    ///
    /// with interact set to `true`, gets detailed information on the tile in a
    /// given direction, and triggers an interaction where applicable
    pub fn ck(&self, db: &DB, interact: bool, room: &mut MvRoom, direction: char) -> String {
        let output: String;
        let next = next_pos(direction, &self.position, room);
        let target = room.tiles[next];
        match interact {
            false => {
                // this replaces the old separate peek command

                output = format!(
                    "peeked `{}` @ `{}`:\n{}",
                    &self.position, &direction, target
                );
            }
            true => {
                // this is the same as the old check command
                let item_type = db.get(&format!("{}", (target))).unwrap().clone();

                // check for specific interractions
                match target {
                    8 => {
                        // read the thing
                        output = format!(
                            "{} ({}):\n{}\n\n{}",
                            target,
                            next,
                            &item_type.description,
                            room.notes.get(&next).unwrap()
                        );
                    }
                    4 => {
                        // open door
                        output = format!(
                            "{}:\n{} \n```opened unlocked door.```",
                            target, &item_type.description
                        );

                        room.tiles[next] = 5;
                    }
                    5 => {
                        // close door
                        output = format!(
                            "{}:\n{} \n```closed unlocked door.```",
                            target, &item_type.description
                        );

                        room.tiles[next] = 4;
                    }
                    _ => {
                        output = format!("{}:\n{}", target, &item_type.description);
                    }
                }
            }
        }
        format!("{}", output)
    }
}
