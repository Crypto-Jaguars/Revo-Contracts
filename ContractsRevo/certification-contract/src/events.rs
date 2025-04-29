use soroban_sdk::{BytesN, Env, Symbol, Vec};
use crate::datatypes::{CertificationEvent, DataKey};

pub struct EventManager;

impl EventManager {
    pub fn record_event(
        env: &Env,
        certification_id: &BytesN<32>,
        event_type: Symbol,
        data: Vec<Symbol>
    ) {
        // Create the event
        let event = CertificationEvent {
            certification_id: certification_id.clone(),
            event_type,
            timestamp: env.ledger().timestamp(),
            data,
        };
        
        // Get existing events or create a new vector
        let mut events = env.storage().persistent()
            .get::<_, Vec<CertificationEvent>>(&DataKey::CertificationEvents(certification_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        // Add the new event and store
        events.push_back(event);
        env.storage().persistent().set(&DataKey::CertificationEvents(certification_id.clone()), &events);
    }
} 