/*
PLANET CONFIGURATION:
    - planet type: C
    - base resource: Silicon
    - complex resource: Robot
 */

use crate::planet_ai::AI;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::messages;
use crossbeam_channel::{Receiver, Sender};
use std::sync::mpsc;

pub fn create_planet(
    rx_orchestrator: Receiver<messages::OrchestratorToPlanet>,
    tx_orchestrator: Sender<messages::PlanetToOrchestrator>,
    rx_explorer: Receiver<messages::ExplorerToPlanet>,
) -> Planet {
    let id = 1;
    let ai = AI::default();
    let gen_rules = vec![BasicResourceType::Silicon];
    let comb_rules = vec![ComplexResourceType::Robot];

    // Construct the planet and return it
    let planet_creation_result = Planet::new(
        id,
        PlanetType::C,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );

    // maybe doesn't need to panic
    planet_creation_result.unwrap_or_else(|err_string| panic!("{}", err_string))
}
