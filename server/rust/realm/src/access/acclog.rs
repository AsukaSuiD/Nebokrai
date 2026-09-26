//! Typed-записи журнала аккаунтов LoginServer. Оригинальный
//! `AccLogQueue::push(char*)` принимал промежуточную C-строку; SQL строится
//! consumer-ом в исходной producer-позиции времени, поэтому записи хранят
//! owned-поля и момент `recorded_at`.

use chrono::NaiveDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountEnterRecord {
    pub account: Vec<u8>,
    pub client_ip: u32,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountLeaveRecord {
    pub account: Vec<u8>,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleEnterRecord {
    pub account: Vec<u8>,
    pub role_name: Vec<u8>,
    pub role_level: u8,
    pub world_number: i32,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionLeaveRecord {
    pub account: Vec<u8>,
    pub recorded_at: NaiveDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccountLogRecord {
    Enter(AccountEnterRecord),
    RoleEnter(RoleEnterRecord),
    SessionLeave(SessionLeaveRecord),
    Leave(AccountLeaveRecord),
}
