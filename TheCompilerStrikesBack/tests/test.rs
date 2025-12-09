use common_game::components::asteroid::Asteroid;
use common_game::components::planet::Planet;
use common_game::components::resource::ComplexResourceType::{AIPartner, Diamond, Robot};
use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::components::sunray::Sunray;
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};
use crossbeam_channel::bounded;
use TheCompilerStrikesBack::planet::*;
use std::collections::HashSet;
use std::sync::Once;
use std::thread;


pub fn init_logger() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .is_test(true)
            .filter_level(log::LevelFilter::Info) // ensure INFO logs appear
            .try_init()
            .ok();
    });
}


fn setup_planet_for_tests() -> Planet {
    let (_tx_orch, rx_planet) = bounded(10);
    let (tx_planet, _rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);

    create_planet(rx_planet, tx_planet, rx_explorer, 1)
}

/// test for basic TheCompilerStrikesBack creation
#[test]
fn test_create_planet_success() {
    // Planet creation
    let planet = setup_planet_for_tests();

    // Basic checks
    assert_eq!(planet.id(), 1);
    let gen_rules = planet.generator().all_available_recipes();
    assert!(gen_rules.contains(&common_game::components::resource::BasicResourceType::Silicon));
}

/// test for orchestrator-like TheCompilerStrikesBack creation + basic start and kill messages
#[test]
fn test_planet_run_threaded() {
    init_logger();

    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Starts the TheCompilerStrikesBack AI
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

    // Kill the TheCompilerStrikesBack
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    match rx_orch.recv() {
        Ok(PlanetToOrchestrator::KillPlanetResult { planet_id }) => {
            assert_eq!(planet_id, 1, "Planet sent KillPlanetResult with wrong ID");
        }
        Ok(other) => {
            panic!("Test failed: expected KillPlanetResult");
        }
        Err(e) => {
            panic!(
                "Test failed: TheCompilerStrikesBack did not respond before exiting: {:?}",
                e
            );
        }
    }
    handle.join().unwrap();
}

//test for basic orchestrator interactions
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

    // Send start TheCompilerStrikesBack AI
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

    // Send sunray
    tx_orch
        .send(OrchestratorToPlanet::Sunray(Sunray::default()))
        .unwrap();
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
                assert_eq!(planet_state.has_rocket, true);          //planet gives priority to rocket construction
                assert_eq!(planet_state.charged_cells_count, 0);
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Kill the TheCompilerStrikesBack
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    match rx_orch.recv() {
        Ok(PlanetToOrchestrator::KillPlanetResult { planet_id }) => {
            assert_eq!(
                planet_id, pln_id,
                "Planet sent KillPlanetResult with wrong ID"
            );
        }
        Ok(other) => {
            panic!("Test failed: expected KillPlanetResult");
        }
        Err(e) => {
            panic!(
                "Test failed: TheCompilerStrikesBack did not respond before exiting: {:?}",
                e
            );
        }
    }
    handle.join().unwrap();
}

//test for TheCompilerStrikesBack destruction based on asteroidAck
#[test]
fn test_planet_destroyed_asteroid() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start TheCompilerStrikesBack AI
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
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    handle.join().unwrap();
}

//test for TheCompilerStrikesBack destruction based on asteroidAck
#[test]
fn test_planet_survives_asteroid() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (_tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start TheCompilerStrikesBack AI
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

    // Send sunray
    tx_orch
        .send(OrchestratorToPlanet::Sunray(Sunray::default()))
        .unwrap();
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


    // Send asteroid
    tx_orch
        .send(OrchestratorToPlanet::Asteroid(Asteroid::default()))
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            PlanetToOrchestrator::AsteroidAck {
                planet_id,
                rocket,
            } => {
                assert_eq!(planet_id, pln_id);
                match rocket {
                    Some(_) => {} //expected behavior, planet is safe
                    None => {
                        panic!("Rocket was not sent!")
                    }
                }
            }
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    // Kill the TheCompilerStrikesBack
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    match rx_orch.recv() {
        Ok(PlanetToOrchestrator::KillPlanetResult { planet_id }) => {
            assert_eq!(
                planet_id, pln_id,
                "Planet sent KillPlanetResult with wrong ID"
            );
        }
        Ok(other) => {
            panic!("Test failed: expected KillPlanetResult");
        }
        Err(e) => {
            panic!(
                "Test failed: TheCompilerStrikesBack did not respond before exiting: {:?}",
                e
            );
        }
    }
    handle.join().unwrap();
}


