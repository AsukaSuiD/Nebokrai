//! Player-side состояние и правила аукциона живого игрока исторического
//! GameServer: флаг окна, поисковый фильтр/страница, timestamp-гейты, плата за
//! выставление, ожидающие узлы и правила допуска товара и возврата денег.
//! Исходный владелец `appserver/player.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`.
//!
//! Контейнеры `auction_listing/auction_goods/auction_wallet`, `CGoods` и фабрика
//! остаются переходными владельцами старого пакета и передают сюда только
//! скаляры; порядок сообщений, journal-отправок, billing-ожидание и сборка узла
//! — у message runtime caller-ов. `CGoodsNode` принадлежит Realm
//! `auction/auctionnode.rs`, поэтому состояние параметризовано типом узла
//! `Node`; `LegacyReader/Writer` здесь не появляются.
//!
//! UNKNOWN: заполнение таблицы `IsGoodAllowedInAuction` — INFERRED; Rust
//! использует конфиг-список `GlobeSetupSnapshot::auction_goods_allowed` (Shared
//! resources). Fee-формулы `GetOptMoneyJin/Yuan` здесь не реализованы.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#торговля-и-деньги

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionMoneyMoveCapacity {
    pub wallet_amount: u32,
    pub auction_amount: u32,
    pub maximum: u32,
    pub allowed: bool,
}

/// State-часть `CheckAuctionMoneyMove` `0x000369B0`: unsigned overflow суммы
/// основного и auction wallet делает перенос невозможным, иначе сумма должна
/// укладываться в max stack основного wallet.
pub fn check_auction_money_move(
    wallet_amount: u32,
    auction_amount: u32,
    maximum: u32,
) -> AuctionMoneyMoveCapacity {
    let allowed = wallet_amount
        .checked_add(auction_amount)
        .is_some_and(|total| total <= maximum);
    AuctionMoneyMoveCapacity {
        wallet_amount,
        auction_amount,
        maximum,
        allowed,
    }
}

/// `AuctionLimit` `0x0002EF00` для ячейки выставления `0`: принимается только
/// предмет без particular-флагов `0x20`/`0x04` и без life-type addon (`0xE5`).
/// Player state в формуле не участвует; скаляры снимает вызывающая сторона с
/// `CGoods`/фабрики.
pub fn auction_listing_goods_allowed(particular_attribute: u32, has_life_type: bool) -> bool {
    particular_attribute & 0x20 == 0 && particular_attribute & 0x04 == 0 && !has_life_type
}

/// Текстовый dotted-quad IPv4-адреса, укладываемый в `AuctionInfo` узла;
/// слово адреса хранится little-endian, как у сетевого peer record.
pub fn legacy_ipv4_text(ip: u32) -> Vec<u8> {
    format!(
        "{}.{}.{}.{}",
        ip & 0xff,
        (ip >> 8) & 0xff,
        (ip >> 16) & 0xff,
        ip >> 24
    )
    .into_bytes()
}

/// Gate покупки лота `BuyItemFromAauction`: отдельный второй sample
/// записывается до GUID decode/query вызывающей стороны.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionBuyGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

/// Gate выставления лота `MakeCurAucNode` с отдельным вторым tick sample.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionListingGate {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Ready {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
    },
}

/// Gate обновления собственных лотов `ReFlushSelfGoods`; `goods/wallet space`
/// собираются фасадом из контейнеров после прохождения gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionSelfGoodsRefresh {
    Throttled {
        sampled_tick_ms: u32,
        previous_tick_ms: u32,
    },
    Requested {
        sampled_tick_ms: u32,
        recorded_tick_ms: u32,
        goods_space: u32,
        wallet_space: u32,
    },
}

/// Вложенное в живого игрока состояние аукциона: окно, поиск, плата,
/// timestamp-гейты и два ожидающих узла. Тип узла параметризован, потому что
/// `CGoodsNode` принадлежит Realm auction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerAuction<Node> {
    open: bool,
    search_name: Vec<u8>,
    search_lower_level: i32,
    search_upper_level: i32,
    search_use_self: i32,
    search_money_type: i32,
    search_weapon_type: i32,
    current_page: i32,
    last_limit_tick_ms: u32,
    last_option_tick_ms: u32,
    current_node: Option<Node>,
    listing_fee: u32,
    current_buy_node: Option<Node>,
}

impl<Node> Default for PlayerAuction<Node> {
    fn default() -> Self {
        Self {
            open: false,
            search_name: Vec::new(),
            search_lower_level: 0,
            search_upper_level: 0,
            search_use_self: 0,
            search_money_type: 0,
            search_weapon_type: 0,
            current_page: 0,
            last_limit_tick_ms: 0,
            last_option_tick_ms: 0,
            current_node: None,
            listing_fee: 0,
            current_buy_node: None,
        }
    }
}

