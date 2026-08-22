//! DB-владелец `CDBGoods` исторического WorldServer из `dbgoods.cpp`.
//!
//! `DeleteGoods` RVA `0x001174F0`, `SaveGoodsFiled` RVA `0x001175D0`,
//! `SaveGoodsProperties` RVA `0x001178C0` и `SaveGoods` RVA `0x00117B30` имеют
//! статус `IMPLEMENTED`; caller-connection путь `LoadGoods` RVA `0x00117E20` —
//! `IMPLEMENTED_PARTIAL/VERIFIED_DISASSEMBLY`, остальные функции ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbgoods.cpp`.
//!
//! Exact `0x005174F0..0x005175CE` восстанавливает потерянные raw vararg и
//! `AL`: SQL получает signed `playerID`, успешный `ExecuteCn` возвращает
//! `true`, null connection и DB-ошибка — `false`. Исходный owner удаляет только
//! строки `player_goods` и не выполняет отдельный DELETE из
//! `extend_properties`; Rust сохраняет эту границу и не назначает поведение
//! возможному DB constraint/cascade вне достигнутой функции. Ссылка исключает
//! старый null connection, число затронутых строк по-прежнему не проверяется.
//!
//! PDB задаёт `CGoods::tagAddonProperty` размером `0x1C`: `gapType` по `+0`,
//! два runtime-флага по `+4/+8` и vector значений по `+0xC`; флаги при save не
//! читаются. `tagAddonPropertyValue` имеет размер `0xC`: `dwId: unsigned long`,
//! `lBaseValue: long`, `lModifier: long`. Snapshot заранее получает результат
//! `CGoodsBaseProperties::GetOccurProbability`, не перенося фабрику и частично
//! восстановленный игровой объект внутрь DB-owner-а.
//!
//! Exact `0x005178C0..0x00517B2C` подтверждает странность оригинала: четыре
//! accumulator-а обнуляются один раз до внешнего цикла и не сбрасываются между
//! addon properties. Значение с `dwId == 1` обновляет первый base/modifier,
//! `dwId == 2` — второй; остальные ID игнорируются. Для type `0x25` запись
//! создаётся при `modifier1 != 0 || base1 != base2` и получает второй base;
//! для остальных type при probability не `10000` запись обязательна, а при
//! `10000` пропускается только нулевая пара modifier-ов. Этот carry-state
//! сохранён буквально, а не «исправлен».
//!
//! Старый внутренний индекс значений жил в `BL`: vector длиннее 255 элементов
//! заставлял его циклически переполняться и не завершать обход. Конструктор
//! Rust-snapshot принимает только доказанный завершающийся диапазон `0..=255`;
//! отказ материализации явно сообщает эту legacy-границу. Параметризованный
//! TDS сохраняет значения тех же четырёх колонок и последовательность отдельных
//! INSERT; каждый исходный batch гарантированно короче `char[512]` даже для
//! крайних 32-битных чисел и 38-символьного GUID. `tiberius`, stream
//! consumption и Rust Drop заменяют только ADO/COM, `_sprintf` и cleanup.
//!
//! PDB задаёт source-поля одного goods snapshot: inherited
//! `CBaseObject::m_guExID: CGUID` по `+0xC`, `m_strName: std::string` по `+0x20`,
//! затем `CGoods::m_dwBasePropertiesIndex/m_dwAmount/m_dwPrice: unsigned long`
//! по `+0x6C/+0x70/+0x74`. `place` и `position` являются параметрами
//! `unsigned char`; listener отдельно обрезает найденный `unsigned long`
//! position до младшего байта при вызове. Player ID — signed `long` по
//! `CBaseObject +0x8`.
//!
//! `SaveGoods` создаёт новый row GUID, но сохраняет исходный goods GUID отдельно
//! в `GoodsID`. Затем `FixSingleQuotes` удваивает каждый apostrophe в C-string-
//! части имени в `char[256]`, а `_sprintf` строит точный batch в `char[1024]`.
//! Rust-параметры передают те же значения без SQL injection. Выход escaping за
//! первую ёмкость остаётся локальным `BLOCKED_MISSING_FACT`, потому что результат
//! buffer overflow не доказан. При безопасном escaped имени максимальный
//! `_sprintf` batch вместе с NUL равен 499 байтам, поэтому второй `char[1024]`
//! доказанно достаточен. Имя декодируется из Windows-1251, `unsigned long` с
//! `%d` сохраняет тот же signed 32-битный шаблон, GUID остаются в верхнем
//! регистре и скобках.
//!
//! Exact `0x00517B30..0x00517E1F` возвращает все выходы: ошибка main INSERT,
//! отсутствие base-properties для непустого addon vector и отказ вложенного
//! property-helper дают `false`; пустые properties либо полный успех дают
//! `true`. Основная строка уже существует внутри caller-транзакции до позднего
//! `false`. Вызов `AddPlayerList` RVA `0x00001000`, ошибочно похожий в raw на
//! cleanup с connection pointer, является доказанной пустой project-функцией
//! (`ret`) и не получает Rust-аналога. Исходный `CreateGUID` игнорировал HRESULT-
//! bool; если Linux RNG не отдаёт полный GUID, Rust не назначает неизвестные
//! частичные байты и возвращает отдельный `BLOCKED_MISSING_FACT`.
//!
//! `SaveGoodsFiled` сначала отвергает null connection, затем удаляет прежние
//! строки игрока и при `DeleteGoods == false` бросает `E_FAIL` в собственный
//! catch. Только после успешного удаления он обходит ровно пятнадцать полей
//! `CPlayer`, каждый раз назначая listener-у place: `m_cPacket(+0xE0)=1`,
//! `m_cEquipment(+0x154)=2`, `m_cHand(+0x80)=3`, `m_cWallet(+0x184)=4`,
//! `m_cYuanBao(+0x1AC)=5`, `m_cJiFen(+0x1D4)=6`, `m_cBank(+0x1FC)=7`,
//! `m_cDepot(+0x228)=8`, `m_cFairy(+0x2A0)=9`, `m_cBF(+0x328)=10`,
//! `m_cAuctionGoodsContainer(+0x3A0)=11`, `m_cAuctionWallet(+0x488)=12`,
//! `m_cAuctionContainer(+0x414)=13`, `m_cCiQing(+0x4B0)=14` и
//! `m_cComposeCiQing(+0x524)=15`. PDB и достигнутый `CPlayer` подтверждают имена
//! и offsets; порядок и byte-place подтверждены телом owner-а.
//!
//! Exact `0x005175D0..0x005178B7` имеет статус `VERIFIED_DISASSEMBLY` для
//! потерянного raw-эпилога: null connection и catch сходятся к `xor al,al`, а
//! полный обход — к `mov al,1`. Все достигнутые container traversal-ы имеют
//! `void` и игнорируют callback `int`, поэтому отдельный `SaveGoods == false`
//! не останавливает обход и не меняет финальный `true`. Rust сохраняет этот
//! частичный эффект; только собственный `BLOCKED_MISSING_FACT` вложенного save
//! не превращается в придуманный успешный исход и передаётся наружу.
//!
//! Catch после отказа delete выполнял и `AddErrorLogText`, и `PrintErr`; typed
//! `NestedDeleteFailed` объединяет их факт с уже поставленным внутренним DB-
//! notice, не копируя SQL либо player ID. В исходном `__snprintf` размер был
//! буквально `4`, поэтому задуманная строка с player ID обрезалась и могла не
//! получить NUL; Rust не воспроизводит чтение за stack-buffer. Null connection
//! сохраняется отдельным `MissingConnection`. Caller-owned container snapshots
//! меняют только ещё не материализованный полный `CPlayer` layout: каждый
//! snapshot обязан сохранить порядок конкретного исходного map/list/wallet.
//!
//! `LoadGoods` сначала очищает все пятнадцать container-ов в exact порядке и
//! восстанавливает их limit/volume, затем читает исходный left join
//! `player_goods/extend_properties`. Повторная строка той же place/position
//! mutates уже вставленный goods, поэтому несколько addon rows сохраняют
//! исходную построчную семантику без staging позднего Linux-donor-а. Tiberius
//! parameter binding заменяет `_sprintf`, `encoding_rs` — BSTR/ANSI conversion,
//! `uuid` — валидный GUID parse, а Rust ownership — factory/STL cleanup.
//! `ChangeGoodsIndexMap` и множество DaKong-type передаются caller-owned
//! `BTreeMap/BTreeSet`, не создавая недоказанный mutable singleton.
//!
//! Exact normal tail `0x005190E3` выполняет `mov al,1`, catch-tail
//! `0x005191A4` — `xor al,al`. Поэтому неизвестный place, null factory-result
//! и rejected container insertion остаются успешными skipped rows; DB/ADO
//! ошибки дают false. Небезопасные края исходника — повреждённый GUID длиной
//! 38, невыразимое ANSI-имя, отказ RNG при смене индекса и addon-vector короче
//! двух значений — не получают выдуманного результата и возвращают typed
//! `BlockedMissingFact`. Автономная ветка `connection == null` пока честно
//! обозначена `PendingStandaloneConnection`; основной `CRsPlayer::LoadPlayer`
//! теперь либо переиспользует caller-owned connection, либо сам открывает одно
//! connection и передаёт его goods-owner-у.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::Query;

