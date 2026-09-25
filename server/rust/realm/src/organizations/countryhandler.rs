//! Data-контракты карты государств `CCountryHandler` из
//! `countryhandler.cpp/.h`, подтверждённые точной парой `worldserver.exe` и
//! `worldserver.pdb`. Data-уровень перенесён в Realm `organizations/`.
//!
//! Run/new-day/initialize/append отчёты handler-а цитируют ещё не перенесённый
//! `KingPointUpdate` (через AI- и new-day семьи) и саму `CCountry` — они
//! переносятся вместе с этими владельцами. `CountryHandlerSerializeError`
//! остаётся волне servermessage-типов.

use crate::app::world_message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryHandlerReleaseReport {
    pub released_countries: usize,
    pub released_top_infos: usize,
}

pub trait CountryInfoDeliveryContext {
    fn send_all(&mut self, message: &CMessage) -> i32;
}
