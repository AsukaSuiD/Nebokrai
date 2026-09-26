//! Реестр goods link мира (исходный `CGame::m_listGoodsLink` и его
//! process-global счётчик индексов) — primary state владельца `social`
//! (прежнее transitional-поле `CGame`). Типы `WorldGoodsLink`/
//! `WorldGoodsLinkPayload` перенесены из `app/worldothermessage`; прежний
//! путь сохранён re-export-ом для прежних consumers.
//!
//! `CGame` хранит только composition handle `goods_links` и делегирует прежний
//! pub facade построчно. Constructor-контракт владельца — 500 нулевых
//! placeholder-ов списка; process-global индекс начинается с `1` и не
//! сбрасывается между Init. Typed add сохраняет legacy-семантику:
//! `pop_front` на `list::max_size`, changed-запись сохраняет ID товара без
//! хода global-а; find — первое list-order совпадение включая placeholder-ы
//! индекса `0`. Release-clear остаётся оркестрацией app (событие
//! `ReleaseGoodsLinks`).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::content::cgoods::CGoods;

const INITIAL_GOODS_LINK_PLACEHOLDERS: usize = 500;

const LEGACY_GOODS_LINK_MAX_SIZE: usize = 0x0CCC_CCCC;

/// Процессно-глобальный индекс original-ссылок; буквально не сбрасывается
/// между Init (static процесса, а не поле игры).
static NEXT_GOODS_LINK_INDEX: AtomicU32 = AtomicU32::new(1);

/// Владеющая Rust-форма точного 20-байтового `CGame::tagGoodsLink`.
///
/// `Box<CGoods>` заменяет сырой owning pointer только для `bChange != 0`;
/// unchanged-запись хранит исходные `dwType/lNum`. Старый padding не
/// Создаётся, потому что ни lookup, ни wire его не наблюдают.
pub enum WorldGoodsLinkPayload {
    Changed(Box<CGoods>),
    Original { goods_type: u32, amount: u8 },
}

pub struct WorldGoodsLink {
    pub index: u32,
    pub payload: WorldGoodsLinkPayload,
}

impl WorldGoodsLink {
    pub fn placeholder() -> Self {
        Self {
            index: 0,
            payload: WorldGoodsLinkPayload::Original {
                goods_type: 0,
                amount: 0,
            },
        }
    }

    pub fn changed(goods: Box<CGoods>) -> Self {
        Self {
            index: goods.get_id() as u32,
            payload: WorldGoodsLinkPayload::Changed(goods),
        }
    }

    pub const fn original(goods_type: u32, amount: u8) -> Self {
        Self {
            index: 0,
            payload: WorldGoodsLinkPayload::Original { goods_type, amount },
        }
    }

    pub const fn payload(&self) -> &WorldGoodsLinkPayload {
        &self.payload
    }
}

/// Действующий реестр goods link мира.
///
/// Constructor создаёт 500 нулевых placeholder-ов исходного списка, а
/// process-global индекс остаётся общим для всех экземпляров процесса.
pub struct WorldGoodsLinks {
    links: VecDeque<WorldGoodsLink>,
}

impl WorldGoodsLinks {
    pub fn new() -> Self {
        Self {
            links: std::iter::repeat_with(WorldGoodsLink::placeholder)
                .take(INITIAL_GOODS_LINK_PLACEHOLDERS)
                .collect(),
        }
    }

    /// Добавляет точную POD-запись в хвост `m_listGoodsLink`.
    ///
    /// Constructor уже создал 500 нулевых placeholder-ов, а process-global
    /// индекс начинается с `1`. Changed-запись сохраняет ID декодированного
    /// товара и global не двигает. Редкая `list::max_size` ветвь удаляет голову;
    /// Rust одновременно освобождает её owned товар, исправляя только утечку.
    pub fn add_goods_link(&mut self, mut link: WorldGoodsLink) -> u32 {
        if self.links.len() == LEGACY_GOODS_LINK_MAX_SIZE {
            let _ = self.links.pop_front();
        }
        if matches!(&link.payload, WorldGoodsLinkPayload::Original { .. }) {
            link.index = NEXT_GOODS_LINK_INDEX.fetch_add(1, Ordering::Relaxed);
        }
        let index = link.index;
        self.links.push_back(link);
        index
    }

    /// Возвращает первое совпадение в list-order, включая constructor-ный
    /// placeholder для индекса `0`.
    pub fn find_goods_link(&self, index: u32) -> Option<&WorldGoodsLink> {
        self.links.iter().find(|link| link.index == index)
    }

    pub fn clear(&mut self) {
        self.links.clear();
    }
}