use crate::dbaccess::worlddb::goodslistener::{
    GoodsContainerTraversalSnapshot, GoodsListener, GoodsTraversalBlock,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoods::GoodsLoadedAddonBlock;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods_no_probability,
};
use crate::worldserver::appworld::player::{CPlayer, PlayerLoadedGoodsInsertBlock};

/// Точное содержимое одного `CGoods::tagAddonPropertyValue`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GoodsAddonPropertyValue {
    pub(crate) id: u32,
    pub(crate) base_value: i32,
    pub(crate) modifier: i32,
}

/// Ошибка материализации старого byte-indexed цикла значений.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GoodsAddonValueCountBlock {
    pub(crate) value_count: usize,
}

/// Caller-owned view одного addon property и его достигнутого base-property.
pub(crate) struct GoodsAddonPropertySnapshot {
    property_type: u32,
    occur_probability: u32,
    values: Vec<GoodsAddonPropertyValue>,
}

impl GoodsAddonPropertySnapshot {
    /// Материализует только диапазон, в котором исходный `unsigned char` loop
    /// действительно достигал конца vector.
    pub(crate) fn from_legacy_parts(
        property_type: u32,
        occur_probability: u32,
        values: Vec<GoodsAddonPropertyValue>,
    ) -> Result<Self, GoodsAddonValueCountBlock> {
        if values.len() > u8::MAX as usize {
            return Err(GoodsAddonValueCountBlock {
                value_count: values.len(),
            });
        }
        Ok(Self {
            property_type,
            occur_probability,
            values,
        })
    }
}

