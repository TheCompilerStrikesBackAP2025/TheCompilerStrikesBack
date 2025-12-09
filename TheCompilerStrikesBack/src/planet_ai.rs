use common_game::components::planet::{PlanetAI, PlanetState};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Carbon, Combinator, ComplexResource, ComplexResourceRequest,
    Generator, GenericResource, Hydrogen, Life, Robot, Silicon,
};
use common_game::components::rocket::Rocket;
use common_game::logging::{ActorType, Channel, EventType, LogEvent};
use common_game::protocols::messages::PlanetToExplorer::{
    AvailableEnergyCellResponse, CombineResourceResponse, GenerateResourceResponse,
    SupportedCombinationResponse, SupportedResourceResponse,
};
use common_game::protocols::messages::PlanetToOrchestrator::{
    AsteroidAck, InternalStateResponse, KillPlanetResult, StartPlanetAIResult, StopPlanetAIResult,
    SunrayAck,
};
use common_game::protocols::messages::{
    ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
};

pub struct AI {}

impl Default for AI {
    fn default() -> Self {
        AI {}
    }
}

impl PlanetAI for AI {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                state.charge_cell(sunray);
                if(state.to_dummy().has_rocket == false) {
                    match state.full_cell() {
                        None => {}
                        Some((_cell, i)) => {
                            // assert!(cell.is_charged());
                            let _ = state.build_rocket(i);
                        }
                    }
                }
                Some(SunrayAck {
                    planet_id: state.id(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest => Some(InternalStateResponse {
                planet_id: state.id(),
                planet_state: state.to_dummy(),
            }),
            _ => None,
        }
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        // Why is the explorer_id sent in this message
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                Some(SupportedResourceResponse {
                    resource_list: generator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                Some(SupportedCombinationResponse {
                    combination_list: combinator.all_available_recipes(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => {
                if let Some((cell, index)) = state.full_cell() {
                    return match resource {
                        BasicResourceType::Hydrogen => match generator.make_hydrogen(cell) {
                            Ok(basic) => Some(GenerateResourceResponse {
                                resource: Some(BasicResource::Hydrogen(basic)),
                            }),
                            Err(_) => Some(GenerateResourceResponse { resource: None }),
                        },
                        BasicResourceType::Carbon => match generator.make_carbon(cell) {
                            Ok(basic) => Some(GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(basic)),
                            }),
                            Err(_) => Some(GenerateResourceResponse { resource: None }),
                        },
                        BasicResourceType::Oxygen => match generator.make_oxygen(cell) {
                            Ok(basic) => Some(GenerateResourceResponse {
                                resource: Some(BasicResource::Oxygen(basic)),
                            }),
                            Err(_) => Some(GenerateResourceResponse { resource: None }),
                        },
                        BasicResourceType::Silicon => match generator.make_silicon(cell) {
                            Ok(basic) => Some(GenerateResourceResponse {
                                resource: Some(BasicResource::Silicon(basic)),
                            }),
                            Err(_) => Some(GenerateResourceResponse { resource: None }),
                        },
                    };
                }
                Some(GenerateResourceResponse { resource: None })
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => match msg {
                ComplexResourceRequest::Robot(silicon, life) => {
                    if let Some((cell, _index)) = state.full_cell() {
                        return match combinator.make_robot(silicon, life, cell) {
                            Ok(complex) => Some(CombineResourceResponse {
                                complex_response: Ok(ComplexResource::Robot(complex)),
                            }),
                            Err((err, silicon, life)) => Some(CombineResourceResponse {
                                complex_response: Err((
                                    err,
                                    GenericResource::BasicResources(BasicResource::Silicon(
                                        silicon,
                                    )),
                                    GenericResource::ComplexResources(ComplexResource::Life(life)),
                                )),
                            }),
                        };
                    }
                    Some(CombineResourceResponse {
                        complex_response: Err((
                            "There isn't any charged cell".to_string(),
                            GenericResource::BasicResources(BasicResource::Silicon(silicon)),
                            GenericResource::ComplexResources(ComplexResource::Life(life)),
                        )),
                    })
                }
                ComplexResourceRequest::Water(hydrogen, oxygen) => Some(CombineResourceResponse {
                    complex_response: Err((
                        "there isn't a recipe for water".to_string(),
                        GenericResource::BasicResources(BasicResource::Hydrogen(hydrogen)),
                        GenericResource::BasicResources(BasicResource::Oxygen(oxygen)),
                    )),
                }),
                ComplexResourceRequest::Diamond(carbon1, carbon2) => {
                    if let Some((cell, _index)) = state.full_cell() {
                        return match combinator.make_diamond(carbon1, carbon2, cell) {
                            Ok(complex) => Some(CombineResourceResponse {
                                complex_response: Ok(ComplexResource::Diamond(complex)),
                            }),
                            Err((err, carbon1, carbon2)) => Some(CombineResourceResponse {
                                complex_response: Err((
                                    err,
                                    GenericResource::BasicResources(BasicResource::Carbon(carbon1)),
                                    GenericResource::BasicResources(BasicResource::Carbon(carbon2)),
                                )),
                            }),
                        };
                    }
                    Some(CombineResourceResponse {
                        complex_response: Err((
                            "there isn't a recipe for diamond".to_string(),
                            GenericResource::BasicResources(BasicResource::Carbon(carbon1)),
                            GenericResource::BasicResources(BasicResource::Carbon(carbon2)),
                        )),
                    })
                }
                ComplexResourceRequest::Life(water, carbon) => Some(CombineResourceResponse {
                    complex_response: Err((
                        "there isn't a recipe for life".to_string(),
                        GenericResource::ComplexResources(ComplexResource::Water(water)),
                        GenericResource::BasicResources(BasicResource::Carbon(carbon)),
                    )),
                }),
                ComplexResourceRequest::Dolphin(water, life) => Some(CombineResourceResponse {
                    complex_response: Err((
                        "there isn't a recipe for dolphin".to_string(),
                        GenericResource::ComplexResources(ComplexResource::Water(water)),
                        GenericResource::ComplexResources(ComplexResource::Life(life)),
                    )),
                }),
                ComplexResourceRequest::AIPartner(robot, diamond) => {
                    if let Some((cell, _index)) = state.full_cell() {
                        return match combinator.make_aipartner(robot, diamond, cell) {
                            Ok(complex) => Some(CombineResourceResponse {
                                complex_response: Ok(ComplexResource::AIPartner(complex)),
                            }),
                            Err((err, robot, diamond)) => Some(CombineResourceResponse {
                                complex_response: Err((
                                    err,
                                    GenericResource::ComplexResources(ComplexResource::Robot(
                                        robot,
                                    )),
                                    GenericResource::ComplexResources(ComplexResource::Diamond(
                                        diamond,
                                    )),
                                )),
                            }),
                        };
                    }
                    Some(CombineResourceResponse {
                        complex_response: Err((
                            "there isn't a recipe for AI partner".to_string(),
                            GenericResource::ComplexResources(ComplexResource::Robot(robot)),
                            GenericResource::ComplexResources(ComplexResource::Diamond(diamond)),
                        )),
                    })
                }
            },
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                let available_cells = state.to_dummy().charged_cells_count;
                Some(AvailableEnergyCellResponse {
                    available_cells: available_cells as u32,
                })
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            return Some(state.take_rocket()).unwrap()
        }
        if let Some((_cell, index)) = state.full_cell() {
            let build_rocket_result = state.build_rocket(index);
            return match build_rocket_result {
                Ok(_) => Some(state.take_rocket().unwrap()),
                Err(_) => None,
            };
        }
        None
    }

    fn start(&mut self, state: &PlanetState) {
        LogEvent::new(ActorType::Planet, state.id(), ActorType::Orchestrator, 0.to_string(), EventType::MessagePlanetToOrchestrator, Channel::Info, Default::default()).emit();
    }

    fn stop(&mut self, state: &PlanetState) {}
}
