use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use domain::{
    event::Event,
    rsvp::{Rsvp, RsvpStatus},
    user::User,
};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::ApiError,
    state::{AppServices, AppState, EventRepoState, RsvpRepoState},
};

pub fn router<S: AppServices>() -> Router<AppState<S>> {
    Router::new()
        .route("/events", get(list_events).post(create_event))
        .route(
            "/events/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/events/{id}/rsvp", post(set_rsvp).delete(remove_rsvp))
        .route("/events/{id}/rsvps", get(list_rsvps))
        .route("/me/events", get(my_events))
}

#[derive(Deserialize)]
struct CreateEventBody {
    title: String,
    description: Option<String>,
    address: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    #[serde(with = "time::serde::rfc3339")]
    start_time: OffsetDateTime,
    max_capacity: Option<i32>,
}

#[derive(Deserialize)]
struct SetRsvpBody {
    status: RsvpStatus,
}

async fn list_events<S: AppServices>(
    _user: User,
    State(event_repo): State<EventRepoState<S>>,
) -> Result<Json<Vec<Event>>, ApiError> {
    let events = domain::event::crud::list_upcoming(event_repo.as_ref(), 50).await?;
    Ok(Json(events))
}

async fn create_event<S: AppServices>(
    user: User,
    State(event_repo): State<EventRepoState<S>>,
    Json(body): Json<CreateEventBody>,
) -> Result<(StatusCode, Json<Event>), ApiError> {
    let event = domain::event::crud::create_event(
        event_repo.as_ref(),
        user.id,
        body.title,
        body.description,
        body.address,
        body.latitude,
        body.longitude,
        body.start_time,
        body.max_capacity,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(event)))
}

async fn get_event<S: AppServices>(
    _user: User,
    State(event_repo): State<EventRepoState<S>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Event>, ApiError> {
    let event = domain::event::crud::get_event(event_repo.as_ref(), event_id).await?;
    Ok(Json(event))
}

async fn update_event<S: AppServices>(
    user: User,
    State(event_repo): State<EventRepoState<S>>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<CreateEventBody>,
) -> Result<Json<Event>, ApiError> {
    let event = domain::event::crud::update_event(
        event_repo.as_ref(),
        event_id,
        user.id,
        body.title,
        body.description,
        body.address,
        body.latitude,
        body.longitude,
        body.start_time,
        body.max_capacity,
    )
    .await?;
    Ok(Json(event))
}

async fn delete_event<S: AppServices>(
    user: User,
    State(event_repo): State<EventRepoState<S>>,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    domain::event::crud::delete_event(event_repo.as_ref(), event_id, user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_rsvp<S: AppServices>(
    user: User,
    State(event_repo): State<EventRepoState<S>>,
    State(rsvp_repo): State<RsvpRepoState<S>>,
    Path(event_id): Path<Uuid>,
    Json(body): Json<SetRsvpBody>,
) -> Result<Json<Rsvp>, ApiError> {
    let rsvp = domain::event::crud::set_rsvp(
        event_repo.as_ref(),
        rsvp_repo.as_ref(),
        user.id,
        event_id,
        body.status,
    )
    .await?;
    Ok(Json(rsvp))
}

async fn remove_rsvp<S: AppServices>(
    user: User,
    State(rsvp_repo): State<RsvpRepoState<S>>,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    domain::event::crud::remove_rsvp(rsvp_repo.as_ref(), user.id, event_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_rsvps<S: AppServices>(
    _user: User,
    State(rsvp_repo): State<RsvpRepoState<S>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Vec<Rsvp>>, ApiError> {
    let rsvps = domain::event::crud::list_rsvps(rsvp_repo.as_ref(), event_id).await?;
    Ok(Json(rsvps))
}

async fn my_events<S: AppServices>(
    user: User,
    State(event_repo): State<EventRepoState<S>>,
) -> Result<Json<Vec<Event>>, ApiError> {
    let events = domain::event::crud::list_hosted(event_repo.as_ref(), user.id).await?;
    Ok(Json(events))
}