/// Состояние lookup-а base-properties после успешного main goods INSERT.
pub(crate) enum GoodsPropertiesSnapshot {
    /// Пустой либо полностью материализованный addon vector.
    Available(Vec<GoodsAddonPropertySnapshot>),
    /// Addon vector непуст, но `QueryGoodsBaseProperties` вернул null.
    MissingBaseProperties,
}

/// Caller-owned view одного `CGoods`, общий для обхода контейнера и DB-save.
pub(crate) struct GoodsObjectSnapshot {
    pub(crate) goods_id: CGuid,
    pub(crate) base_properties_index: u32,
    /// Полный byte-content старого `std::string`; save читает его как C-string.
    pub(crate) name: Vec<u8>,
    pub(crate) price: u32,
    pub(crate) amount: u32,
    pub(crate) properties: GoodsPropertiesSnapshot,
}

/// Полный caller-owned view одного вызова `CDBGoods::SaveGoods`.
pub(crate) struct GoodsSaveSnapshot<'goods> {
    pub(crate) player_id: i32,
    pub(crate) goods: &'goods GoodsObjectSnapshot,
    pub(crate) place: u8,
    pub(crate) position: u8,
}

/// Неразрешённая legacy-граница `SaveGoods`.
#[derive(Debug)]
pub(crate) enum GoodsSaveBlock {
    /// `CreateGUID`-аналог не получил полный идентификатор из системного RNG.
    GuidGeneration(getrandom::Error),
    /// `FixSingleQuotes` вышел бы за `char[256]`.
    EscapedNameBuffer { required_bytes: usize },
}