impl<Node> PlayerAuction<Node> {
    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub const fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    /// Строгий wrapping threshold `BuyItemFromAauction`: отказ при
    /// `sampled ≤ last + 5000` (signed `jbe` после wrapping-сложения), второй
    /// сэмпл записывается до GUID decode/query вызывающей стороны.
    pub fn begin_buy(&mut self, mut tick_ms: impl FnMut() -> u32) -> AuctionBuyGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionBuyGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_option_tick_ms = recorded_tick_ms;
        AuctionBuyGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// Тот же 5-секундный двухсэмпловый gate у выставления `MakeCurAucNode`.
    pub fn begin_listing(&mut self, mut tick_ms: impl FnMut() -> u32) -> AuctionListingGate {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionListingGate::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_option_tick_ms = recorded_tick_ms;
        AuctionListingGate::Ready {
            sampled_tick_ms,
            recorded_tick_ms,
        }
    }

    /// 1-секундный gate `IsAollowAuction`: timestamp обновляется до limit
    /// queries, поэтому failed limit также поглощает текущую попытку.
    /// Owner/global count и оба threshold-значения снимает вызывающая сторона
    /// с комнаты и setup-снимка.
    pub fn begin_limit_check(
        &mut self,
        tick_ms: u32,
        owner_goods_count: usize,
        global_goods_count: usize,
        player_maximum: f32,
        global_maximum: f32,
        extension_bonus: i32,
    ) -> bool {
        if tick_ms.wrapping_sub(self.last_limit_tick_ms) <= 1_000 {
            return false;
        }
        self.last_limit_tick_ms = tick_ms;
        (owner_goods_count as f32) < extension_bonus as f32 + player_maximum
            && (global_goods_count as f32) < global_maximum
    }

    /// Тот же strict wrapping `last + 5000 < first sample` у
    /// `ReFlushSelfGoods`; отдельный второй `timeGetTime` sample записывается
    /// до World send.
    pub fn begin_self_goods_refresh(
        &mut self,
        mut tick_ms: impl FnMut() -> u32,
        goods_space: u32,
        wallet_space: u32,
    ) -> AuctionSelfGoodsRefresh {
        let sampled_tick_ms = tick_ms();
        let previous_tick_ms = self.last_option_tick_ms;
        if previous_tick_ms.wrapping_add(5_000) >= sampled_tick_ms {
            return AuctionSelfGoodsRefresh::Throttled {
                sampled_tick_ms,
                previous_tick_ms,
            };
        }
        let recorded_tick_ms = tick_ms();
        self.last_option_tick_ms = recorded_tick_ms;
        AuctionSelfGoodsRefresh::Requested {
            sampled_tick_ms,
            recorded_tick_ms,
            goods_space,
            wallet_space,
        }
    }

    pub fn begin_search(
        &mut self,
        name: &[u8],
        lower_level: i32,
        upper_level: i32,
        use_self: i32,
        money_type: i32,
        weapon_type: i32,
    ) {
        self.search_name.clear();
        self.search_name.extend_from_slice(name);
        self.search_lower_level = lower_level;
        self.search_upper_level = upper_level;
        self.search_use_self = use_self;
        self.search_money_type = money_type;
        self.search_weapon_type = weapon_type;
        self.current_page = 0;
    }

    pub const fn listing_fee(&self) -> u32 {
        self.listing_fee
    }

    pub const fn set_listing_fee(&mut self, fee: u32) {
        self.listing_fee = fee;
    }

    pub fn current_node(&self) -> Option<&Node> {
        self.current_node.as_ref()
    }

    pub fn set_current_node(&mut self, node: Node) -> bool {
        if self.current_node.is_some() {
            return false;
        }
        self.current_node = Some(node);
        true
    }

    /// Точная запись `AutoAddAuctionGoods`: каждый созданный предмет целиком
    /// заменяет предыдущий `m_CurrentAucNode` без проверки занятости узла.
    pub fn replace_current_node(&mut self, node: Node) {
        self.current_node = Some(node);
    }

    pub fn take_current_node(&mut self) -> Option<Node> {
        self.current_node.take()
    }

    pub fn current_buy_node(&self) -> Option<&Node> {
        self.current_buy_node.as_ref()
    }

    pub fn set_current_buy_node(&mut self, node: Node) -> bool {
        if self.current_buy_node.is_some() {
            return false;
        }
        self.current_buy_node = Some(node);
        true
    }

    pub fn take_current_buy_node(&mut self) -> Option<Node> {
        self.current_buy_node.take()
    }
}
