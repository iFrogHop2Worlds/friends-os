use std::sync::{Arc, Mutex};
pub use rocket::response::stream::{EventStream, Event};
pub use rocket::{State, Shutdown};
use rocket::serde::json::Json;
use crate::chat::{ChatState, Message};
pub use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
pub use rocket::tokio::select;

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events")]
pub async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

/// Receive a message from a form submission and broadcast it to any receivers.
/// Update App State
#[post("/message", data = "<json>")]
pub fn post(json: Json<Message>, queue: &State<Sender<Message>>, state: &State<Arc<Mutex<ChatState>>>){
    let mut chat_state = state.lock().unwrap();

    if let Some(mut room) = chat_state.rooms.iter_mut().find(|room| room.name == json.room) {
        // Add the message to the room's messages vector
        room.messages.push(json.clone().into_inner());
    } else {
        // Handle room not found scenario (e.g., error message)
        println!("Room does not exist");
    }
    // if json.name === chat_state.rooms.*.name then append messages
    println!("json message in send: {:?}", chat_state.rooms);
    // A send 'fails' if there are no active subscribers. That's okay.
    let _res = queue.send(json.into_inner());
}

#[get("/rooms/<room_name>/messages")]
pub fn get_room_messages(room_name: &str, state: &State<Arc<Mutex<ChatState>>>) -> Option<Json<Vec<Message>>> {
    let chat_state = state.inner().lock().unwrap();
    println!("Requested room name: {}", room_name);
    let messages = chat_state.get_room_messages(room_name);
    println!("Messages: {:?}", &messages);
    messages.map(|messages| Json(messages.to_vec()))
}