/// Доказанный bool `SaveGoods` либо локальная UB/OS-граница.
#[derive(Debug)]
pub(crate) enum GoodsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(GoodsSaveBlock),
}

/// Caller-owned view пятнадцати goods-container-ов одного `CPlayer`.
pub(crate) struct PlayerGoodsFiledSnapshot<'snapshot> {
    pub(crate) player_id: i32,
    pub(crate) packet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) equipment: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) hand: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) yuan_bao: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) ji_fen: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) bank: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) depot: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) battle_fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction_goods: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction_wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) compose_ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
}

/// Доказанный bool `SaveGoodsFiled` либо вложенная неизвестная граница.
#[derive(Debug)]
pub(crate) enum GoodsFiledSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(GoodsTraversalBlock),
}

#[derive(Debug)]
pub(crate) enum GoodsLoadFailure {
    Database {
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
}

#[derive(Debug)]
pub(crate) enum GoodsLoadBlock {
    NameOutsideWindows1251 {
        row_index: usize,
    },
    GoodsGuid {
        row_index: usize,
        source: uuid::Error,
    },
    ChangedGuid {
        row_index: usize,
        source: getrandom::Error,
    },
    Addon {
        row_index: usize,
        source: GoodsLoadedAddonBlock,
    },
    Insert {
        row_index: usize,
        source: PlayerLoadedGoodsInsertBlock,
    },
}

#[derive(Debug)]
pub(crate) enum GoodsLoadOutcome {
    ReturnedTrue {
        row_count: usize,
        created_count: usize,
        skipped_count: usize,
    },
    ReturnedFalse(GoodsLoadFailure),
    /// Исходный null connection открывал новый ADO connection; внешний
    /// connection-owner ещё не подключён к `TiberiusDbGoods`.
    PendingStandaloneConnection,
    BlockedMissingFact(GoodsLoadBlock),
}

/// Операция исходного `CDBGoods`, породившая log-эквивалент.
#[derive(Clone, Copy, Debug)]
pub(crate) enum DbGoodsOperation {
    DeleteGoods,
    SaveGoodsFiled,
    SaveGoods,
    SaveGoodsProperties,
}

/// Структурированная замена исходных `PrintErr` без SQL и runtime values.
#[derive(Debug)]
pub(crate) struct DbGoodsNotice {
    pub(crate) operation: DbGoodsOperation,
    pub(crate) error: DbGoodsSaveError,
}

/// Причина исходного goods log-эквивалента без SQL и runtime values.
#[derive(Debug)]
pub(crate) enum DbGoodsSaveError {
    Database(DbGoodsDatabaseError),
    MissingConnection,
    NestedDeleteFailed,
    NestedPropertiesFailed,
}

impl fmt::Display for DbGoodsSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World goods DB"),
            Self::NestedDeleteFailed => {
                write!(
                    formatter,
                    "предварительное удаление вещей завершилось ошибкой"
                )
            }
            Self::NestedPropertiesFailed => {
                write!(formatter, "сохранение addon properties завершилось ошибкой")
            }
        }
    }
}

impl Error for DbGoodsSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection | Self::NestedDeleteFailed | Self::NestedPropertiesFailed => {
                None
            }
        }
    }
}

