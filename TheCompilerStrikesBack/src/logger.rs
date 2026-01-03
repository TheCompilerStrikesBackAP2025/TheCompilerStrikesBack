use crate::planet_ai::AI;
use common_game::logging::ActorType::*;
use common_game::logging::Channel::*;
use common_game::logging::{EventType, LogEvent, Participant, Payload};

/*
   Our planet will only log activities that regard its internal state
   = those actions that won't be notified to the orchestrator through an ack message

   For instance, actions like start, stop (planet AI), kill planet and outgoing/incoming explorer request
   can be logged via the orchestrator based on the following returning messages coming from each planet:
   -
   - StopPlanetAIResult
   - KillPlanetResult
   - OutgoingExplorerResponse
   - IncomingExplorerResponse
*/

impl AI {
    pub fn log_charge_cell(&self, detail: String) {
        let mut payload = Payload::new();
        payload.insert("Energy cell".to_string(), detail);

        LogEvent::new(
            Some(Participant::new(Orchestrator, 0_u32)),
            Some(self.log_part.clone()),
            EventType::InternalPlanetAction,
            Debug,
            payload,
        )
        .emit();
    }

    pub fn log_build_rocket(&self) {
        let mut payload = Payload::new();
        payload.insert("Rocket".to_string(), "Built".to_string());

        LogEvent::new(
            Some(Participant::new(Orchestrator, 0_u32)),
            Some(self.log_part.clone()),
            EventType::InternalPlanetAction,
            Debug,
            payload,
        )
        .emit();
    }

}