//test for explorer messages
#[test]
fn test_planet_explorer() {
    let (tx_orch, rx_planet) = bounded(10);
    let (tx_planet, rx_orch) = bounded(10);
    let (tx_explorer, rx_explorer) = bounded(10);
    let pln_id = 1;

    let mut planet = create_planet(rx_planet, tx_planet, rx_explorer, pln_id);

    // We call the run method in a new thread
    let handle = thread::spawn(move || {
        planet.run().unwrap();
    });

    // Send start TheCompilerStrikesBack AI
    tx_orch.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StartPlanetAIResult {
                planet_id,
            } => {}
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    //having only one energy cell, we send a Sunray for each resource creation:
    tx_orch
        .send(OrchestratorToPlanet::Sunray(Sunray::default()))
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::SunrayAck { planet_id } => {}
            _ => panic!("Unattended message"),
        }
    }

    tx_orch
        .send(OrchestratorToPlanet::Sunray(Sunray::default()))
        .unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::SunrayAck { planet_id } => {}
            _ => panic!("Unattended message"),
        }
    }


    // explorer arrives on TheCompilerStrikesBack:
    let explorer_id = 101;
    let (expl_tx_local, expl_rx_local) = bounded::<PlanetToExplorer>(10);

    tx_orch
        .send(OrchestratorToPlanet::IncomingExplorerRequest {
            explorer_id,
            new_mpsc_sender: expl_tx_local,
        })
        .unwrap();
    // 6. Verify Ack from Planet
    match rx_orch.recv() {
        Ok(PlanetToOrchestrator::IncomingExplorerResponse { planet_id, res }) => {
            assert_eq!(planet_id, pln_id);
            assert!(res.is_ok());
        }
        _ => panic!("Expected IncomingExplorerResponse"),
    }

    //EXPLORER ASKS FOR AVAILABLE ENERGY CELLS
    tx_explorer
        .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id })
        .unwrap();

    match expl_rx_local.recv() {
        Ok(PlanetToExplorer::AvailableEnergyCellResponse { available_cells }) => {
            assert_eq!(available_cells, 1);
        }
        _ => panic!("Expected AvailableEnergyCellResponse"),
    }

    //basic generation - WRONG
    tx_explorer
        .send(ExplorerToPlanet::GenerateResourceRequest {
            explorer_id,
            resource: BasicResourceType::Oxygen,
        })
        .unwrap();

    match expl_rx_local.recv() {
        Ok(PlanetToExplorer::GenerateResourceResponse { resource }) => match resource {
            None => {}
            Some(res) => {
                panic!("wrong resource got generated!")
            }
        },
        _ => panic!("Unattended message"),
    }

    //basic generation - CORRECT
    tx_explorer
        .send(ExplorerToPlanet::GenerateResourceRequest {
            explorer_id,
            resource: BasicResourceType::Silicon,
        })
        .unwrap();

    match expl_rx_local.recv() {
        Ok(PlanetToExplorer::GenerateResourceResponse { resource }) => match resource {
            None => {
                panic!("Expected resource in return, but nothing was returned")
            }
            Some(res) => {
                assert_eq!(res.get_type(), BasicResourceType::Silicon)
            }
        },
        _ => panic!("Unattended message"),
    }

    // Send stop TheCompilerStrikesBack AI message
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

    //explorer asks for something, but TheCompilerStrikesBack AI is stopped
    tx_explorer
        .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id })
        .unwrap();

    match expl_rx_local.recv() {
        Ok(PlanetToExplorer::Stopped) => {} //correct behavior
        _ => panic!("Unattended message"),
    }

    // Restart TheCompilerStrikesBack AI
    tx_orch.send(OrchestratorToPlanet::StartPlanetAI).unwrap();
    if let Ok(msg) = rx_orch.recv() {
        match msg {
            common_game::protocols::messages::PlanetToOrchestrator::StartPlanetAIResult {
                planet_id,
            } => {}
            _ => panic!("Unattended message"),
        }
    } else {
        panic!("No responses");
    }

    tx_explorer
        .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id })
        .unwrap();

    match expl_rx_local.recv() {
        Ok(PlanetToExplorer::SupportedCombinationResponse { combination_list }) => {
            let mut h: HashSet<ComplexResourceType> = HashSet::new();
            h.insert(Robot);
            h.insert(Diamond);
            h.insert(AIPartner);
            assert_eq!(combination_list, h)
        } //correct behavior
        _ => panic!("Unattended message"),
    }

    // 11. Cleanup
    tx_orch.send(OrchestratorToPlanet::KillPlanet).unwrap();
    match rx_orch.recv() {
        Ok(PlanetToOrchestrator::KillPlanetResult { planet_id }) => {
            assert_eq!(
                planet_id, pln_id,
                "Planet sent KillPlanetResult with wrong ID"
            );
        }
        Ok(other) => {
            panic!("Test failed: expected KillPlanetResult");
        }
        Err(e) => {
            panic!(
                "Test failed: TheCompilerStrikesBack did not respond before exiting: {:?}",
                e
            );
        }
    }
    handle.join().unwrap();
}
