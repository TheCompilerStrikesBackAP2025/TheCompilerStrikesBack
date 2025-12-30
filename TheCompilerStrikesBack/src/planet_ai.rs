use common_game::components::planet::{DummyPlanetState, PlanetAI, PlanetState};
use common_game::components::resource::{
    BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
    Generator, GenericResource,
};
use common_game::components::rocket::Rocket;
use common_game::components::sunray::Sunray;
use common_game::logging::ActorType::Planet;
use common_game::logging::Participant;
use common_game::protocols::planet_explorer::PlanetToExplorer::{
    AvailableEnergyCellResponse, CombineResourceResponse, GenerateResourceResponse,
    SupportedCombinationResponse, SupportedResourceResponse,
};
use common_game::protocols::planet_explorer::{ExplorerToPlanet, PlanetToExplorer};

pub struct AI {
    pub(crate) log_part: Participant,
}

impl AI {
    pub fn new(id: u32) -> Self {
        Self {
            log_part: Participant::new(Planet, id),
        }
    }
}

impl PlanetAI for AI {
    /// Handle a sunray event:
    /// - Charge an energy cell
    /// - If there is no rocket yet, attempt to build one when a full cell is available
    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        sunray: Sunray,
    ) {
        match state.charge_cell(sunray) {
            None => self.log_charge_cell("charged".to_string()),
            Some(_) => self.log_charge_cell("already charged".to_string()),
        }

        if state.to_dummy().has_rocket == false {
            match state.full_cell() {
                None => {}
                Some((_cell, i)) => {
                    let _ = state.build_rocket(i);
                    self.log_build_rocket();
                }
            }
        }
    }

    /// Handle an asteroid event:
    /// - If a rocket already exists, return it
    /// - Otherwise, try to build a rocket if a charged cell is available
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            return Some(state.take_rocket()).unwrap();
        }
        if let Some((_cell, index)) = state.full_cell() {
            let build_rocket_result = state.build_rocket(index);
            self.log_build_rocket();
            return match build_rocket_result {
                Ok(_) => Some(state.take_rocket().unwrap()),
                Err(_) => None,
            };
        }
        None
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
    ) -> DummyPlanetState {
        state.to_dummy()
    }

    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest {
                explorer_id: _explorer_id,
            } => Some(SupportedResourceResponse {
                resource_list: generator.all_available_recipes(),
            }),
            ExplorerToPlanet::SupportedCombinationRequest {
                explorer_id: _explorer_id,
            } => Some(SupportedCombinationResponse {
                combination_list: combinator.all_available_recipes(),
            }),
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: _explorer_id,
                resource,
            } => {
                if let Some((cell, _index)) = state.full_cell() {
                    return match resource {
                        BasicResourceType::Silicon => match generator.make_silicon(cell) {
                            Ok(basic) => Some(GenerateResourceResponse {
                                resource: Some(BasicResource::Silicon(basic)),
                            }),
                            Err(_) => Some(GenerateResourceResponse { resource: None }),
                        },
                        // Attempts to generate unsupported resources fail
                        _ => Some(GenerateResourceResponse { resource: None }),
                    };
                }
                // there isn't any charged cell available
                Some(GenerateResourceResponse { resource: None })
            }
            ExplorerToPlanet::CombineResourceRequest {
                explorer_id: _explorer_id,
                msg,
            } => match msg {
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
                            "There isn't any charged cell".to_string(),
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
                            "There isn't any charged cell".to_string(),
                            GenericResource::ComplexResources(ComplexResource::Robot(robot)),
                            GenericResource::ComplexResources(ComplexResource::Diamond(diamond)),
                        )),
                    })
                }
            },
            ExplorerToPlanet::AvailableEnergyCellRequest {
                explorer_id: _explorer_id,
            } => {
                let available_cells = state.to_dummy().charged_cells_count;
                Some(AvailableEnergyCellResponse {
                    available_cells: available_cells as u32,
                })
            }
        }
    }

    fn on_explorer_arrival(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: u32,
    ) {
    }

    fn on_explorer_departure(
        &mut self,
        _state: &mut PlanetState,
        _generator: &Generator,
        _combinator: &Combinator,
        _explorer_id: u32,
    ) {
    }

    fn on_start(&mut self, _state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
    }

    fn on_stop(&mut self, _state: &PlanetState, _generator: &Generator, _combinator: &Combinator) {
    }
}
