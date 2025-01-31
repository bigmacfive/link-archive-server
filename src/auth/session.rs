use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use once_cell::sync::Lazy;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

static SESSIONS: Lazy<Mutex<HashMap<String, Session>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: Uuid,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_activity: SystemTime,
    pub ip_address: String,
    pub user_agent: String,
}

impl Session {
    pub fn new(user_id: Uuid, ip_address: String, user_agent: String) -> Self {
        let now = SystemTime::now();
        let session_id = Uuid::new_v4().to_string();
        let expires_at = now + Duration::from_secs(24 * 3600); // 24 hours

        Self {
            session_id,
            user_id,
            created_at: now,
            expires_at,
            last_activity: now,
            ip_address,
            user_agent,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now();
    }
}

pub fn create_session(user_id: Uuid, ip_address: String, user_agent: String) -> Result<Session, AppError> {
    let session = Session::new(user_id, ip_address, user_agent);
    let mut sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    sessions.insert(session.session_id.clone(), session.clone());
    Ok(session)
}

pub fn get_session(session_id: &str) -> Result<Option<Session>, AppError> {
    let sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    Ok(sessions.get(session_id).cloned())
}

pub fn update_session_activity(session_id: &str) -> Result<(), AppError> {
    let mut sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    if let Some(session) = sessions.get_mut(session_id) {
        session.update_activity();
    }
    Ok(())
}

pub fn remove_session(session_id: &str) -> Result<(), AppError> {
    let mut sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    sessions.remove(session_id);
    Ok(())
}

pub fn cleanup_expired_sessions() -> Result<(), AppError> {
    let mut sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    sessions.retain(|_, session| !session.is_expired());
    Ok(())
}

pub fn get_active_sessions(user_id: Uuid) -> Result<Vec<Session>, AppError> {
    let sessions = SESSIONS.lock().map_err(|_| AppError::InternalServerError)?;
    Ok(sessions
        .values()
        .filter(|session| session.user_id == user_id && !session.is_expired())
        .cloned()
        .collect())
}