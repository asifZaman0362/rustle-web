use rusqlite::{Connection, Result, Statement};
use std::path::Path;
use serde::Serialize;

#[derive(Serialize)]
pub struct Stats {
    played: i32,
    won: i32,
    streak_current: i32,
    streak_max: i32,
    frequency: [i32; 6],
}

pub fn open_database<P>(url: P) -> Connection
where
    P: AsRef<Path>,
{
    let mut conn = Connection::open(url).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS stats 
            (id TEXT PRIMARY KEY, played INTEGER, won INTEGER, streak_curr INTEGER, streak_max INTEGER, 
            ones INTEGER, twos INTEGER, threes INTEGER, fours INTEGER, fives INTEGERS, sixes INTEGER);",
        []).unwrap();
    conn
}

pub fn get_stats(id: &str, conn: &Connection) -> Result<Stats> {
    let mut statement = conn.prepare("SELECT played, won, streak_curr, streak_max, ones, twos, threes, fours, fives, sixes FROM stats WHERE id = ?")?;
    match statement.query_row([id], |row| {
        let played: i32 = row.get(0)?;
        let won: i32 = row.get(1)?;
        let streak_current = row.get(2)?;
        let streak_max = row.get(3)?;
        let mut frequency = [0i32; 6];
        frequency[0] = row.get(4)?;
        frequency[1] = row.get(5)?;
        frequency[2] = row.get(6)?;
        frequency[3] = row.get(7)?;
        frequency[4] = row.get(8)?;
        frequency[5] = row.get(9)?;
        Ok(Stats {
            played,
            won,
            streak_current,
            streak_max,
            frequency,
        })
    }) {
        Ok(stats) => Ok(stats),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            let mut statement = conn.prepare("INSERT INTO stats VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")?;
            statement.execute((id, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0))?;
            Ok(Stats { played: 0, won: 0, streak_current: 0, streak_max: 0, frequency: [0i32; 6] })
        },
        Err(err) => Err(err)
    }
}

pub fn put_stats(id: &str, conn: &Connection, stats: &Stats) -> Result<usize> {
    let mut statement = conn.prepare("UPDATE stats SET played = ?, won = ?, streak_curr = ?, streak_max = ?, ones = ?, twos = ?, threes = ?, fours = ?, fives = ?, sixes = ? WHERE id = ?")?;
    statement.execute((
        stats.played,
        stats.won,
        stats.streak_current,
        stats.streak_max,
        stats.frequency[0],
        stats.frequency[1],
        stats.frequency[2],
        stats.frequency[3],
        stats.frequency[4],
        stats.frequency[5],
        id,
    ))
}

pub fn add_win(id: &str, conn: &mut Connection, attempts: usize) -> Result<usize> {
    let mut stats = get_stats(id, conn)?;
    stats.played += 1;
    stats.won += 1;
    stats.streak_current += 1;
    if stats.streak_current > stats.streak_max {
        stats.streak_max = stats.streak_current;
    }
    if attempts < 1 || attempts > 6 {
        return Err(rusqlite::Error::IntegralValueOutOfRange(5, attempts as i64));
    } else {
        stats.frequency[(attempts - 1) as usize] += 1;
    }
    return put_stats(id, conn, &stats);
}

pub fn add_loss(id: &str, conn: &mut Connection) -> Result<usize> {
    if let Ok(mut stats) = get_stats(id, conn) {
        stats.played += 1;
        stats.streak_current = 0;
        return put_stats(id, conn, &stats);
    }
    return Err(rusqlite::Error::QueryReturnedNoRows);
}
