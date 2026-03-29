use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::DomainError,
    event::Event,
    ports::{EventRepository, RsvpRepository},
    rsvp::{Rsvp, RsvpStatus},
};

pub async fn create_event(
    repo: &impl EventRepository,
    host_id: Uuid,
    title: String,
    description: Option<String>,
    address: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    start_time: OffsetDateTime,
    max_capacity: Option<i32>,
) -> Result<Event, DomainError> {
    let event = Event {
        id: Uuid::now_v7(),
        host_id,
        title,
        description,
        address,
        latitude,
        longitude,
        start_time,
        max_capacity,
        created_at: OffsetDateTime::now_utc(),
    };
    repo.upsert(&event).await
}

pub async fn get_event(repo: &impl EventRepository, event_id: Uuid) -> Result<Event, DomainError> {
    repo.find(event_id).await
}

pub async fn list_upcoming(
    repo: &impl EventRepository,
    limit: i64,
) -> Result<Vec<Event>, DomainError> {
    repo.find_upcoming(limit).await
}

pub async fn list_hosted(
    repo: &impl EventRepository,
    host_id: Uuid,
) -> Result<Vec<Event>, DomainError> {
    repo.find_by_host(host_id).await
}

pub async fn update_event(
    repo: &impl EventRepository,
    event_id: Uuid,
    host_id: Uuid,
    title: String,
    description: Option<String>,
    address: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    start_time: OffsetDateTime,
    max_capacity: Option<i32>,
) -> Result<Event, DomainError> {
    let event = repo.find(event_id).await?;
    if event.host_id != host_id {
        return Err(DomainError::NotFound);
    }
    let updated = Event {
        title,
        description,
        address,
        latitude,
        longitude,
        start_time,
        max_capacity,
        ..event
    };
    repo.upsert(&updated).await
}

pub async fn delete_event(
    repo: &impl EventRepository,
    event_id: Uuid,
    host_id: Uuid,
) -> Result<(), DomainError> {
    let event = repo.find(event_id).await?;
    if event.host_id != host_id {
        return Err(DomainError::NotFound);
    }
    repo.delete(event_id).await
}

pub async fn set_rsvp(
    event_repo: &impl EventRepository,
    rsvp_repo: &impl RsvpRepository,
    user_id: Uuid,
    event_id: Uuid,
    status: RsvpStatus,
) -> Result<Rsvp, DomainError> {
    // Ensure the event exists
    event_repo.find(event_id).await?;
    let rsvp = Rsvp {
        user_id,
        event_id,
        status,
        created_at: OffsetDateTime::now_utc(),
    };
    rsvp_repo.upsert(&rsvp).await
}

pub async fn remove_rsvp(
    rsvp_repo: &impl RsvpRepository,
    user_id: Uuid,
    event_id: Uuid,
) -> Result<(), DomainError> {
    rsvp_repo.delete(user_id, event_id).await
}

pub async fn list_rsvps(
    rsvp_repo: &impl RsvpRepository,
    event_id: Uuid,
) -> Result<Vec<Rsvp>, DomainError> {
    rsvp_repo.find_by_event(event_id).await
}
