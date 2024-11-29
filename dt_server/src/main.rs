mod hotel;

use crate::hotel::*;
use alkahest::{deserialize, serialize};
use axum::extract::ws::Message as AWsMessage;
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::extract::State as AState;
use axum::http::HeaderMap;
use axum::{Error as AError, Router};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message as TWsMessage;

struct WsSocket {
    stream: SplitStream<WebSocket>,
    sink: SplitSink<WebSocket, AWsMessage>,
}

impl From<WebSocket> for WsSocket {
    fn from(val: WebSocket) -> Self {
        let (sink, stream) = val.split();
        WsSocket { stream, sink }
    }
}
use std::{
    collections::HashMap,
    time::Instant,
};

use dt_lib::{
    battle::{army::*, battlefield::*, troop::Troop},
    items::item::*,
    locale::{parse_locale, Locale},
    map::{
        event::{execute_event, Event, Execute},
        map::*,
        object::ObjectInfo,
        tile::*,
    },
    network::net::*,
    parse::{parse_items, parse_objects, parse_settings, parse_story, parse_units},
    time::time::Data as TimeData,
    units::{
        unit::{ActionResult, Unit, UnitPos},
        unitstats::ModifyUnitStats,
    },
};
type MutRc<T> = Arc<Mutex<T>>;
enum ServerService {
    Matchmaking,
}
#[derive(Clone)]
pub struct State {
    pub hotel: MutRc<Hotel>,
}
struct RoomInstance {
	pub armies: Vec<Army>,
	pub battle: BattleInfo,
}
impl RoomInstance {
	pub fn new(units: &Vec<Unit>) -> Self {
		let mut armies = Vec::new();
		armies.push(gen_army(0, units));
		armies.push(gen_army(1, units));
		let battle = BattleInfo::new(&mut armies, 0, 1);
		Self {
			armies,
			battle
		}
	}
}
fn gen_army(army_num: usize, units: &Vec<Unit>) -> Army {
    let mut army = Army::new(
        vec![],
        ArmyStats {
            gold: 0,
            mana: 0,
            army_name: String::new(),
        },
        vec![],
        (0, 0),
        true,
        dt_lib::battle::control::Control::PC,
    );
    for _ in 0..10 {
        army.add_troop(Troop::new(units.get(4).unwrap().clone()).into())
            .ok();
    }
    army
}
static UNITS: LazyLock<Vec<Unit>> = LazyLock::new(|| parse_units(None).unwrap().0);
fn setup() -> State {
    let settings = parse_settings();
    let _ = parse_items(None, &settings.locale);
	State {
		hotel: Arc::new(Mutex::new(Hotel::new())),
	}
}
fn process_move(message: AWsMessage, instance: &mut RoomInstance, army: usize) ->  Result<AWsMessage, AError> {
	let RoomInstance { armies, battle } = instance;
	if battle.active_unit.is_some_and(|x| x.0 != army) {
		return Err(AError::new("fuck off!"));
	}
	let buf = message.into_data();
	let Ok((target_army, target_index)) = deserialize::<(usize, usize), (usize, usize)>(&*buf) else {
		return Err(AError::new("bad data."));
	};
	handle_action(Action::Cell(target_army, target_index), battle, armies);
	let mut buf = vec![];
	serialize::<(Vec<Army>, BattleInfo), (Vec<Army>, BattleInfo)>((armies.clone(), battle.clone()), &mut buf);
	Ok(AWsMessage::Binary(buf))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a hotel with a room
	let state = setup();
    let hotel = state.hotel;
	
    // Start the server
    let app = Router::new()
        .route("/ws", axum::routing::any(ws_handler))
        .with_state(hotel);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    let server = tokio::spawn(async {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    server.await.unwrap();
    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    AState(hotel): AState<MutRc<Hotel>>,
) -> impl axum::response::IntoResponse {
    let room_code = headers
        .get("room-code")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    ws.on_upgrade(move |socket| handle_socket(socket, room_code, (hotel, RoomInstance::new(&UNITS))))
}

async fn handle_socket(socket: WebSocket, room_code: String, (hotel, mut instance): (Arc<Mutex<Hotel>>, RoomInstance)) {
    let room = {
        let mut hotel = hotel.lock().await;
        match hotel.put_socket(&room_code, socket) {
            Ok(None) => return,
            Ok(Some(room)) => room,
            Err((mut socket, e)) => {
                socket.send(AWsMessage::Text(e.to_string())).await.unwrap();
                socket.close().await.unwrap();
                return;
            }
        }
    };

	fn stream_with_id<T, E>(
        stream: impl StreamExt<Item = Result<T, E>>,
        id: usize,
    ) -> impl StreamExt<Item = (T, usize)> {
        stream
            .filter_map(|x| futures::future::ready(x.ok()))
            .map(move |x| (x, id))
    }

    futures::stream::select(
        stream_with_id(room.0.stream, 0),
        stream_with_id(room.1.stream, 1),
    )
		.map(|(x, id)| process_move(x, &mut instance, id))
		.forward(room.1.sink.fanout(room.0.sink))
		.await
		.unwrap();
}
