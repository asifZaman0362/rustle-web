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

mod game;
use game::Game;

type GamesList = Mutex<HashMap<String, Game>>;
type Dictionary = Vec<String>;

struct AppState {
    dict: Dictionary,
    games: GamesList,
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

#[get("/submit/{word}")]
async fn submit(req: HttpRequest, path: Path<String>, data: Data<AppState>) -> impl Responder {
    if let Some(cookie) = req.cookie("id") {
        if let Some(game) = data.games.lock().unwrap().get_mut(cookie.value()) {
            let result = game.submit_guess(&path.into_inner());
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
    let data = Data::new(AppState { dict, games });
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .service(index)
            .service(submit)
    })
    .bind(("localhost", 8000))
    .unwrap()
    .run()
    .await
}