/// Ошибка достигнутой ADO/TDS-границы goods-owner-а.
#[derive(Debug)]
pub(crate) struct DbGoodsDatabaseError(tiberius::error::Error);

impl fmt::Display for DbGoodsDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World goods DB: {}", self.0)
    }
}

impl Error for DbGoodsDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for DbGoodsDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Узкая объектная граница достигнутых функций `CDBGoods`.
pub(crate) trait DbGoodsOwner {
    /// Загружает joined goods rows внутри caller-owned connection.
    async fn load_goods(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> GoodsLoadOutcome;

    /// Удаляет старые строки игрока внутри уже начатой caller-транзакции.
    async fn delete_goods(
        &mut self,
        player_id: i32,
        active_transaction: &mut WorldTdsClient,
    ) -> bool;

    /// Последовательно сохраняет materialized addon properties одного goods ID.
    async fn save_goods_properties(
        &mut self,
        properties: &[GoodsAddonPropertySnapshot],
        row_id: CGuid,
        active_transaction: &mut WorldTdsClient,
    ) -> bool;

    /// Сохраняет основную строку и затем её addon properties в той же транзакции.
    async fn save_goods(
        &mut self,
        snapshot: &GoodsSaveSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
    ) -> GoodsSaveOutcome;

    /// Удаляет прежние строки и обходит 15 player container-ов в исходном порядке.
    async fn save_goods_filed(
        &mut self,
        snapshot: &PlayerGoodsFiledSnapshot<'_>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> GoodsFiledSaveOutcome;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<DbGoodsNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CDBGoods`.
#[derive(Default)]
pub(crate) struct TiberiusDbGoods {
    notices: VecDeque<DbGoodsNotice>,
}

impl DbGoodsOwner for TiberiusDbGoods {
    async fn load_goods(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> GoodsLoadOutcome {
        player.reset_goods_for_db_load();
        let Some(active_transaction) = active_transaction else {
            return GoodsLoadOutcome::PendingStandaloneConnection;
        };

        let mut query = Query::new(
            "SELECT CONVERT(varchar(38),a.GoodsID) AS GoodsID, a.goodsIndex, \
             a.name, a.price, a.amount, a.place, a.position, \
             CONVERT(int,b.type) AS type, b.modifierValue1, b.modifierValue2 \
             FROM player_goods AS a LEFT JOIN extend_properties AS b ON a.id=b.id \
             WHERE playerid=@P1 ORDER BY playerid",
        );
        query.bind(player.get_id());
        let mut rows = match query.query(&mut *active_transaction).await {
            Ok(rows) => rows,
            Err(source) => {
                return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                    row_index: None,
                    source,
                });
            }
        };

        let mut row_index = 0usize;
        let mut created_count = 0usize;
        let mut skipped_count = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    });
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };

