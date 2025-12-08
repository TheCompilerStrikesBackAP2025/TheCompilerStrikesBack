use common_game::components::asteroid::Asteroid;
use common_game::components::planet::Planet;
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToExplorer, OrchestratorToPlanet,
};
use crossbeam_channel::bounded;
use planet::planet::*;
use std::thread;

fn setup_planet_for_tests() -> Planet {
    let (_tx_orch, rx_planet) = bounded(10);
    let (tx_planet, _rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);

    create_planet(rx_planet, tx_planet, rx_explorer, 1)
}

#[test]
fn test_create_planet_success() {
    // Planet creation
    let planet = setup_planet_for_tests();

    // Basic checks
    assert_eq!(planet.id(), planet.id());
    let gen_rules = planet.generator().all_available_recipes();
    assert!(gen_rules.contains(&common_game::components::resource::BasicResourceType::Silicon));
}

#[test]
fn test_planet_run_threaded() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start a planet AI message
    tx_orch.send(OrchestratorToPlanet::StartPlanetAI).unwrap();

    // Receive planet response
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StartPlanetAIResult {
                planet_id,
            } => {
                assert_eq!(planet_id, pln_id);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Kill the planet
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    handle.join().unwrap(); // attendi che il thread termini
}

#[test]
fn test_planet_orchestrator_msg() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start planet AI
    tx_orch.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
    // Receive planet response
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StartPlanetAIResult {
                planet_id,
            } => {
                assert_eq!(planet_id, pln_id);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Send sunray
    tx_orch
        .send(OrchestratorToPlanet::Sunray(Sunray::default()))
        .unwrap();
    // Receive planet response
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::SunrayAck { planet_id } => {
                assert_eq!(planet_id, pln_id);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Send internal state
    tx_orch
        .send(OrchestratorToPlanet::InternalStateRequest)
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::InternalStateResponse {
                planet_id,
                planet_state,
            } => {
                assert_eq!(planet_id, pln_id);
                assert_eq!(planet_state.has_rocket, false);
                assert_eq!(planet_state.charged_cells_count, 1);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Send asteroid
    tx_orch
        .send(OrchestratorToPlanet::Asteroid(Asteroid::default()))
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::AsteroidAck {
                planet_id,
                rocket,
            } => {
                assert_eq!(planet_id, pln_id);
                match rocket {
                    Some(_) => assert!(true),
                    None => panic!("No rocket returned from asteroid message"),
                }
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Send stop planet AI message
    tx_orch.send(OrchestratorToPlanet::StopPlanetAI).unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StopPlanetAIResult {
                planet_id,
            } => {
                assert_eq!(planet_id, pln_id);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Kill the planet
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    handle.join().unwrap(); // attendi che il thread termini
}

#[test]
fn test_planet_destroy_planet() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start planet AI
    tx_orch.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StartPlanetAIResult {
                planet_id,
            } => {
                assert_eq!(planet_id, pln_id);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Send asteroid
    tx_orch
        .send(OrchestratorToPlanet::Asteroid(Asteroid::default()))
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::AsteroidAck {
                planet_id,
                rocket,
            } => {
                assert_eq!(planet_id, pln_id);
                match rocket {
                    Some(_) => panic!("Rocket returned from asteroid when none was expected"),
                    None => {
                        tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
                        if let Ok(msg) = rx_orch.recv() {
                            match msg {
                                common_game::protocols::messages::PlanetToOrchestrator::KillPlanetResult { planet_id } => {
                                    assert_eq!(planet_id, pln_id);
                                }
                                _ => panic!("Unattended message"),
                            }
                        } else {
                            panic!("No responses");
                        }
                    }
                }
            }
            _ => panic!("Unattended message"), // Da guardare domattina
        }
    } else {
        panic!("No responses");
    }

    handle.join().unwrap(); // attendi che il thread termini
}
