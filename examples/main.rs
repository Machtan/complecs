#[macro_use]
extern crate complecs;

// ============ Components =============


components_and_store! {
    /// Stores all the components!
    pub struct Components {
        /// The name of an entity.
        pub name: CName (String),
        /// The age of an entity.
        pub age: CAge (u32),
    }
}


// ============= Processes ================

process! {
    pub mod print_info {
        /// Prints info about an entity.
        pub fn PPrintInfo::run(ref name[n]: &CName, ref age[a]: &CAge,) { 
            println!("{} is {} year(s) old", name, age); 
        }
    }
}

process! {
    pub mod double_age {
        /// Doubles the age of an entity.
        pub fn PDoubleAge::run(mut age[a]: &mut CAge,) {
            *age *= 2;
        }
    }
}

process! {
    pub mod print_with_last_name {
        /// Prints the name of the entity with an added last name.
        pub fn PPrintWithLastName::run(ref name[n]: &CName, ext last_name: &str,) {
            println!("Name: {} {}", name, last_name);
        }
    }
}

process_store! {
    pub struct Processes {
        print_info: PPrintInfo,
        double_age: PDoubleAge,
        pwln: PPrintWithLastName,
    }
}

// ============= Entities ================

entity! {
    pub mod player {
        /// The avatar that the player controls in the game.
        pub struct EPlayer {
            name: CName, 
            age: CAge,
        }
        impl {
            PPrintInfo,
            PDoubleAge,
            PPrintWithLastName,
        }
    }
}

entity_store! {
    pub struct Entities {
        player: EPlayer,
    }
}

// ====== SIM data ======

#[derive(Debug, Default)]
pub struct Sim {
    components: Components,
    entities: Entities,
    processes: Processes,
}

impl Sim {
    pub fn new() -> Sim {
        Sim::default()
    }
    
    pub fn update(&mut self) {
        PPrintInfo::run(self);
        PDoubleAge::run(self);
        PPrintWithLastName::run(self, "Erroinen");
    }
}

contains_processes! {
    Sim.processes: Processes
}

contains_components! {
    Sim.components: Components
}

contains_entities! {
    Sim.entities: Entities
}

fn main() {
    println!("Hello world!");

    let mut sim = Sim::new();
    
    let player = EPlayer::new_data(String::from("Jakob"), 22);
    player.add_to(&mut sim);
    
    let another = EPlayer::new_data(String::from("test"), 9001);
    another.add_to(&mut sim);
    
    //println!("\n==== BEFORE WRITE ====\n");
    //println!("print_info: {:?}", sim.processes.print_info);
    //println!("players:    {:?}", sim.entities.players);
    
    //sim.processes.print_info.write();
    
    //println!("\n==== AFTER WRITE ====\n");
    
    //println!("Sim: {:?}", sim);
    //println!("print_info: {:?}", sim.processes.print_info);
    //println!("players:    {:?}", sim.entities.players);
    
    sim.update();
    sim.update();
}
