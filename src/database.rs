use rusqlite::{Connection, OptionalExtension, Result};
use crate::types;
use std::collections::HashMap;
use poise::serenity_prelude as serenity;
use std::time::SystemTime;
use chrono::{DateTime, Utc};

const TEAM_SWAPPING_PATH: &str = "databases/teamSwaps.db";

pub fn get_teams() -> Result<Vec<types::TeamMembers>> {
    const GET_TEAMS: &str = "
    SELECT members.member_id, members.name, teams.team_id, teams.team_name, teams.team_image_url
    FROM members
    INNER JOIN teams ON members.team = teams.team_id;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut team_query = conn.prepare(GET_TEAMS)?;
    let team_iter = team_query.query_map([], |row| {
        Ok((
            types::Team {
                team_id: row.get(2)?,
                team_image_url: row.get(4)?,
                team_name: row.get(3)?,
            },
            types::Member {
                member_id: row.get(0)?,
                member_name: row.get(1)?,
            },
        ))
    })?;
    let mut teams: HashMap<u64, (types::Team, Vec<types::Member>)> = HashMap::new();
    for m in team_iter {
        match m {
            Ok((team, member)) => {
                teams.entry(team.team_id).or_insert_with(|| {
                    (team.clone(), Vec::new()) // Clone the team to insert into the HashMap
                }).1.push(member);
            },
            Err(_) => (),
        }
    }
    let team_members_list: Vec<types::TeamMembers> = teams
    .into_iter()
    .map(|(_, (team, members))| types::TeamMembers { team, members })
    .collect();
    Ok(team_members_list)
}


/// Creates a team with 1-3 members and a name in the database
pub fn create_team(members: &Vec<serenity::User>, team_name: &String) -> Result<usize> {
    const CREATE_TEAM: &str = "
    INSERT INTO teams (team_name)
    VALUES(?1);
    ";
    const UPDATE_MEMBERS: &str = "
    UPDATE members
    SET team = ?1
    WHERE member_id = ?2;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_TEAM, rusqlite::params![team_name])?;
    let team_id = conn.last_insert_rowid() as u64;
    for member in members {
        let id = member.id.get();
        conn.execute(UPDATE_MEMBERS, rusqlite::params![team_id, id])?;
    };
    Ok(res)
}

pub fn create_member(member: serenity::User) -> Result<usize> {
    const CREATE_MEMBER: &str = "
    INSERT INTO members VALUES (?1, ?2, NULL);
    ";
    let id = member.id.get();
    let name = member.name;
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_MEMBER, rusqlite::params![id, name])?;
    Ok(res)
}

pub fn create_anime(anime_id: &u64, anime_name: &String, submitter_id: u64) -> Result<usize> {
    const CREATE_ANIME: &str = "
    INSERT INTO anime VALUES (?1, ?2, ?3);
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_ANIME, rusqlite::params![anime_id, anime_name, submitter_id])?;
    Ok(res)
}

pub fn create_claimed_anime(anime_id: u64, team_id: u64, user_id :u64) -> Result<usize> {
    const CREATE_CLAIMED_ANIME: &str = "
    INSERT INTO claimed_anime (anime_id, team_id, claimed_by, claimed_on)
    VALUES (?1, ?2, ?3, ?4);
    ";
    let present_time: SystemTime = SystemTime::now();
    let present_time: DateTime<Utc> = present_time.into();
    let present_time: String = present_time.to_rfc3339();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    println!("{} {} {} {}", anime_id, team_id, user_id, present_time);
    let res = conn.execute(CREATE_CLAIMED_ANIME, rusqlite::params![anime_id, team_id, user_id, present_time])?;
    Ok(res)
}

