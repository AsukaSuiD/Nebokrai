//! Достигнутая plug-list storage-часть GameServer `CSession`, перенесённая в Zone `sessions/`.
//!
//! PDB `GameServer/GameServer.pdb` фиксирует `std::list<long> m_lPlugs` по
//! `+0x64`; inline `GetPlugList` RVA `0x00070910` экспортирован из точного
//! source-owner `server/gameserver/appserver/area.cpp` и материализован теперь
//! здесь вместе с типом (ранее дополнительным impl в area.rs).
//! `Vec<i32>` сохраняет порядок обхода. Normal equipment-session materializes
//! constructor defaults, Start gate и InsertPlug capacity/state prefix;
//! team create/restore и terminal lifecycle используют тот же storage.
//! `from_plug_ids` является assembly-границей уже восстановленного registry
//! state. Для normal equipment-session материализован terminal `End`:
//! ended/remove state и ordered обход plug IDs; concrete plug callback/registry
//! lookup выполняет `CSessionFactory`. `Start` сохраняет caller-owned DWORD
//! tick, `Serialize` выдаёт wrapping остаток lifetime, а base `AI` lifetime
//! gate подключён к concrete team owner до его derived idle-проверки.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CSession {
    plug_ids: Vec<i32>,
    minimum_plugs: u32,
    maximum_plugs: u32,
    lifetime: u32,
    starting_time_stamp: u32,
    started: bool,
    ended: bool,
    aborted: bool,
    remove_requested: bool,
}

impl CSession {
    pub const fn from_plug_ids(plug_ids: Vec<i32>) -> Self {
        Self {
            plug_ids,
            minimum_plugs: 0,
            maximum_plugs: u32::MAX,
            lifetime: 0,
            starting_time_stamp: 0,
            started: true,
            ended: false,
            aborted: false,
            remove_requested: false,
        }
    }

    pub const fn normal(minimum_plugs: u32, maximum_plugs: u32, lifetime: u32) -> Self {
        Self {
            plug_ids: Vec::new(),
            minimum_plugs,
            maximum_plugs,
            lifetime,
            starting_time_stamp: 0,
            started: false,
            ended: false,
            aborted: false,
            remove_requested: false,
        }
    }

    pub const fn start(&mut self, now_ms: u32) -> bool {
        if self.started || self.ended || self.aborted {
            return false;
        }
        self.starting_time_stamp = now_ms;
        self.started = true;
        true
    }

    pub fn insert_plug(&mut self, plug_id: i32) -> bool {
        if !self.started
            || self.ended
            || self.aborted
            || self.maximum_plugs as usize <= self.plug_ids.len()
        {
            return false;
        }
        self.plug_ids.push(plug_id);
        true
    }

    pub fn remove_plug(&mut self, plug_id: i32) -> bool {
        let Some(position) = self.plug_ids.iter().position(|id| *id == plug_id) else {
            return false;
        };
        self.plug_ids.remove(position);
        true
    }

    pub fn plug_ids_storage(&self) -> &[i32] {
        &self.plug_ids
    }

    pub fn end(&mut self) -> Vec<i32> {
        self.starting_time_stamp = 0;
        self.ended = true;
        self.remove_requested = true;
        self.plug_ids.clone()
    }

    pub fn abort(&mut self) -> Vec<i32> {
        self.starting_time_stamp = 0;
        self.aborted = true;
        self.remove_requested = true;
        self.plug_ids.clone()
    }

    pub const fn is_ended(&self) -> bool {
        self.started && self.ended
    }

    pub const fn remove_requested(&self) -> bool {
        self.remove_requested
    }

    pub const fn is_available_prefix(&self) -> bool {
        self.started && !self.ended && !self.aborted
    }

    pub const fn minimum_plugs(&self) -> u32 {
        self.minimum_plugs
    }

    pub const fn maximum_plugs(&self) -> u32 {
        self.maximum_plugs
    }

    /// Exact base `AI` lifetime gate: legacy compares the wrapping DWORD sum
    /// with the current `timeGetTime` sample and ignores dormant sessions.
    pub const fn lifetime_expired(&self, now_ms: u32) -> bool {
        self.started
            && !self.ended
            && !self.aborted
            && self.lifetime != 0
            && self.starting_time_stamp.wrapping_add(self.lifetime) <= now_ms
    }

    /// `Serialize` publishes the remaining lifetime, not the constructor
    /// duration. Zero-lifetime sessions keep the literal zero wire value.
    pub const fn remaining_lifetime(&self, now_ms: u32) -> u32 {
        if self.lifetime == 0 {
            0
        } else {
            self.starting_time_stamp
                .wrapping_add(self.lifetime)
                .wrapping_sub(now_ms)
        }
    }
}

impl CSession {
    /// Inline `GetPlugList` RVA `0x00070910` (`lea eax, [ecx+0x64]; ret`)
    /// из source-owner `area.cpp`; ранее был дополнительным impl в area.rs.
    pub fn get_plug_list(&self) -> &[i32] {
        self.plug_ids_storage()
    }
}
