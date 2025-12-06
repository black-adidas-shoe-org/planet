// use log::{}
use common_game::components::planet::PlanetAI;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Payload};

fn main() {
    let mut payload = Payload::new();
    payload.insert(1.to_string(), "Omati si è pisciato addosso".to_string());

    env_logger::init();
    LogEvent::new(
        ActorType::Planet,
        123u64,
        ActorType::Orchestrator,
        String::from("1"),
        EventType::MessageOrchestratorToPlanet,
        Channel::Error,
        Payload::from([("AI disabled".to_string(), "AI field `is_on` is false".to_string())])
    ).emit();
}