pub fn delete_team_by_team_id(team_id: u64) -> Result<usize> {
    const DELETE_TEAM_FOR_MEMBERS: &str = "
    UPDATE MEMBERS
    SET team = NULL
    WHERE team = ?1;
    ";
    const DELETE_CLAIMS: &str = "
    DELETE FROM claimed_anime WHERE team_id = ?1;
    ";
    const DELETE_TEAM: &str = "
    DELETE FROM teams WHERE team_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    conn.execute(DELETE_TEAM_FOR_MEMBERS, rusqlite::params![team_id])?;
    conn.execute(DELETE_CLAIMS, rusqlite::params![team_id])?;
    conn.execute(DELETE_TEAM, rusqlite::params![team_id])?;
    Ok(0)
}

pub fn check_if_user_in_team(user_id :u64) -> Result<Option<u64>> {
    const CHECK_QUERY: &str = "
    SELECT team FROM members WHERE member_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: Option<u64>= conn.query_row(CHECK_QUERY,
        rusqlite::params![user_id],
         |row| row.get(0))?;
    match res {
        Some(team_id) => Ok(Some(team_id)),
        None => Ok(None),
}
}

pub fn check_if_team_exists(team_name: &String) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT COUNT(*) FROM teams WHERE team_name = ?1;
    "; 
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut query = conn.prepare(CHECK_QUERY)?;
    let count: u64 = query.query_row(rusqlite::params![team_name], |row| row.get(0))?;
    if count == 0 {
        return Ok(false)
    } else {
        return Ok(true);
    }
}

pub fn check_if_user_exists(user_id: u64) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT member_id FROM members WHERE member_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let user_exists: Option<u64> = conn.query_row(
        CHECK_QUERY,
        rusqlite::params![user_id],
        |row| row.get(0),
    ).optional()?;
    Ok(user_exists.is_some())
}

pub fn check_if_anime_exists(anime_id: u64) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT anime_id FROM anime WHERE anime_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let anime_exists: Option<u64> = conn.query_row(
        CHECK_QUERY,
        rusqlite::params![anime_id],
        |row| row.get(0),
    ).optional()?;
    Ok(anime_exists.is_some())
}

pub fn check_if_anime_is_claimed(anime_name: &String) -> Result<bool> {
    const CHECK_QUERY: &str = "
        SELECT EXISTS(
        SELECT 1 
        FROM anime a
        JOIN claimed_anime ca ON a.anime_id = ca.anime_id
        WHERE a.name = ?1
    )";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(CHECK_QUERY)?;
    let is_claimed: bool = stmt.query_row([anime_name], |row| row.get(0))?;
    Ok(is_claimed)
}

pub fn count_submitted_anime(user_id: u64) -> Result<u64> {
    const COUNT_QUERY: &str = "
    SELECT COUNT(*) AS anime_count FROM anime WHERE submitter = ?1; 
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let anime_count: u64 = conn.query_row(COUNT_QUERY, rusqlite::params![user_id], |row| row.get(0))?;
    Ok(anime_count)
}

pub fn get_submitted_anime(user_id: u64) -> Result<Vec<String>> {
    const ANIME_NAME_QUERY: &str = "
    SELECT name FROM anime WHERE submitter = ?1;
    ";
    let mut names: Vec<String> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut names_query = conn.prepare(ANIME_NAME_QUERY)?;
    let names_iter = names_query.query_map(rusqlite::params![user_id],
    |row| {
        Ok(
            row.get::<_, String>(0)?
        )
    })?;
    for name in names_iter {
        match name {
            Ok(n,) => names.push(n),
            Err(_) => (),
        }
    }
    Ok(names)
}

