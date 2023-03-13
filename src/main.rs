use actix_web::{
    self,
    cookie::{self, time::Duration, Cookie},
    get, post,
    middleware::Logger,
    web::{Data, Path},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Error
};
use serde_json;
use rand::Rng;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::Mutex;
use uuid::Uuid;
use futures_util::future::FutureExt;
use rusqlite::Connection;

mod game;
mod database;

use game::Game;
use database::{ get_stats, add_win, add_loss, open_database, Stats };

type GamesList = Mutex<HashMap<String, Game>>;
type Dictionary = Vec<String>;

struct AppState {
    dict: Dictionary,
    games: GamesList,
    connection: Mutex<Connection>,
}

#[get("/")]
async fn index(req: HttpRequest, data: Data<AppState>) -> impl Responder {
    use std::io::Read;
    let mut contents = String::new();
    std::fs::File::open("public/index.html").unwrap().read_to_string(&mut contents).unwrap();
    if let Some(id) = req.cookie("id") {
        if let Some(_) = data.games.lock().unwrap().get(id.value()) {
            return HttpResponse::Ok().body(contents);
        }
        let rnd_idx = rand::thread_rng().gen_range(0..data.dict.len());
        let word = &data.dict[rnd_idx];
        let game = Game::new(word);
        data.games
            .lock()
            .unwrap()
            .insert(id.value().to_owned(), game);
        return HttpResponse::Ok().body(contents);
    } else {
        let id = Uuid::new_v4().to_string();
        let mut cookie = Cookie::new("id", id.clone());
        cookie.set_max_age(Duration::days(1));
        let rnd_idx = rand::thread_rng().gen_range(0..data.dict.len());
        let word = &data.dict[rnd_idx];
        let game = Game::new(word);
        data.games.lock().unwrap().insert(id, game);
        let mut response = HttpResponse::Ok().body(contents);
        response.add_cookie(&cookie).unwrap();
        response
    }
}

#[get("/state")]
async fn get_state(req: HttpRequest, data: Data<AppState>) -> impl Responder {
    if let Some(cookie) = req.cookie("id") {
        if let Some(game) = data.games.lock().unwrap().get_mut(cookie.value()) {
            let guesses = &game.guesses;
            return HttpResponse::Ok().json(Ok::<&Vec<Vec<game::Entry>>, &'static str>(guesses));
        } else {
            return HttpResponse::Ok().json(Err::<&Vec<Vec<game::Entry>>, &'static str>("not in game"));
        }
    } else {
        return HttpResponse::BadRequest().json(Err::<&Vec<Vec<game::Entry>>, &'static str>("unregistered"));
    }
}

#[get("/stats")]
async fn stats(req: HttpRequest, data: Data<AppState>) -> impl Responder {
    if let Some(cookie) = req.cookie("id") {
        let connection = data.connection.lock().unwrap();
        match get_stats(cookie.value(), &connection) {
            Ok(stats) => HttpResponse::Ok().json(Ok::<Stats, String>(stats)),
            Err(err) => HttpResponse::Ok().json(Err::<Stats, String>(err.to_string()))
        }
    } else {
        HttpResponse::BadRequest().json(Err::<Stats, String>("unregistered".to_string()))
    }
}

#[get("/restart")]
async fn restart(req: HttpRequest, data: Data<AppState>) -> impl Responder {
    if let Some(cookie) = req.cookie("id") {
        if let Some(game) = data.games.lock().unwrap().get_mut(cookie.value()) {
            let rnd_idx = rand::thread_rng().gen_range(0..data.dict.len());
            let word = &data.dict[rnd_idx];
            game.restart(word);
            return HttpResponse::Ok().body("");
        } else {
            HttpResponse::Ok().body("")
        }
    } else {
        HttpResponse::BadRequest().body("not in game!")
    }
}

#[get("/submit/{word}")]
async fn submit(req: HttpRequest, path: Path<String>, data: Data<AppState>) -> impl Responder {
    if let Some(cookie) = req.cookie("id") {
        if let Some(game) = data.games.lock().unwrap().get_mut(cookie.value()) {
            let word = path.into_inner();
            if let Err(_) = data.dict.binary_search(&word.to_uppercase()) {
                return HttpResponse::Ok().json(Err::<(), game::GuessError>(game::GuessError::NonexistentWordError));
            }
            let result = game.submit_guess(&word);
            if let Ok(res) = &result {
                match res {
                    game::GuessResult::Won(..) => {
                        add_win(cookie.value(), &mut data.connection.lock().unwrap(), game.guesses.len()).unwrap();
                    },
                    game::GuessResult::MaxGuess(_) => {
                        add_loss(cookie.value(), &mut data.connection.lock().unwrap()).unwrap();
                    },
                    _ => {}
                }
            }
            println!("{}", game.answer);
            return HttpResponse::Ok().json(result);
        } else {
            HttpResponse::BadRequest().body("not in game!")
        }
    } else {
        HttpResponse::BadRequest().body("not in game!")
    }
}

#[actix::main]
async fn main() -> std::io::Result<()> {
    let mut dict = vec![];
    let dict_file = std::fs::File::open("dict.txt")?;
    let reader = BufReader::new(&dict_file);
    for line in reader.lines() {
        dict.push(line?);
    }
    let games = Mutex::new(HashMap::new());
    let connection = Mutex::new(open_database("database.db"));
    let data = Data::new(AppState { dict, games, connection });
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .service(index)
            .service(submit)
            .service(get_state)
            .service(stats)
            .service(restart)
    })
    .bind(("localhost", 8000))
    .unwrap()
    .run()
    .await
}
