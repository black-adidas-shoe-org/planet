mod planet;

// use log::{}
use common_game::components::planet::PlanetAI;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};

fn main() {
    let mut payload = Payload::new();
    payload.insert(1.to_string(), "Omati si è pisciato addosso".to_string());

    env_logger::init();
    LogEvent::new(
        ActorType::Orchestrator,
        45 as u64,
        ActorType::Planet,
        String::from("903"),
        EventType::MessageOrchestratorToPlanet,
        Channel::Error,
        payload
    ).emit();


}