pub fn get_all_anime() -> Result<Vec<types::SubmittedAnime>> {
    const GET_ANIME_QUERY: &str = "
    SELECT 
        a.anime_id,
        a.name AS anime_name,
        m.name AS submitter_name,
        t.team_name AS claimed_team_name,
        ca.claimed_on
    FROM 
        anime a
    JOIN 
        members m ON a.submitter = m.member_id
    LEFT JOIN 
        claimed_anime ca ON a.anime_id = ca.anime_id
    LEFT JOIN 
        teams t ON ca.team_id = t.team_id;
    ";
    let mut anime: Vec<types::SubmittedAnime> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut anime_query = conn.prepare(GET_ANIME_QUERY)?;
    let anime_iter = anime_query.query_map((),
    |row| {
        Ok(types::SubmittedAnime {
            anime_id: row.get(0)?,
            anime_name: row.get(1)?,
            submitter_name: row.get(2)?,
            claimed_by_team: row.get::<_, Option<String>>(3)?,
            claimed_on: row.get::<_, Option<String>>(4)?,
        })
    })?;
    for a in anime_iter {
        match a {
            Ok(valid_anime) => anime.push(valid_anime),
            Err(_) => (),
        }
    }
    Ok(anime)
}

pub fn get_anime_submitter(anime_name: &String) -> Result<u64> {
    const SUBMITTER_QUERY: &str = "
    SELECT submitter FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: u64 = conn.query_row(SUBMITTER_QUERY, rusqlite::params![anime_name], |row| row.get(0))?;
    Ok(res)
}

pub fn get_member_with_team(user_id :u64) -> Result<(types::Member, u64)> {
    const MEMBER_QUERY: &str = "
    SELECT * FROM members WHERE member_id = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: (types::Member, u64) = conn.query_row(MEMBER_QUERY, rusqlite::params![user_id], 
    |row| {
        Ok((types::Member {
            member_id: row.get(0)?,
            member_name: row.get(1)?,
        },
            row.get(2)?,
        ))
    })?;
    Ok(res)
}

