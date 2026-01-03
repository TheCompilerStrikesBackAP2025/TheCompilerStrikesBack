/*
PLANET CONFIGURATION:
    - TheCompilerStrikesBack type: C
    - base resource: Silicon
    - complex resource: Robot, Diamond, AI partner
 */

use crate::planet_ai::AI;
use common_game::components::planet::{Planet, PlanetType};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use crossbeam_channel::{Receiver, Sender};

pub fn create_planet(
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
    planet_id: u32,
) -> Planet {
    let id = planet_id;
    let ai = AI::new(id);
    let gen_rules = vec![BasicResourceType::Silicon];
    let comb_rules = vec![
        ComplexResourceType::Robot,
        ComplexResourceType::AIPartner,
        ComplexResourceType::Diamond,
    ];

    // Construct the TheCompilerStrikesBack and return it
    let planet_creation_result = Planet::new(
        id,
        PlanetType::C,
        Box::new(ai),
        gen_rules,
        comb_rules,
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    );

    planet_creation_result.unwrap()
}