            macro_rules! required {
                ($type:ty, $column:literal) => {
                    match row.try_get::<$type, _>($column) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            return GoodsLoadOutcome::ReturnedFalse(
                                GoodsLoadFailure::MissingRequiredValue {
                                    row_index,
                                    column: $column,
                                },
                            );
                        }
                        Err(source) => {
                            return GoodsLoadOutcome::ReturnedFalse(
                                GoodsLoadFailure::Database {
                                    row_index: Some(row_index),
                                    source,
                                },
                            );
                        }
                    }
                };
            }

            let goods_guid_text = required!(&str, "GoodsID");
            let mut goods_guid = match CGuid::from_legacy_text(Some(goods_guid_text)) {
                Ok(guid) => guid,
                Err(source) => {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::GoodsGuid {
                        row_index, source,
                    });
                }
            };
            let mut goods_index = required!(i32, "goodsIndex") as u32;
            let name = required!(&str, "name");
            let (name, _, name_had_errors) = WINDOWS_1251.encode(name);
            if name_had_errors {
                return GoodsLoadOutcome::BlockedMissingFact(
                    GoodsLoadBlock::NameOutsideWindows1251 { row_index },
                );
            }
            let price = required!(i32, "price") as u32;
            let amount = required!(i32, "amount") as u32;
            let place = required!(i32, "place");
            let position = required!(i32, "position") as u32;
            let addon_type = match row.try_get::<i32, _>("type") {
                Ok(value) => value,
                Err(source) => {
                    return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    });
                }
            };
            let addon = if let Some(addon_type) = addon_type {
                Some((
                    addon_type,
                    required!(i32, "modifierValue1"),
                    required!(i32, "modifierValue2"),
                ))
            } else {
                None
            };

            if let Some(&replacement) = changed_goods_indices.get(&goods_index) {
                goods_index = replacement;
                goods_guid = match CGuid::create() {
                    Ok(guid) => guid,
                    Err(source) => {
                        return GoodsLoadOutcome::BlockedMissingFact(
                            GoodsLoadBlock::ChangedGuid { row_index, source },
                        );
                    }
                };
            }

            if let Some(goods) = player.loaded_goods_mut(place, position) {
                if let Some((property_type, first_modifier, second_modifier)) = addon
                    && let Err(source) = goods.apply_loaded_addon(
                        property_type,
                        first_modifier,
                        second_modifier,
                        registry,
                        dakong_addon_types,
                    )
                {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Addon {
                        row_index,
                        source,
                    });
                }
                row_index += 1;
                continue;
            }

            let Some(mut goods) = create_goods_no_probability(registry, goods_index) else {
                skipped_count += 1;
                row_index += 1;
                continue;
            };
            if let Some((property_type, first_modifier, second_modifier)) = addon
                && let Err(source) = goods.apply_loaded_addon(
                    property_type,
                    first_modifier,
                    second_modifier,
                    registry,
                    dakong_addon_types,
                )
            {
                return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Addon {
                    row_index,
                    source,
                });
            }
            goods.set_ex_id(&goods_guid);
            goods.set_name(&name);
            goods.set_price(price);
            goods.set_amount(amount);
            match player.insert_loaded_goods(place, position, goods, registry) {
                Ok(None) => created_count += 1,
                Ok(Some(_)) => skipped_count += 1,
                Err(source) => {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Insert {
                        row_index,
                        source,
                    });
                }
            }
            row_index += 1;
        }

        GoodsLoadOutcome::ReturnedTrue {
            row_count: row_index,
            created_count,
            skipped_count,
        }
    }

    async fn delete_goods(
        &mut self,
        player_id: i32,
        active_transaction: &mut WorldTdsClient,
    ) -> bool {
        let mut query = Query::new("DELETE FROM player_goods WHERE playerID=@P1");
        query.bind(player_id);
        match query.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(DbGoodsNotice {
                    operation: DbGoodsOperation::DeleteGoods,
                    error: DbGoodsSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn save_goods_properties(
        &mut self,
        properties: &[GoodsAddonPropertySnapshot],
        row_id: CGuid,
        active_transaction: &mut WorldTdsClient,
    ) -> bool {
        let row_id = row_id.to_string();
        let mut first_base = 0_i32;
        let mut first_modifier = 0_i32;
        let mut second_base = 0_i32;
        let mut second_modifier = 0_i32;

        for property in properties {
            for value in &property.values {
                match value.id {
                    1 => {
                        first_base = value.base_value;
                        first_modifier = value.modifier;
                    }
                    2 => {
                        second_base = value.base_value;
                        second_modifier = value.modifier;
                    }
                    _ => {}
                }
            }

            let modifier_value_2 = if property.property_type == 0x25 {
                if first_modifier == 0 && first_base == second_base {
                    continue;
                }
                second_base
            } else {
                if property.occur_probability == 10_000
                    && first_modifier == 0
                    && second_modifier == 0
                {
                    continue;
                }
                second_modifier
            };

            let mut query = Query::new(
                "INSERT INTO extend_properties(type,modifierValue1,modifierValue2,id) \
                 VALUES(@P1,@P2,@P3,@P4)",
            );
            query.bind(property.property_type as i32);
            query.bind(first_modifier);
            query.bind(modifier_value_2);
            query.bind(row_id.as_str());
            if let Err(error) = query.execute(&mut *active_transaction).await {
                self.notices.push_back(DbGoodsNotice {
                    operation: DbGoodsOperation::SaveGoodsProperties,
                    error: DbGoodsSaveError::Database(error.into()),
                });
                return false;
            }
        }
        true
    }

    async fn save_goods(
        &mut self,
        snapshot: &GoodsSaveSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
    ) -> GoodsSaveOutcome {
        let row_id = match CGuid::create() {
            Ok(row_id) => row_id,
            Err(error) => {
                return GoodsSaveOutcome::BlockedMissingFact(GoodsSaveBlock::GuidGeneration(error));
            }
        };
        let visible_name = visible_c_string(&snapshot.goods.name);
        let escaped_name = escape_single_quotes(visible_name);
        let escaped_required_bytes = escaped_name.len().saturating_add(1);
        if escaped_required_bytes > 256 {
            return GoodsSaveOutcome::BlockedMissingFact(GoodsSaveBlock::EscapedNameBuffer {
                required_bytes: escaped_required_bytes,
            });
        }
        let sql_required_bytes = legacy_goods_sql_required_bytes(snapshot, row_id, &escaped_name);
        debug_assert!(sql_required_bytes <= 499);

        if let Err(error) =
            insert_goods_row(snapshot, row_id, visible_name, active_transaction).await
        {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoods,
                error: DbGoodsSaveError::Database(error.into()),
            });
            return GoodsSaveOutcome::Failed;
        }

        let properties = match &snapshot.goods.properties {
            GoodsPropertiesSnapshot::Available(properties) => properties,
            GoodsPropertiesSnapshot::MissingBaseProperties => {
                return GoodsSaveOutcome::Failed;
            }
        };
        if !properties.is_empty()
            && !self
                .save_goods_properties(properties, row_id, active_transaction)
                .await
        {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoods,
                error: DbGoodsSaveError::NestedPropertiesFailed,
            });
            return GoodsSaveOutcome::Failed;
        }
        GoodsSaveOutcome::Saved
    }

    async fn save_goods_filed(
        &mut self,
        snapshot: &PlayerGoodsFiledSnapshot<'_>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> GoodsFiledSaveOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoodsFiled,
                error: DbGoodsSaveError::MissingConnection,
            });
            return GoodsFiledSaveOutcome::ReturnedFalse;
        };
        if !self
            .delete_goods(snapshot.player_id, &mut *active_transaction)
            .await
        {
            // WorldServer RVA 0x0011783E: `__snprintf(buf, 4,
            // "CDBGoods::SaveGoodsFiled() :%d", player_id)` обрезал строку и
            // мог оставить её без NUL перед `AddErrorLogText`.
            // BLOCKED_MISSING_FACT: какие байты старый logger читал после
            // первых четырёх, не доказано; typed notice сохраняет сам факт
            // outer-ошибки без воспроизведения чтения за stack-buffer.
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoodsFiled,
                error: DbGoodsSaveError::NestedDeleteFailed,
            });
            return GoodsFiledSaveOutcome::ReturnedFalse;
        }

        let containers = [
            (1_u8, &snapshot.packet),
            (2, &snapshot.equipment),
            (3, &snapshot.hand),
            (4, &snapshot.wallet),
            (5, &snapshot.yuan_bao),
            (6, &snapshot.ji_fen),
            (7, &snapshot.bank),
            (8, &snapshot.depot),
            (9, &snapshot.fairy),
            (10, &snapshot.battle_fairy),
            (11, &snapshot.auction_goods),
            (12, &snapshot.auction_wallet),
            (13, &snapshot.auction),
            (14, &snapshot.ci_qing),
            (15, &snapshot.compose_ci_qing),
        ];
        let mut listener = GoodsListener::new(Some(snapshot.player_id), self, active_transaction);
        for (place, container) in containers {
            listener.set_place(place);
            if let Err(block) = container.traverse(&mut listener).await {
                return GoodsFiledSaveOutcome::BlockedMissingFact(block);
            }
        }
        GoodsFiledSaveOutcome::ReturnedTrue
    }

    fn pop_notice(&mut self) -> Option<DbGoodsNotice> {
        self.notices.pop_front()
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn escape_single_quotes(bytes: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(
        bytes
            .len()
            .saturating_add(bytes.iter().filter(|byte| **byte == b'\'').count()),
    );
    for byte in bytes {
        if *byte == b'\'' {
            escaped.push(b'\'');
        }
        escaped.push(*byte);
    }
    escaped
}

fn legacy_goods_sql_required_bytes(
    snapshot: &GoodsSaveSnapshot<'_>,
    row_id: CGuid,
    escaped_name: &[u8],
) -> usize {
    let mut sql = Vec::with_capacity(256 + escaped_name.len());
    sql.extend_from_slice(b"INSERT INTO player_goods(id,GoodsID,goodsIndex,playerID,name,price,amount,place,position) \t\t\t VALUES('");
    sql.extend_from_slice(row_id.to_string().as_bytes());
    sql.extend_from_slice(b"','");
    sql.extend_from_slice(snapshot.goods.goods_id.to_string().as_bytes());
    sql.extend_from_slice(b"',");
    append_i32(&mut sql, snapshot.goods.base_properties_index as i32);
    sql.push(b',');
    append_i32(&mut sql, snapshot.player_id);
    sql.extend_from_slice(b",N'");
    sql.extend_from_slice(escaped_name);
    sql.extend_from_slice(b"',");
    append_i32(&mut sql, snapshot.goods.price as i32);
    sql.push(b',');
    append_i32(&mut sql, snapshot.goods.amount as i32);
    sql.push(b',');
    append_i32(&mut sql, i32::from(snapshot.place));
    sql.push(b',');
    append_i32(&mut sql, i32::from(snapshot.position));
    sql.push(b')');

    sql.len().saturating_add(1)
}

fn append_i32(target: &mut Vec<u8>, value: i32) {
    target.extend_from_slice(value.to_string().as_bytes());
}

async fn insert_goods_row(
    snapshot: &GoodsSaveSnapshot<'_>,
    row_id: CGuid,
    visible_name: &[u8],
    active_transaction: &mut WorldTdsClient,
) -> Result<(), tiberius::error::Error> {
    let (name, _, _) = WINDOWS_1251.decode(visible_name);
    let mut query = Query::new(
        "INSERT INTO player_goods(id,GoodsID,goodsIndex,playerID,name,price,amount,place,position) \
         VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9)",
    );
    query.bind(row_id.to_string());
    query.bind(snapshot.goods.goods_id.to_string());
    query.bind(snapshot.goods.base_properties_index as i32);
    query.bind(snapshot.player_id);
    query.bind(name.into_owned());
    query.bind(snapshot.goods.price as i32);
    query.bind(snapshot.goods.amount as i32);
    query.bind(snapshot.place);
    query.bind(snapshot.position);
    query.execute(active_transaction).await?;
    Ok(())
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbgoods.cpp



// ============================================================================
// FUNCTION: CDBGoods::LoadGoods
// STATUS: IMPLEMENTED_PARTIAL/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbgoods.cpp:355
// RVA: 0x00117E20
// ADDRESS: 00517e20
// PROTOTYPE: bool __cdecl LoadGoods(CPlayer * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// IMPLEMENTED_OWNER: `DbGoodsOwner::load_goods` выше восстанавливает полный
// caller-connection путь; автономное открытие при null connection ещё pending.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005190ea
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbgoods.cpp:750
// RVA: 0x001190EA
// ADDRESS: 005190ea
// PROTOTYPE: undefined Catch@005190ea()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_005191a4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbgoods.cpp:762
// RVA: 0x001191A4
// ADDRESS: 005191a4
// PROTOTYPE: undefined FUN_005191a4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: WorldServer
