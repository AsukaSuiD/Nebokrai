//! Player-side состояние и правила аукциона живого игрока GameServer,
//! перенесённые в Zone `trade/`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `server/gameserver/appserver/player.cpp`. Здесь живёт вложенное в игрока
//! состояние аукциона: флаг открытого окна, поисковый фильтр и текущая
//! страница, оба операционных timestamp-гейта, плата за выставление и два
//! ожидающих узла (выставление и покупка), а также чистые правила допуска
//! товара в ячейку выставления и вместимости кошельков при возврате денег.
//! Контейнеры `auction_listing/auction_goods/auction_wallet`, `CGoods` и
//! фабрика остаются переходными владельцами старого пакета; операции над ними
//! исполняют фасады `CPlayer` и передают сюда только скаляры. Порядок
//! сообщений, journal-отправок, billing-ожидание и сборка узла остаются у
//! message runtime caller-ов.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Машинные статусы по дизассемблу тел точной пары (разведка
//! trade/auction/bank/ground currency, запись аудита «Zone player: машинная
//! разведка trade/auction/bank/ground currency — MATCH по подсемействам»
//! от 26 сентября 2026):
//!
//! | функция | RVA | здесь | статус |
//! |---|---|---|---|
//! | `CPlayer::AuctionLimit` | `0x0002EF00` | [`auction_listing_goods_allowed`] | `VERIFIED_DISASSEMBLY` (побитовая логика GAP13: `&0x20→2`, `&0x04→−1`, attr `0xE5`→`−1`, иначе `3`; bool-форма — допуск «иначе 3») |
//! | `CPlayer::CheckAuctionMoneyMove` | `0x000369B0` | [`check_auction_money_move`] | `VERIFIED_DISASSEMBLY` (checked unsigned sum обоих wallet против max stack основного; notify `GPM015` у caller-а) |
//! | `CPlayer::IsAollowAuction` | `0x00036200` | [`PlayerAuction::begin_limit_check`] | `VERIFIED_DISASSEMBLY` (1-сек gate `[+0xCD0]`: timestamp до limit-queries; owner count `CAuctionRoom@CGame+0x1B8`; bonus slot1 `GAP 0xEA == 3 → value2`; float-compare; отказ поглощает попытку + notify) |
//! | `CPlayer::BuyItemFromAauction` | `0x00030BC0` | [`PlayerAuction::begin_buy`] | `VERIFIED_DISASSEMBLY` (reject при `sampled ≤ last + 5000` — signed `jbe` после wrapping-сложения; второй сэмпл записывается до GUID decode) |
//! | `CPlayer::MakeCurAucNode` | `0x00045A50` | [`PlayerAuction::begin_listing`], [`PlayerAuction::set_current_node`] | `VERIFIED_DISASSEMBLY` (5-сек двухсэмпловый gate; pipeline `IsAollow → IsMoney → IsCurAucNodeOK → DeleteGoods → узел` у caller-а) |
//! | `CPlayer::ReFlushSelfGoods` | `0x00030D90` | [`PlayerAuction::begin_self_goods_refresh`] | `VERIFIED_DISASSEMBLY` (тот же 5-сек gate; `goods/wallet space` собирает фасад из контейнеров) |
//! | `CPlayer::OpenAuction` | `0x000367B0` | [`PlayerAuction::set_open`] | `VERIFIED_DISASSEMBLY` (клиент `0xC0706` 11 dword → World `0x60810` 1 dword — у caller-а) |
//! | `CPlayer::AutoAddAuctionGoods` | `0x000466A0` | [`PlayerAuction::replace_current_node`] | `VERIFIED_DISASSEMBLY` (каждый созданный предмет безусловно заменяет `m_CurrentAucNode`) |
//! | `CPlayer::AddItemToAuction` | `0x00045910` | [`PlayerAuction::take_current_node`], [`PlayerAuction::set_listing_fee`] | `VERIFIED_DISASSEMBLY` (порядок `node Serialize → World 0x60801 → SendSaleLog → AddByteGS2WS → TellClietAuctionOK → Clear` у caller-а) |
//!
//! Соседние владельцы, не вошедшие в этот файл: `ModifyAuctionSpace`
//! `0x0003F7A0`, `GetAuctionMoney` `0x0002EEF0` (tail-jmp amount `[+0x8F0]`)
//! и `SetAuctionMoney` `0x00030E60` — wallet/container мутации, остаются в
//! фасаде `CPlayer` до порции Game-контейнеров; `TellClietAuctionOK`
//! `0x0003ED50` (`0xC0701`, `Unserialize → SerializeForOldClient → Add`),
//! `WriteBuyAuctionLog` `0x00036370` (`0x60214`), `SendToGSBaiTan`
//! `0x0002F1C0` / `NoticyWS_BaiTan_Over` `0x0002F140` (`0x60811`/`0x60812`) —
//! message runtime caller-ы прежних обработчиков. fee-формулы
//! `GetOptMoneyJin/Yuan` (x87) не переносятся — отдельная buy-порция.
//!
//! UNKNOWN: `IsGoodAllowedInAuction` `0x0002EA50` — линейный скан 256 dword
//! таблицы `0xEF4AC8`; таблица в образе нулевая, её заполнение — `INFERRED`.
//! Rust использует принятый конфиг-список `globesetup`
//! (`GlobeSetupSnapshot::auction_goods_allowed`, Shared resources), что
//! поведенчески совпадает при заполненном конфиге; сама проверка живёт вне
//! этого файла у setup-снимка.
//!
//! Швы переноса: `CGoodsNode` принадлежит Realm `auction/auctionnode.rs`, а
//! Zone не импортирует владельцев другой роли, поэтому состояние
//! параметризовано типом узла `Node`; мгновенный владелец — `CPlayer`
//! (`PlayerAuction<CGoodsNode>`). `LegacyReader/Writer` и wire-кадры здесь не
//! появляются: сборка и разбор сообщений — за message runtime caller-ами.

/// State-отчёт `CheckAuctionMoneyMove`: checked сумма обоих кошельков
/// сравнивается с max stack основного кошелька.
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
