//! Комната аукциона `CAuctionRoom`: owned-состояние MiscServer, добавление
//! товара и полная очистка основных и вторичных списков.
//!
//! Статус владельца: `IMPLEMENTED` для `CAuctionRoom`, `~CAuctionRoom`,
//! `Clear`, `ClearAuctionList`, `ClearSucessedList`, `ClearBackList`,
//! `ClearTimeList`, `ClearOwnerList`, `ClearTypeList`, `ClearDelList` и
//! MiscServer `AddItemToAuctionRoom` с его четырьмя прямыми `Push`-владельцами,
//! `QueryGoodsNodeInfo`, `PushItemToOptList`, `UnityGoods`,
//! `QueryItemFromOwnerList`, `AddByteAuctionSelfToClient`, `ComputePlayerPage`,
//! `IsMatchCondition`, `AddByteAtPageByTime`, `ModifyPlayerSeachCondition`,
//! `AddItemToDelList`, три `DelItemFromAuctionRoom*`, `DoneOptList` и
//! `DoneAuction`, Misc `DelItemFromTimeList`, `DelItemFromOwnerList`,
//! `DelItemFromTypeList`, `ClearRecond`, `PopItemFromGoodsList`,
//! `AddItemToSucessedList`, `AddItemToBackList`, `DoneDelList`,
//! `DoneSucessedGoodsList`, `DoneBackList` и `AI`; остальной аукционный протокол
//! ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально).
//!
//! Исходные `.cpp`: `h:\fengyun\fy_russia\src\public\auctionroom\aucitionroom.cpp`
//! и `e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp`. Точные
//! пары: `MiscServer/miscserver.exe + MiscServer/miscserver.pdb` и
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`. Существенные RVA:
//! MiscServer — конструктор `0x0000B1A0`, деструктор `0x0000AA00`, `Clear`
//! `0x000093D0`, семь частных очисток `0x00007020`, `0x00007090`, `0x00007110`,
//! `0x000085B0`, `0x00008750`, `0x000088A0`, `0x00008940`; GameServer —
//! конструктор `0x00077D70`, деструктор `0x000779F0`, частные очистки
//! `0x00075B70`, `0x00075C20`, `0x00075CD0`, `0x00075D80`, `0x00075E50`,
//! `0x00075F20`, `0x00075FF0`. Добавление MiscServer: `PushItemToGoodsList`
//! `0x00009620`, `PushItemToTimeList` `0x0000ABE0`, `PushItemToOwnerList`
//! `0x0000ACC0`, `PushItemToTypeList` `0x0000AE20` и
//! `AddItemToAuctionRoom` `0x0000B330`; сопоставленный GameServer-вариант —
//! `0x000774C0`, `0x00077CA0` и `0x00077F00`. Достигнутые Misc lookup/opt —
//! `QueryGoodsNodeInfo` `0x00006DB0` и `PushItemToOptList` `0x00009410`;
//! `UnityGoods` `0x00009BF0`; self-list — `QueryItemFromOwnerList`
//! `0x0000AD90` и `AddByteAuctionSelfToClient` `0x0000B010`; page/filter —
//! `ComputePlayerPage` `0x00006E50`, `IsMatchCondition` `0x00008A00` и
//! `AddByteAtPageByTime` `0x00009F80`; изменение player-search —
//! `ModifyPlayerSeachCondition` `0x00009A10`; обработка opt-list —
//! `AddItemToDelList` `0x0000AF00`, `DelItemFromAuctionRoom` `0x0000B4A0`,
//! `DelItemFromAuctionRoomByPreBuy` `0x0000B590`,
//! `DelItemFromAuctionRoomForSucessed` `0x0000B680` и `DoneOptList`
//! `0x0000B770`; time-ticket drain `DoneAuction` `0x0000B830`; удаление из
//! вторичных индексов `DelItemFromTimeList` `0x000084B0`,
//! `DelItemFromOwnerList` `0x00008650`, `DelItemFromTypeList` `0x000087F0`
//! и объединяющий их `ClearRecond` `0x00009520`; перенос primary ownership
//! `PopItemFromGoodsList` `0x00009750`, `AddItemToSucessedList` `0x00009830`
//! и `AddItemToBackList` `0x00009920`; deletion drain `DoneDelList`
//! `0x0000A2B0`; terminal send — `DoneSucessedGoodsList` `0x0000A750` и
//! `DoneBackList` `0x0000A860`; объединяющий `AI` `0x0000B890`. Отдельный Game
//! lookup `0x00075A40` и `UnityGoodsInGS` `0x000776B0` оставлены raw из-за
//! иных primary map и роли.
//!
//! Оба компонента создают одинаковую топологию из восьми ordered-контейнеров и
//! очереди, но основной контейнер различается существенно: MiscServer хранит
//! `CGUID -> CGoodsNode*` и забирает heap-узел во владение, а GameServer хранит
//! `CGUID -> unsigned long` с owner id и принимает не владеющую ссылку на
//! stack-узел handler-а. Поэтому текущая owned-модель относится только к
//! MiscServer и не выдаётся за общую реализацию GameServer. `Clear`
//! экспортирован только у MiscServer, но его порядок совпал с прологом обоих
//! деструкторов: auction, deletion queue, sucessed, back, time, owner, type.
//! `m_mapOptList` и `m_mapPlayerSearchList` в `Clear` не входят и живут до
//! деструктора комнаты.
//!
//! `std::map` заменён `BTreeMap`, `std::deque` — `VecDeque`, а выделенные
//! `vector<CGUID>*` — непосредственно `Vec<CGuid>`: порядок ключей, повторные
//! GUID в одном индексе и порядок очереди сохраняются, освобождение обеспечивает
//! владение Rust. Узлы основных Misc-списков представлены `Box<GoodsNode>`.
//! Узкий `legacy_auction_goods_count` возвращает `len` primary map в прежней
//! 32-битной форме `_Mysize` только достигнутому `ReFlushLog`.
//! Исходные nullable-указатели сознательно не допускаются Rust API. Точный PDB
//! задаёт `m_mapOptList` как `std::map<CGUID, unsigned long>`; он заменён
//! `BTreeMap<CGuid, u32>` и сохраняет operation `1/2/3` до `DoneOptList`, а не
//! указатель на товар. Поля player/opt объявлены в порядке
//! их C++ destruction — сначала player, затем opt; остальные контейнеры к
//! автоматическому `Drop` уже пусты. Остальные STL/CRT/exception cleanup-блоки
//! заменены стандартным владением коллекций.
//!
//! Статус `VERIFIED_DISASSEMBLY` для return-флагов и component layout: точный
//! код Misc подтвердил, что null/invalid/duplicate дают `false`; при двух
//! последних отказах `AddItemToAuctionRoom` удаляет переданный узел и wrapping
//! увеличивает `CGame::m_dwDelNewCount`. Успех сначала вставляет owned-узел,
//! ставит `STATE_AUCTION`, затем добавляет GUID в time, signed owner и type
//! индексы; результаты трёх вторичных helpers игнорируются. `BTreeMap::entry`
//! заменяет только STL allocation plumbing успешного пути. Вызов `GetGame()`
//! заменён явной `&mut u32` границей счётчика, а `nullptr` исключён типом `Box`.
//!
//! `m_btGoodsType` не инициализировался constructor/`Clear`, хотя метод читал
//! его после основной вставки. Безопасный Rust сохраняет уже доказанные main,
//! state, time и owner эффекты, затем возвращает `BLOCKED_MISSING_FACT`, не
//! назначая неизвестный type key. Разобранные exact EXE-инструкции подтвердили
//! флаги возврата и различающийся тип значения GameServer; reverse был
//! остановлен на этих фактах. GameServer raw-вариант оставлен ниже отдельно.
//!
//! Misc `QueryGoodsNodeInfo` отклоняет `GUID_INVALID` и возвращает nullable
//! узел только из primary auction map. `PushItemToOptList` сначала запрещает
//! повторный opt GUID, затем через этот lookup требует нулевой
//! `AuctionInfo::dwBuyerId`. Operation `1` записывает owner ID; остальные
//! значения записывают signed player ID с сохранением 32-битного шаблона.
//! Buyer меняется до вставки пары `GUID -> operation` в opt-map. Достигнутый handler
//! передаёт ту же process-global комнату, поэтому старый `GetGame()` внутри
//! helper-а безопасно выражен `self` без переноса глобального singleton.
//!
//! Статус `VERIFIED_DISASSEMBLY`: exact EXE `0x00409410..0x00409512`
//! подтвердил offsets аргументов и наблюдаемый дефект `xor al, al` перед
//! возвратом. Поэтому helper возвращает `false` и после успешной мутации;
//! вызывающий `0x14ED04` вследствие этого всегда формирует error-response.
//!
//! Misc `UnityGoods` выполняет ordered merge primary GUID и присланного
//! `std::map<CGUID, bool>`, значения которого не читает. Для каждого primary
//! GUID он строит unity-поток `0x15EB04 + map_id + (byte(1), GUID)*`; для первых
//! двадцати отсутствующих на GameServer GUID перед ним ставит отдельный
//! `0x15EB03 + map_id + CGoodsNode::Serialize`. Полностью совпавшие непустые
//! множества не отправляют ничего; пустой primary map всё равно отправляет
//! unity-сообщение только с map ID. Лишний входной GUID заставляет отправить
//! полный primary список. Три проигнорированных `timeGetTime` не влияли на
//! результат и удалены как ненаблюдаемый CRT/WinMM механизм. Построение
//! сообщений отделено от фактического `Send`, но сохраняет исходный порядок:
//! detail по мере merge, итоговый unity последним. Последующие перемещения
//! между Misc-списками и terminal wire восстановлены ниже; отличающийся
//! GameServer-вариант остаётся отдельным raw-owner.
//!
//! `QueryItemFromOwnerList` ищет signed owner ID и копирует GUID в исходном
//! порядке `vector`, но прекращает копирование сразу после сотого элемента.
//! Его `bool` равен наличию хотя бы одного элемента и вызывающим self-list
//! игнорируется. `AddByteAuctionSelfToClient` сначала пишет 32-битное число
//! скопированных GUID, затем для каждого GUID — 32-битный marker `0/1`; после
//! `1` без дополнительной длины следует готовый `CGoodsNode::Serialize`.
//! Статус `VERIFIED_DISASSEMBLY`: exact Misc EXE `0x0040B076`,
//! `0x0040B0D1` и `0x0040B133` вызывают один RVA `0x00010F20`, поэтому count и
//! оба marker занимают ровно четыре байта. Nullable `CMessage*` исключён
//! ссылкой. Ошибка safe-сериализации локально блокирует только ещё не
//! отправленный response; неизвестные байты старого неинициализированного узла
//! не воспроизводятся через `unsafe`.
//!
//! `ComputePlayerPage` считает верхнюю страницу как wrapping
//! `(primary_size + 6) / 7`, хотя фильтр может исключить часть товаров. Для
//! operation `1` page увеличивается и при достижении границы становится
//! `page_count - 1`; пустая primary map поэтому даёт доказанный `u32::MAX`.
//! Operation `2` уменьшает только ненулевую page, остальные значения сбрасывают
//! её в ноль. Отсутствующий player-search узел не создаётся и не меняется.
//!
//! `IsMatchCondition` безусловно исключает товар самого игрока и требует уже
//! существующий `stPlayerOptNode`. Четыре фильтра независимы: пустое имя либо
//! byte-substring до первого NUL, unsigned inclusive low/up level, точный money
//! type и weapon type с wildcard `-1`. Поле `m_lUseSelf` это тело не читает.
//! Неинициализированные goods type/level и отсутствие NUL локализованы как
//! `BLOCKED_MISSING_FACT`, а не получают придуманный результат.
//!
//! `AddByteAtPageByTime` проходит time-ticket по возрастанию ключа и сохраняет
//! повторы GUID внутри `Vec`. Signed-положительный `current_page * 7` пропускает
//! столько подходящих товаров; wrapping high-bit либо ноль начинают с первой
//! записи. Недостигнутая page пишет только один 32-битный ноль. Иначе ответ
//! получает wrapping `primary_size - число просмотренных GUID`, затем literal
//! `7` и до семи пар `long(1) + Serialize`; неподходящие и stale GUID marker не
//! получают. Скопированный `stPlayerOptNode` и STL iterators заменены обычными
//! immutable borrow/итераторами: в достигнутом однопоточном вызове между ними
//! нет мутации комнаты. Nullable message/item исключены ссылкой и `Option`.
//!
//! `ModifyPlayerSeachCondition` использует unsigned player ID самого значения
//! как ключ ordered map. Отсутствующий ключ получает полную копию узла, а у
//! существующего целиком заменяются все 71 DWORD значения, включая page и
//! 256-байтовое имя; ключ map остаётся прежним. `BTreeMap::insert` сохраняет
//! обе ветви, а освобождение прежнего scalar/array-значения ненаблюдаемо.
//!
//! `DoneOptList` дренирует ordered opt-map от минимального GUID и только после
//! вызова выбранного helper-а удаляет текущую запись. Operation `1/2/3`
//! переводит существующий primary-узел из `STATE_AUCTION` в
//! `STATE_UNDO/STATE_SUCESSED/STATE_PRE_BUY`; неизвестная operation не меняет
//! товар, но запись всё равно удаляется. Все три helper-а игнорируют результат
//! `AddItemToDelList` и возвращают `true` уже после state-мутации. Старый deque
//! при первом GUID создаёт один owned vector, а затем дописывает именно logical
//! element zero/front; `VecDeque<Vec<CGuid>>::front_mut` сохраняет этот порядок.
//! Проверка null allocation не имеет восстанавливаемого значения в безопасной
//! Rust-модели: стандартная коллекция владеет памятью, а allocation failure
//! остаётся фатальной границей процесса, как у остальных заменённых STL-узлов.
//!
//! `DoneAuction` сначала полностью выполняет `DoneOptList`, затем один раз
//! читает Unix-секунды и дренирует только начальный ordered-prefix time-map с
//! unsigned ticket `<= now`. Старый `time(NULL)` заменён безопасным
//! `rustix::time::clock_gettime(CLOCK_REALTIME)`; явное сужение `tv_sec` до
//! `u32` сохраняет прежний 32-битный шаблон. Каждый целый GUID-vector сначала
//! переносится в конец deletion deque и лишь затем удаляется его time-key.
//! `mem::take` заменяет передачу владеющего pointer между STL-контейнерами и
//! позволяет сохранить этот порядок без raw pointer; nullable vector в Rust
//! не представлен.
//!
//! Misc `DelItemFromTimeList` и `DelItemFromOwnerList` удаляют только первое
//! совпадение GUID, возвращают факт удаления и уничтожают key вместе с
//! опустевшим vector. `DelItemFromTypeList` также удаляет лишь первое
//! совпадение, но возвращает наличие самого type-vector и сохраняет его key
//! даже пустым. Статус `VERIFIED_DISASSEMBLY`: exact Misc EXE на
//! `0x0040852E/0x004086CE` восстанавливает заранее обнулённый result после
//! полного прохода без совпадения, а `0x0040859B/0x0040873B` возвращает его в
//! `AL`; значит, неоднозначная ветвь обоих первых helpers равна `false`.
//! `ClearRecond` копирует поля живого primary-узла и вызывает helpers строго
//! time -> owner -> type, игнорируя результаты. Если старый `m_btGoodsType`
//! не был инициализирован, безопасный Rust сохраняет уже выполненные time и
//! owner эффекты, затем возвращает локальный `BLOCKED_MISSING_FACT`, не
//! выбирая неизвестный byte type.
//!
//! Misc `PopItemFromGoodsList` отклоняет `GUID_INVALID`, а для живого GUID
//! сначала отдаёт вызывающему тот же `CGoodsNode*`, затем выполняет готовый
//! `ClearRecond` и удаляет только primary key. Rust заменяет пару
//! `bool + CGoodsNode**` на `Result<Option<Box<CGoodsNode>>, _>`: успешный
//! `Option` одновременно доказывает исходный `true` и передаёт то же владение,
//! а blocked type-граница оставляет узел в primary map после уже совершённых
//! time/owner эффектов. `AddItemToSucessedList` и `AddItemToBackList` исключают
//! старый nullable-вход типом `Box`, копируют его GUID и передают узел в
//! соответствующий ordered map. Нормальный ненулевой путь сохраняет legacy
//! `true`. При duplicate destination key исходный `std::map::insert` не
//! вставлял pointer, а caller не освобождал его; достижимость и наблюдаемый
//! результат этой утечки не доказаны. Безопасный compatibility API сохраняет
//! существующий map-узел и возвращает новый `Box` в typed
//! `BLOCKED_MISSING_FACT`, не выбирая уничтожение, замену либо leak.
//!
//! Misc `DoneDelList` полностью дренирует deletion deque от первого GUID
//! первого batch. Отсутствующий primary-узел только удаляет queue-запись.
//! `STATE_AUCTION` переносится в back либо sucessed по `m_bOfferPrice` с
//! предварительным присваиванием нового state; `STATE_SUCESSED` сохраняет
//! state и переносится в sucessed, `STATE_UNDO/STATE_PRE_BUY` — в back. После
//! каждого нормального случая GUID удаляется из начала vector, а опустевший
//! batch — из начала deque. Pointer-equality проверки исчезают только потому,
//! что `PopItemFromGoodsList` возвращает тот самый единственный owned `Box`.
//! Empty `Vec` не имеет старого nullable allocation-состояния и удаляется тем
//! же проходом.
//!
//! Статус `VERIFIED_DISASSEMBLY` для default-state дефекта: exact Misc EXE
//! `0x0040A5BA..0x0040A5CC` вызывает `ClearRecond`, destructor и
//! `operator delete`, после чего сразу сдвигает GUID-vector; primary
//! `map::erase` отсутствует. Достижимость и наблюдаемый результат оставленного
//! dangling pointer не доказаны. Rust выполняет доказанный `ClearRecond` и
//! удаление GUID из queue, но сохраняет primary `Box` живым и возвращает typed
//! `BLOCKED_MISSING_FACT`; продолжение после dangling, придуманное удаление key
//! и `unsafe` не вводятся. Missing goods type останавливает текущий GUID до его
//! удаления из queue, а duplicate terminal key возвращает owned-узел после
//! доказанного удаления queue-записи.
//!
//! `DoneSucessedGoodsList` полностью дренирует ordered sucessed-map от
//! минимального GUID; каждый живой узел сериализуется в отдельный `0x15EB01`
//! и отправляется текущему nullable client без приоритета. `DoneBackList`
//! аналогично строит `0x15EB02`, но за один вызов снимает только один
//! минимальный back-key. Результат старого `Send` игнорировался, поэтому
//! typed send-error не отменяет уничтожение узла и key. Safe-ошибка
//! `Serialize` также локализована как результат уже завершённого удаления:
//! неизвестные старые bytes не отправляются, но terminal-запись не остаётся
//! на повторную обработку. `BTreeMap::remove` одновременно отделяет key и
//! единственный `Box`; немедленный `Drop` заменяет старые destructor/delete,
//! а ненаблюдаемая внутренняя последовательность освобождения STL tree-node и
//! товара не становится отдельным runtime-контрактом. Общий private helper
//! строит только один terminal-пакет; различие полного drain и single-step
//! сохранено двумя самостоятельными методами.
//!
//! Misc `AI` является буквальной однопоточной композицией без условий:
//! `DoneAuction -> DoneDelList -> DoneSucessedGoodsList -> DoneBackList`.
//! Каждая стадия вызывается ровно один раз. Safe-ошибка `DoneDelList` не
//! превращается в новый ранний return: она сохраняется в typed outcome, после
//! чего исходные sucessed и back стадии выполняются в прежних позициях.
//! Результаты terminal-сериализации и отправки остаются в тех же ordered
//! отчётах. Глобальный `GetGame()` этой функции не требовался; nullable client
//! передаётся явной `Option<&dyn MessageSender>` для двух исходящих стадий.

use std::collections::{BTreeMap, VecDeque};

use rustix::time::{ClockId, clock_gettime};

use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};

use super::auctionnode::{CGoodsNode, GoodsNodeSerializeError, GoodsState};
use super::auctionroom::PlayerOptNode;
use super::guid::CGuid;

/// Неопределённая граница старого неинициализированного `m_btGoodsType`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AddAuctionItemMissingGoodsType {
    pub(crate) guid: CGuid,
}

/// Неопределённая type-граница `ClearRecond` после доказанных time/owner эффектов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClearRecondMissingGoodsType {
    pub(crate) guid: CGuid,
}

/// Неопределённая duplicate-граница вставки owned-узла в конечный список.
pub(crate) struct AddTerminalGoodsDuplicate {
    pub(crate) guid: CGuid,
    pub(crate) item: Box<CGoodsNode>,
}

/// Локальные safe-границы полного прохода `DoneDelList`.
pub(crate) enum DoneDelListError {
    MissingGoodsType(ClearRecondMissingGoodsType),
    DuplicateSucessed(AddTerminalGoodsDuplicate),
    DuplicateBack(AddTerminalGoodsDuplicate),
    DefaultStateDanglingPointer { guid: CGuid, state: i32 },
}

/// Результат сериализации и исходно игнорировавшейся отправки terminal-узла.
#[derive(Debug)]
pub(crate) enum TerminalGoodsDelivery {
    /// Safe-сериализация остановилась на недоказанном старом поле или размере.
    SerializeBlocked(GoodsNodeSerializeError),
    /// Пакет построен; результат nullable client/send сохранён для владельца.
    Sent(Result<i32, SendMessageError>),
}

/// Итог одного уже удалённого terminal-узла в исходном GUID-порядке.
#[derive(Debug)]
pub(crate) struct TerminalGoodsDispatch {
    pub(crate) guid: CGuid,
    pub(crate) delivery: TerminalGoodsDelivery,
}

/// Итог одного полного прохода исходного `CAuctionRoom::AI`.
pub(crate) struct AuctionAiOutcome {
    /// Результат полного deletion drain после уже выполненного `DoneAuction`.
    pub(crate) deletion: Result<(), DoneDelListError>,
    /// Все sucessed-узлы, снятые текущим проходом в GUID-порядке.
    pub(crate) sucessed: Vec<TerminalGoodsDispatch>,
    /// Не более одного минимального back-узла текущего прохода.
    pub(crate) back: Option<TerminalGoodsDispatch>,
}

const AUCTION_UNITY_ITEM: i32 = 0x0015_EB03;
const AUCTION_UNITY_INDEX: i32 = 0x0015_EB04;
const AUCTION_SUCCESSED_RESULT: i32 = 0x0015_EB01;
const AUCTION_BACK_RESULT: i32 = 0x0015_EB02;
const AUCTION_UNITY_DETAIL_LIMIT: usize = 20;

/// Построенный в исходном порядке batch `UnityGoods` и локальная safe-граница.
pub(crate) struct UnityGoodsBuild {
    pub(crate) messages: Vec<CMessage>,
    pub(crate) blocked: Option<GoodsNodeSerializeError>,
}

/// Safe-граница построения одной страницы старого аукционного поиска.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionPageBuildError {
    GoodsCountOutsideLegacyRange { length: usize },
    MissingGoodsType,
    MissingLevelLimit,
    LegacyStringWithoutTerminator { field: &'static str },
    Serialize(GoodsNodeSerializeError),
}

/// Owned-состояние исходного `CAuctionRoom`.
///
/// Параметр сохраняет достигнутые constructor/clear call sites. Доменное
/// добавление определено только для доказанного Misc `CGoodsNode`; GameServer
/// с `CGUID -> owner_id` этой специализацией не представлен.
pub(crate) struct CAuctionRoom<GoodsNode> {
    auction_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    time_ticket: BTreeMap<u32, Vec<CGuid>>,
    owner_list: BTreeMap<i32, Vec<CGuid>>,
    type_list: BTreeMap<u8, Vec<CGuid>>,
    del_from_goods_list: VecDeque<Vec<CGuid>>,
    sucessed_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    back_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    player_search_list: BTreeMap<u32, PlayerOptNode>,
    opt_list: BTreeMap<CGuid, u32>,
}

impl<GoodsNode> Default for CAuctionRoom<GoodsNode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<GoodsNode> CAuctionRoom<GoodsNode> {
    /// Создаёт все исходные ordered-контейнеры и очередь пустыми.
    pub(crate) fn new() -> Self {
        Self {
            auction_goods_list: BTreeMap::new(),
            time_ticket: BTreeMap::new(),
            owner_list: BTreeMap::new(),
            type_list: BTreeMap::new(),
            del_from_goods_list: VecDeque::new(),
            sucessed_goods_list: BTreeMap::new(),
            back_goods_list: BTreeMap::new(),
            player_search_list: BTreeMap::new(),
            opt_list: BTreeMap::new(),
        }
    }

    /// Очищает семь owned-контейнеров в точном порядке `Clear`.
    ///
    /// Незавершённые операции и поиски игроков сохраняются: исходный `Clear`
    /// оставлял `m_mapOptList` и `m_mapPlayerSearchList` нетронутыми.
    pub(crate) fn clear(&mut self) {
        self.clear_auction_list();
        self.clear_del_list();
        self.clear_sucessed_list();
        self.clear_back_list();
        self.clear_time_list();
        self.clear_owner_list();
        self.clear_type_list();

        // MiscServer RVA 0x000093D0 не очищает `m_mapOptList`: его пары
        // `CGUID -> unsigned long operation` сохраняются буквально до
        // `DoneOptList` либо destructor комнаты.
    }

    fn clear_auction_list(&mut self) {
        self.auction_goods_list.clear();
    }

    fn clear_sucessed_list(&mut self) {
        self.sucessed_goods_list.clear();
    }

    fn clear_back_list(&mut self) {
        self.back_goods_list.clear();
    }

    fn clear_time_list(&mut self) {
        self.time_ticket.clear();
    }

    fn clear_owner_list(&mut self) {
        self.owner_list.clear();
    }

    fn clear_type_list(&mut self) {
        self.type_list.clear();
    }

    fn clear_del_list(&mut self) {
        self.del_from_goods_list.clear();
    }
}

impl CAuctionRoom<CGoodsNode> {
    /// Добавляет owned MiscServer-узел и обновляет три вторичных индекса.
    ///
    /// Invalid либо уже существующий GUID дают исходный `false`, уничтожают
    /// новый узел и wrapping увеличивают `m_dwDelNewCount`. Ошибка означает
    /// только старое неинициализированное поле типа: main/time/owner эффекты к
    /// этой границе уже совершены, как требует доказанный порядок оригинала.
    pub(crate) fn add_item_to_auction_room(
        &mut self,
        item: Box<CGoodsNode>,
        deleted_new_count: &mut u32,
    ) -> Result<bool, AddAuctionItemMissingGoodsType> {
        let guid = item.guid();
        if guid == CGuid::GUID_INVALID || self.auction_goods_list.contains_key(&guid) {
            *deleted_new_count = deleted_new_count.wrapping_add(1);
            return Ok(false);
        }

        self.auction_goods_list.insert(guid, item);
        let item = self
            .auction_goods_list
            .get_mut(&guid)
            .expect("только что вставленный auction-узел должен существовать");
        item.mark_as_auction();
        let add_ticket = item.add_ticket();
        let owner_id = item.owner_id() as i32;
        let goods_type = item.goods_type();

        self.time_ticket.entry(add_ticket).or_default().push(guid);
        self.owner_list.entry(owner_id).or_default().push(guid);
        let Some(goods_type) = goods_type else {
            // BLOCKED_MISSING_FACT: MiscServer RVA 0x0000B330 читал
            // `m_btGoodsType`, который constructor и `Clear` не задавали:
            // Тип товара берётся из m_btGoodsType и передаётся в PushItemToTypeList.
            // Какой byte наблюдался для такого не-UnSerialize узла, неизвестно.
            return Err(AddAuctionItemMissingGoodsType { guid });
        };
        self.type_list.entry(goods_type).or_default().push(guid);
        Ok(true)
    }

    /// Возвращает Misc-узел primary auction map либо `None`.
    pub(crate) fn query_goods_node_info(&self, guid: CGuid) -> Option<&CGoodsNode> {
        if guid == CGuid::GUID_INVALID {
            return None;
        }
        self.auction_goods_list.get(&guid).map(Box::as_ref)
    }

    /// Возвращает 32-битный `_Mysize` primary map для Misc diagnostic-owner.
    pub(crate) fn legacy_auction_goods_count(&self) -> u32 {
        self.auction_goods_list.len() as u32
    }

    /// Ставит GUID в очередь операции и обновляет buyer исходного узла.
    ///
    /// Возвращаемое значение всегда `false`, включая успешную мутацию: это
    /// доказанный контракт exact Misc EXE, на который опирается handler.
    pub(crate) fn push_item_to_opt_list(
        &mut self,
        guid: CGuid,
        operation: u32,
        player_id: i32,
    ) -> bool {
        if self.opt_list.contains_key(&guid) {
            return false;
        }

        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if item.auction_buyer_id() != 0 {
            return false;
        }

        let buyer_id = if operation == 1 {
            item.owner_id()
        } else {
            player_id as u32
        };
        item.set_auction_buyer_id(buyer_id);
        self.opt_list.insert(guid, operation);

        // VERIFIED_DISASSEMBLY: Misc RVA 0x00009410 выполняет мутации выше,
        // но по 0x00409505 завершает все пути `xor al, al; ret 0x18`.
        false
    }

    /// Добавляет живой GUID в первый batch очереди удаления.
    fn add_item_to_del_list(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }

        if let Some(front) = self.del_from_goods_list.front_mut() {
            front.push(guid);
        } else {
            self.del_from_goods_list.push_back(vec![guid]);
        }
        true
    }

    /// Ставит auction-товар в обычную очередь отмены.
    fn del_item_from_auction_room(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_undo();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Ставит auction-товар в очередь успешной покупки.
    fn del_item_from_auction_room_for_sucessed(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_sucessed();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Ставит auction-товар в очередь предварительной покупки.
    fn del_item_from_auction_room_by_pre_buy(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_pre_buy();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Полностью обрабатывает текущий ordered opt-list от минимального GUID.
    pub(crate) fn done_opt_list(&mut self) {
        while let Some((&guid, &operation)) = self.opt_list.first_key_value() {
            match operation {
                1 => {
                    let _ = self.del_item_from_auction_room(guid);
                }
                2 => {
                    let _ = self.del_item_from_auction_room_for_sucessed(guid);
                }
                3 => {
                    let _ = self.del_item_from_auction_room_by_pre_buy(guid);
                }
                _ => {}
            }
            let _ = self.opt_list.remove(&guid);
        }
    }

    /// Обрабатывает opt-list и переносит истёкший time-prefix в очередь удаления.
    pub(crate) fn done_auction(&mut self) {
        self.done_opt_list();
        let now = clock_gettime(ClockId::Realtime).tv_sec as u32;

        while let Some((&ticket, _)) = self.time_ticket.first_key_value() {
            if ticket > now {
                break;
            }

            let batch = {
                let guids = self
                    .time_ticket
                    .get_mut(&ticket)
                    .expect("минимальный time-ticket должен существовать");
                std::mem::take(guids)
            };
            self.del_from_goods_list.push_back(batch);
            let _ = self.time_ticket.remove(&ticket);
        }
    }

    /// Удаляет первое совпадение из временного индекса и удаляет пустой key.
    fn del_item_from_time_list(&mut self, ticket: u32, guid: CGuid) -> bool {
        let (removed, became_empty) = {
            let Some(items) = self.time_ticket.get_mut(&ticket) else {
                return false;
            };
            let removed = items
                .iter()
                .position(|item_guid| *item_guid == guid)
                .map(|position| items.remove(position))
                .is_some();
            (removed, items.is_empty())
        };

        if became_empty {
            let _ = self.time_ticket.remove(&ticket);
        }
        removed
    }

    /// Удаляет первое совпадение из owner-индекса и удаляет пустой key.
    fn del_item_from_owner_list(&mut self, owner_id: i32, guid: CGuid) -> bool {
        let (removed, became_empty) = {
            let Some(items) = self.owner_list.get_mut(&owner_id) else {
                return false;
            };
            let removed = items
                .iter()
                .position(|item_guid| *item_guid == guid)
                .map(|position| items.remove(position))
                .is_some();
            (removed, items.is_empty())
        };

        if became_empty {
            let _ = self.owner_list.remove(&owner_id);
        }
        removed
    }

    /// Удаляет первое совпадение из type-индекса, сохраняя существующий key.
    fn del_item_from_type_list(&mut self, goods_type: u8, guid: CGuid) -> bool {
        let Some(items) = self.type_list.get_mut(&goods_type) else {
            return false;
        };
        if let Some(position) = items.iter().position(|item_guid| *item_guid == guid) {
            items.remove(position);
        }
        true
    }

    /// Удаляет GUID живого primary-узла из трёх вторичных индексов.
    ///
    /// Ошибка означает неинициализированный старый goods type: time и owner к
    /// этой границе уже обработаны в исходном порядке.
    fn clear_recond(&mut self, guid: CGuid) -> Result<(), ClearRecondMissingGoodsType> {
        let Some(item) = self.auction_goods_list.get(&guid).map(Box::as_ref) else {
            return Ok(());
        };
        let add_ticket = item.add_ticket();
        let owner_id = item.owner_id() as i32;
        let goods_type = item.goods_type();

        let _ = self.del_item_from_time_list(add_ticket, guid);
        let _ = self.del_item_from_owner_list(owner_id, guid);
        let Some(goods_type) = goods_type else {
            // BLOCKED_MISSING_FACT: MiscServer RVA 0x00009520 читал byte
            // `m_btGoodsType` до вызовов, хотя constructor/`Clear` его не
            // задавали. Какой type-key удалялся для такого узла, неизвестно.
            return Err(ClearRecondMissingGoodsType { guid });
        };
        let _ = self.del_item_from_type_list(goods_type, guid);
        Ok(())
    }

    /// Извлекает живой primary-узел после очистки его вторичных индексов.
    ///
    /// Ошибка сохраняет узел в primary map; time/owner эффекты `ClearRecond` к
    /// этой границе уже могли быть выполнены.
    fn pop_item_from_goods_list(
        &mut self,
        guid: CGuid,
    ) -> Result<Option<Box<CGoodsNode>>, ClearRecondMissingGoodsType> {
        if guid == CGuid::GUID_INVALID || !self.auction_goods_list.contains_key(&guid) {
            return Ok(None);
        }

        self.clear_recond(guid)?;
        Ok(self.auction_goods_list.remove(&guid))
    }

    /// Передаёт owned-узел в список успешно завершённых аукционов.
    ///
    /// При duplicate key существующая запись сохраняется, а входной узел
    /// возвращается внутри ошибки: старую утечку safe Rust не воспроизводит.
    fn add_item_to_sucessed_list(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddTerminalGoodsDuplicate> {
        let guid = item.guid();
        match self.sucessed_goods_list.entry(guid) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(item);
                Ok(true)
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                // BLOCKED_MISSING_FACT: MiscServer RVA 0x00009830 игнорировал
                // result `std::map::insert`, возвращал `true` по non-null
                // pointer и оставлял новый узел без доказанного владельца.
                Err(AddTerminalGoodsDuplicate { guid, item })
            }
        }
    }

    /// Передаёт owned-узел в список возврата продавцу.
    ///
    /// При duplicate key существующая запись сохраняется, а входной узел
    /// возвращается внутри ошибки: старую утечку safe Rust не воспроизводит.
    fn add_item_to_back_list(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddTerminalGoodsDuplicate> {
        let guid = item.guid();
        match self.back_goods_list.entry(guid) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(item);
                Ok(true)
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                // BLOCKED_MISSING_FACT: MiscServer RVA 0x00009920 имеет ту же
                // unchecked insert-ветвь, поэтому duplicate ownership не
                // получает придуманного уничтожения, замены либо leak.
                Err(AddTerminalGoodsDuplicate { guid, item })
            }
        }
    }

    /// Полностью дренирует очередь удаления до первой safe-границы.
    ///
    /// Missing goods type сохраняет текущий GUID в начале batch. Ошибки
    /// duplicate/default-state возникают уже после доказанного удаления этого
    /// GUID; содержащийся в ошибке owned-узел не теряется.
    pub(crate) fn done_del_list(&mut self) -> Result<(), DoneDelListError> {
        loop {
            while self.del_from_goods_list.front().is_some_and(Vec::is_empty) {
                let _ = self.del_from_goods_list.pop_front();
            }

            let Some(guid) = self
                .del_from_goods_list
                .front()
                .and_then(|batch| batch.first())
                .copied()
            else {
                return Ok(());
            };

            match self.process_del_guid(guid) {
                Ok(()) => self.consume_first_del_guid(),
                Err(error @ DoneDelListError::MissingGoodsType(_)) => return Err(error),
                Err(error) => {
                    self.consume_first_del_guid();
                    return Err(error);
                }
            }
        }
    }

    /// Полностью дренирует sucessed-map и отправляет по одному `0x15EB01`.
    ///
    /// Любой результат сериализации или отправки относится к уже удалённому
    /// узлу и не останавливает последующие GUID текущего прохода.
    pub(crate) fn done_sucessed_goods_list(
        &mut self,
        sender: Option<&dyn MessageSender>,
    ) -> Vec<TerminalGoodsDispatch> {
        let mut outcomes = Vec::with_capacity(self.sucessed_goods_list.len());
        while let Some((&guid, item)) = self.sucessed_goods_list.first_key_value() {
            let delivery = Self::deliver_terminal_goods(item, AUCTION_SUCCESSED_RESULT, sender);
            let item = self
                .sucessed_goods_list
                .remove(&guid)
                .expect("минимальный sucessed-узел должен существовать");
            drop(item);
            outcomes.push(TerminalGoodsDispatch { guid, delivery });
        }
        outcomes
    }

    /// Обрабатывает только один минимальный back-key через `0x15EB02`.
    ///
    /// Следующие записи намеренно остаются будущему вызову `AI`; результат
    /// сериализации либо отправки не возвращает уже снятый узел в map.
    pub(crate) fn done_back_list(
        &mut self,
        sender: Option<&dyn MessageSender>,
    ) -> Option<TerminalGoodsDispatch> {
        let (&guid, item) = self.back_goods_list.first_key_value()?;
        let delivery = Self::deliver_terminal_goods(item, AUCTION_BACK_RESULT, sender);
        let item = self
            .back_goods_list
            .remove(&guid)
            .expect("минимальный back-узел должен существовать");
        drop(item);
        Some(TerminalGoodsDispatch { guid, delivery })
    }

    fn deliver_terminal_goods(
        item: &CGoodsNode,
        message_type: i32,
        sender: Option<&dyn MessageSender>,
    ) -> TerminalGoodsDelivery {
        let serialized = match item.serialize() {
            Ok(serialized) => serialized,
            Err(error) => return TerminalGoodsDelivery::SerializeBlocked(error),
        };
        let mut message = CMessage::new(message_type);
        message.base_mut().add(&serialized);
        TerminalGoodsDelivery::Sent(message.send(sender, false))
    }

    /// Выполняет четыре стадии аукционной обработки в исходном порядке.
    ///
    /// Safe-ошибка deletion drain сохраняется в отчёте и не отменяет две
    /// следующие независимые terminal-стадии текущего прохода.
    pub(crate) fn ai(&mut self, sender: Option<&dyn MessageSender>) -> AuctionAiOutcome {
        self.done_auction();
        let deletion = self.done_del_list();
        let sucessed = self.done_sucessed_goods_list(sender);
        let back = self.done_back_list(sender);
        AuctionAiOutcome {
            deletion,
            sucessed,
            back,
        }
    }

    fn process_del_guid(&mut self, guid: CGuid) -> Result<(), DoneDelListError> {
        let Some(item) = self.auction_goods_list.get(&guid).map(Box::as_ref) else {
            return Ok(());
        };
        let state = item.goods_state();
        let offer_price = item.offer_price();

        match state {
            GoodsState::AUCTION => {
                let Some(mut item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                if offer_price {
                    item.mark_as_sucessed();
                    let _ = self
                        .add_item_to_sucessed_list(item)
                        .map_err(DoneDelListError::DuplicateSucessed)?;
                } else {
                    item.mark_as_back();
                    let _ = self
                        .add_item_to_back_list(item)
                        .map_err(DoneDelListError::DuplicateBack)?;
                }
            }
            GoodsState::SUCESSED => {
                let Some(item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                let _ = self
                    .add_item_to_sucessed_list(item)
                    .map_err(DoneDelListError::DuplicateSucessed)?;
            }
            GoodsState::UNDO | GoodsState::PRE_BUY => {
                let Some(item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                let _ = self
                    .add_item_to_back_list(item)
                    .map_err(DoneDelListError::DuplicateBack)?;
            }
            unsupported => {
                self.clear_recond(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?;
                // VERIFIED_DISASSEMBLY: Misc RVA 0x0000A2B0 по
                // 0x0040A5BA..0x0040A5CC выполнял
                // `ClearRecond(guid); item->~CGoodsNode(); delete item;`
                // без `m_mapAuctionGoodsList.erase`, после чего удалял GUID из
                // batch. Достижимость dangling pointer неизвестна, поэтому
                // Box остаётся в primary map, а внешний выбор не выдумывается.
                return Err(DoneDelListError::DefaultStateDanglingPointer {
                    guid,
                    state: unsupported.raw(),
                });
            }
        }
        Ok(())
    }

    fn consume_first_del_guid(&mut self) {
        let became_empty = {
            let batch = self
                .del_from_goods_list
                .front_mut()
                .expect("обрабатываемый deletion batch должен существовать");
            batch.remove(0);
            batch.is_empty()
        };
        if became_empty {
            let _ = self.del_from_goods_list.pop_front();
        }
    }

    /// Копирует не более первых ста GUID signed owner-а в переданный список.
    fn query_item_from_owner_list(&self, player_id: i32, items: &mut Vec<CGuid>) -> bool {
        if let Some(owner_items) = self.owner_list.get(&player_id) {
            for guid in owner_items {
                items.push(*guid);
                if items.len() > 99 {
                    return true;
                }
            }
        }
        !items.is_empty()
    }

    /// Дописывает в сообщение self-auction список указанного игрока.
    pub(crate) fn add_byte_auction_self_to_client(
        &self,
        message: &mut CMessage,
        player_id: u32,
    ) -> Result<(), GoodsNodeSerializeError> {
        let mut items = Vec::new();
        let _ = self.query_item_from_owner_list(player_id as i32, &mut items);
        message.base_mut().add_long(items.len() as i32);

        for guid in items {
            let Some(item) = self.query_goods_node_info(guid) else {
                message.base_mut().add_long(0);
                continue;
            };

            message.base_mut().add_long(1);
            let serialized = item.serialize()?;
            message.base_mut().add(&serialized);
        }
        Ok(())
    }

    /// Вставляет либо целиком заменяет поисковое условие по его player ID.
    pub(crate) fn modify_player_seach_condition(&mut self, condition: PlayerOptNode) {
        let player_id = condition.player_id();
        let _ = self.player_search_list.insert(player_id, condition);
    }

    /// Изменяет сохранённую страницу игрока по старой операции `0/1/2`.
    pub(crate) fn compute_player_page(
        &mut self,
        player_id: u32,
        operation: u32,
    ) -> Result<(), AuctionPageBuildError> {
        let goods_count = u32::try_from(self.auction_goods_list.len()).map_err(|_| {
            AuctionPageBuildError::GoodsCountOutsideLegacyRange {
                length: self.auction_goods_list.len(),
            }
        })?;
        let page_count = goods_count.wrapping_add(6) / 7;
        let Some(condition) = self.player_search_list.get_mut(&player_id) else {
            return Ok(());
        };

        match operation {
            1 => {
                let next_page = condition.current_page().wrapping_add(1);
                condition.set_current_page(if page_count <= next_page {
                    page_count.wrapping_sub(1)
                } else {
                    next_page
                });
            }
            2 if condition.current_page() != 0 => {
                condition.set_current_page(condition.current_page() - 1);
            }
            2 => {}
            _ => condition.set_current_page(0),
        }
        Ok(())
    }

    /// Дописывает страницу временного индекса в исходном wire-порядке.
    pub(crate) fn add_byte_at_page_by_time(
        &self,
        player_id: u32,
        message: &mut CMessage,
    ) -> Result<(), AuctionPageBuildError> {
        let Some(condition) = self.player_search_list.get(&player_id) else {
            return Ok(());
        };
        let goods_count = u32::try_from(self.auction_goods_list.len()).map_err(|_| {
            AuctionPageBuildError::GoodsCountOutsideLegacyRange {
                length: self.auction_goods_list.len(),
            }
        })?;

        let mut start_ticket = self.time_ticket.keys().next().copied();
        let mut skip_in_start = 0usize;
        let mut visited = 0u32;
        let mut remaining_to_skip = condition.current_page().wrapping_mul(7) as i32;

        if remaining_to_skip > 0 {
            start_ticket = None;
            'tickets: for (ticket, guids) in &self.time_ticket {
                let mut visited_in_ticket = 0usize;
                for guid in guids {
                    visited_in_ticket += 1;
                    visited = visited.wrapping_add(1);
                    if self.is_match_condition(player_id, condition, *guid)? {
                        remaining_to_skip -= 1;
                        if remaining_to_skip == 0 {
                            start_ticket = Some(*ticket);
                            skip_in_start = visited_in_ticket;
                            break 'tickets;
                        }
                    }
                }
            }
        }

        let Some(start_ticket) = start_ticket else {
            message.base_mut().add_long(0);
            return Ok(());
        };

        message
            .base_mut()
            .add_ulong(goods_count.wrapping_sub(visited));
        message.base_mut().add_long(7);

        let mut slots = 7u32;
        let mut first_ticket = true;
        for (_, guids) in self.time_ticket.range(start_ticket..) {
            let skip = if first_ticket { skip_in_start } else { 0 };
            first_ticket = false;
            for guid in guids.iter().skip(skip) {
                if slots == 0 {
                    return Ok(());
                }
                if !self.is_match_condition(player_id, condition, *guid)? {
                    continue;
                }

                let item = self
                    .query_goods_node_info(*guid)
                    .expect("успешный IsMatchCondition гарантирует живой auction-узел");
                slots -= 1;
                message.base_mut().add_long(1);
                let serialized = item.serialize().map_err(AuctionPageBuildError::Serialize)?;
                message.base_mut().add(&serialized);
            }
        }
        Ok(())
    }

    fn is_match_condition(
        &self,
        player_id: u32,
        condition: &PlayerOptNode,
        guid: CGuid,
    ) -> Result<bool, AuctionPageBuildError> {
        let Some(item) = self.query_goods_node_info(guid) else {
            return Ok(false);
        };
        if player_id == item.owner_id() {
            return Ok(false);
        }

        let condition_name =
            legacy_name(condition.goods_name(), "stPlayerOptNode::m_strGoodsName")?;
        let name_matches = if condition_name.is_empty() {
            true
        } else {
            let item_name = legacy_name(item.goods_name(), "CGoodsNode::m_strGoodsName")?;
            item_name
                .windows(condition_name.len())
                .any(|window| window == condition_name)
        };
        let level = item
            .level_limit()
            .ok_or(AuctionPageBuildError::MissingLevelLimit)?;
        let goods_type = item
            .goods_type()
            .ok_or(AuctionPageBuildError::MissingGoodsType)?;

        let level_matches =
            (condition.low_level() as u32) <= level && level <= condition.up_level() as u32;
        let money_matches = u32::from(item.money_type()) == condition.money_type() as u32;
        let weapon_matches = condition.weapon_type() == -1
            || u32::from(goods_type) == condition.weapon_type() as u32;
        Ok(name_matches && level_matches && money_matches && weapon_matches)
    }

    /// Строит исходный ordered batch синхронизации одного GameServer map.
    pub(crate) fn unity_goods(
        &self,
        existing: &BTreeMap<CGuid, bool>,
        map_id: u8,
    ) -> UnityGoodsBuild {
        let mut unity_message = CMessage::new(AUCTION_UNITY_INDEX);
        unity_message.base_mut().add_byte(map_id);

        let mut messages = Vec::new();
        let mut detail_slots = AUCTION_UNITY_DETAIL_LIMIT;
        let mut need_send = false;

        if !self.auction_goods_list.is_empty() {
            let mut existing_iter = existing.keys().peekable();
            let mut primary_iter = self.auction_goods_list.iter().peekable();

            while existing_iter.peek().is_some() {
                let Some((primary_guid, primary_item)) = primary_iter.peek().copied() else {
                    break;
                };
                let existing_guid = *existing_iter
                    .peek()
                    .expect("цикл проверил существующий входной GUID");

                match existing_guid.cmp(primary_guid) {
                    std::cmp::Ordering::Equal => {
                        Self::append_unity_guid(&mut unity_message, *primary_guid);
                        existing_iter.next();
                        primary_iter.next();
                    }
                    std::cmp::Ordering::Less => {
                        need_send = true;
                        existing_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        Self::append_unity_guid(&mut unity_message, *primary_guid);
                        if let Err(error) = Self::append_unity_detail(
                            &mut messages,
                            primary_item,
                            map_id,
                            &mut detail_slots,
                            &mut need_send,
                        ) {
                            return UnityGoodsBuild {
                                messages,
                                blocked: Some(error),
                            };
                        }
                        primary_iter.next();
                    }
                }
            }

            for (primary_guid, primary_item) in primary_iter {
                Self::append_unity_guid(&mut unity_message, *primary_guid);
                if let Err(error) = Self::append_unity_detail(
                    &mut messages,
                    primary_item,
                    map_id,
                    &mut detail_slots,
                    &mut need_send,
                ) {
                    return UnityGoodsBuild {
                        messages,
                        blocked: Some(error),
                    };
                }
            }

            if !need_send && existing_iter.peek().is_none() {
                return UnityGoodsBuild {
                    messages,
                    blocked: None,
                };
            }
        }

        messages.push(unity_message);
        UnityGoodsBuild {
            messages,
            blocked: None,
        }
    }

    fn append_unity_guid(message: &mut CMessage, guid: CGuid) {
        message.base_mut().add_byte(1);
        message.base_mut().add_guid(guid);
    }

    fn append_unity_detail(
        messages: &mut Vec<CMessage>,
        item: &CGoodsNode,
        map_id: u8,
        detail_slots: &mut usize,
        need_send: &mut bool,
    ) -> Result<(), GoodsNodeSerializeError> {
        if *detail_slots == 0 {
            return Ok(());
        }

        *detail_slots -= 1;
        *need_send = true;
        let serialized = item.serialize()?;
        let mut detail = CMessage::new(AUCTION_UNITY_ITEM);
        detail.base_mut().add_byte(map_id);
        detail.base_mut().add(&serialized);
        messages.push(detail);
        Ok(())
    }
}

fn legacy_name<'value>(
    value: &'value [u8; 256],
    field: &'static str,
) -> Result<&'value [u8], AuctionPageBuildError> {
    let terminator = value
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(AuctionPageBuildError::LegacyStringWithoutTerminator { field })?;
    Ok(&value[..terminator])
}

impl<GoodsNode> Drop for CAuctionRoom<GoodsNode> {
    fn drop(&mut self) {
        self.clear();
    }
}

// COMPONENT_VARIANT_BEGIN: MiscServer
// Точная пара: MiscServer/miscserver.exe + MiscServer/miscserver.pdb
// SHA-256 EXE: F4426942465E6E9D1397EEF7A977B87D0D8C5B12957832770F57656F998AED65
// SHA-256 PDB: ED5F482DADB3E8B050B37F9911067479D297C5B6D33C1EA2CE99C9CD0FC11FA7
// Исходный владелец PDB: h:\fengyun\fy_russia\src\public\auctionroom\aucitionroom.cpp


// COMPONENT_VARIANT_END: MiscServer

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp

// ============================================================================
// FUNCTION: CAuctionRoom::QueryGoodsNodeInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:259
// RVA: 0x00075A40
// ADDRESS: 00475a40
// PROTOTYPE: CGoodsNode * __thiscall QueryGoodsNodeInfo(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::QueryItemNumFromOwerList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:1160
// RVA: 0x00075AE0
// ADDRESS: 00475ae0
// PROTOTYPE: long __thiscall QueryItemNumFromOwerList(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::DelItemFromOwnerList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:1073
// RVA: 0x000770E0
// ADDRESS: 004770e0
// PROTOTYPE: bool __thiscall DelItemFromOwnerList(int param_1, CGUID param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::ClearRecond
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:730
// RVA: 0x00077430
// ADDRESS: 00477430
// PROTOTYPE: void __thiscall ClearRecond(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::PushItemToGoodsList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:791
// RVA: 0x000774C0
// ADDRESS: 004774c0
// PROTOTYPE: bool __thiscall PushItemToGoodsList(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::UnityGoodsInGS
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:438
// RVA: 0x000776B0
// ADDRESS: 004776b0
// PROTOTYPE: void __thiscall UnityGoodsInGS(map<CGUID,bool,std::less<CGUID>,std::allocator<std::pair<CGUID_const_,bool>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::CountGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:307
// RVA: 0x00077BD0
// ADDRESS: 00477bd0
// PROTOTYPE: void __thiscall CountGoods(vector<CGUID,std::allocator<CGUID>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::PushItemToOwnerList
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:1030
// RVA: 0x00077CA0
// ADDRESS: 00477ca0
// PROTOTYPE: bool __thiscall PushItemToOwnerList(int param_1, CGUID param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAuctionRoom::AddItemToAuctionRoom
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\public\auctionroom\aucitionroom.cpp:110
// RVA: 0x00077F00
// ADDRESS: 00477f00
// PROTOTYPE: bool __thiscall AddItemToAuctionRoom(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