pub fn get_anime_id_by_name(anime_name: &String) -> Result<Option<(u64, u64)>> {
    const ID_QUERY: &str = "
    SELECT anime_id, submitter FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.query_row(ID_QUERY, rusqlite::params![anime_name], 
        |row| {
            let anime_id: u64 = row.get(0)?;
            let submitter_id: u64 = row.get(1)?;
            Ok((anime_id, submitter_id))
        });
    match res {
        Ok((anime_id, submitter_id)) => Ok(Some((anime_id, submitter_id))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_unclaimed_anime_names() -> Result<Vec<String>> {
    const UNCLAIMED_QUERY: &str = "
    SELECT a.name
    FROM anime a
    LEFT JOIN claimed_anime ca ON a.anime_id = ca.anime_id
    WHERE ca.anime_id IS NULL;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(UNCLAIMED_QUERY)?;
    let anime_names = stmt.query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect::<Vec<String>>();
    Ok(anime_names)
}

pub fn get_claimed_anime_by_user(user_id: u64) -> Result<Vec<String>> {
    const NAMES_QUERY: &str = "
    SELECT anime.name
    FROM claimed_anime
    JOIN anime ON claimed_anime.anime_id = anime.anime_id
    WHERE claimed_anime.team_id = (
    SELECT team
    FROM members
    WHERE member_id = ?
);

    ";
    let conn: Connection =  Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(NAMES_QUERY)?;
    let anime_names = stmt.query_map(rusqlite::params![user_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect::<Vec<String>>();
    Ok(anime_names)
}

pub fn get_teammembers_id_by_team_id(team_id: u64) -> Result<Vec<u64>> {
    const TEAM_QUERY: &str = "
    SELECT member_id
    FROM members
    WHERE team = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(TEAM_QUERY)?;
    let ids: Vec<u64> = stmt.query_map(rusqlite::params![team_id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect::<Vec<u64>>();
    Ok(ids)
}

pub fn get_lonely_users() -> Result<Vec<(u64, String)>> {
    const TEAMLESS_QUERY: &str = "
    SELECT m.member_id, m.name
    FROM members m
    WHERE m.team IS NULL;
    ";
    let mut users: Vec<(u64, String)> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut query = conn.prepare(TEAMLESS_QUERY)?;
    let query_iter = query.query_map((),
    |row| {
        Ok((
            row.get::<_, u64>(0)?,
            row.get::<_, String>(1)?,
        ))
    })?;
    for user in query_iter {
        match user {
            Ok(u) => users.push(u),
            Err(_) => (),
        }
    }
    Ok(users)
}

pub fn get_lonely_eligible_users() -> Result<Vec<String>> {
    const TEAMLESS_QUERY: &str = "
    SELECT 
        m.name
    FROM 
        members m
    LEFT JOIN 
        anime a 
    ON 
        m.member_id = a.submitter
    WHERE 
        m.team IS NULL
    GROUP BY 
        m.member_id, m.name
    HAVING 
        COUNT(a.anime_id) >= 7;
    ";
    let conn: Connection =  Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(TEAMLESS_QUERY)?;
    let users = stmt.query_map((), |row| row.get(0))?
    .filter_map(Result::ok)
    .collect::<Vec<String>>();
    Ok(users)
}

pub fn get_team_and_time_claimed_anime(anime_id: u64) -> Result<(u64, String)> {
    const TEAM_TIME_QUERY: &str = "
    SELECT team_id, claimed_on
    FROM claimed_anime
    WHERE anime_id = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: (u64, String) = conn.query_row(TEAM_TIME_QUERY, rusqlite::params![anime_id], |row| {
        Ok((
            row.get::<_, u64>(0)?,
            row.get::<_, String>(1)?,
        ))
    })?;
    Ok(res)
}

pub fn count_submissions_by_user() -> Result<Vec<(u64, String, u64)>> {
    const COUNT_SUBMISSION_QUERY: &str = "
    SELECT m.member_id, m.name, COUNT(a.anime_id) AS anime_count
    FROM members m
    LEFT JOIN anime a ON m.member_id = a.submitter
    GROUP BY m.member_id, m.name;
    ";
    let mut submission_counts: Vec<(u64, String, u64)> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut query = conn.prepare(COUNT_SUBMISSION_QUERY)?;
    let query_iter = query.query_map((), 
    |row| {
        Ok((
            row.get::<_, u64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u64>(2)?,
        ))
    })?;
    for submission_count in query_iter {
        match submission_count {
            Ok(c) => submission_counts.push(c),
            Err(_) => (),
        }
    }
    Ok(submission_counts)
}

pub fn delete_anime(anime_name: &String) -> Result<usize> {
    const DELETE_QUERY: &str = "
    DELETE FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(DELETE_QUERY, rusqlite::params![anime_name])?;
    Ok(res)
}

pub fn delete_claim(anime_id: u64) -> Result<usize> {
    const DELETE_QUERY: &str = "
    DELETE FROM claimed_anime WHERE anime_id = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(DELETE_QUERY, rusqlite::params![anime_id])?;
    Ok(res)
}

pub fn delete_user(user_id: &u64) -> Result<usize> {
    let anime_deletion: &str = "
    DELETE FROM anime WHERE submitter = ?1;
    ";
    let user_deletion: &str = "
    DELETE FROM members WHERE member_id = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    conn.execute(anime_deletion, rusqlite::params![user_id])?;
    let res = conn.execute(user_deletion, rusqlite::params![user_id]);
    res
}

pub fn update_team_name(team_id: u64, new_name: String) -> Result<usize> {
    const UPDATE_QUERY: &str = "
    UPDATE teams
    SET team_name = ?1
    WHERE team_id = ?2;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(UPDATE_QUERY, rusqlite::params![new_name, team_id])?;
    Ok(res)
}

pub fn update_team_image(team_id: u64, new_image: String) -> Result<usize> {
    const UPDATE_QUERY: &str = "
    UPDATE teams
    SET team_image_url = ?1
    WHERE team_id = ?2;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(UPDATE_QUERY, rusqlite::params![new_image, team_id])?;
    Ok(res)
}