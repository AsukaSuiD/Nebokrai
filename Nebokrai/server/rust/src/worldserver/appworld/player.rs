//! Владелец игрока исторического `WorldServer`.
//!
//! Статус `CPlayer::GetAccount` RVA `0x00002F90`, `CPlayer::SaveData` RVA
//! `0x0005B4E0`, `CPlayer::CheckGoodsInPacket` RVA `0x0005BA90`, inherited
//! `GetName`, reached `ProcessPlayerDataQueue`, `CPlayer::ChangeCountry` RVA
//! `0x0005EA30`
//! accessors для level/friends и inherited `CShape::SetState`,
//! process-wide `CPlayer::GetNetExID` inline-path в `CUnion::ApplyForJoin`
//! `0x004C2C51..0x004C2C5E`,
//! достигнутого base-подобъекта `CMoveShape`, поля `m_bGetFactionData` в
//! `CPlayer::CPlayer` RVA `0x0005EB10` и `CPlayer::~CPlayer` RVA
//! `0x0005F020` — `IMPLEMENTED`; остальной корпус ниже остаётся
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.h:236` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp`.
//!
//! Старый getter выбирал inline-буфер либо heap-pointer MSVC `std::string`
//! `m_BaseProperty.strAccount` и возвращал `char const*`. Owned `Vec<u8>`
//! заменяет только внутреннее хранение строки; Rust API возвращает заимствованный
//! byte-slice и не публикует raw pointer. Завершающий ноль не является частью
//! `std::string`: когда конкретный consumer использует значение как C-строку,
//! его владелец отдельно сохраняет усечение по первому нулю и добавляет terminator.
//!
//! Исходный `CPlayer::CPlayer` первым вызывает `CMoveShape::CMoveShape`, а
//! затем создаёт множество inventory/wallet и других собственных владельцев.
//! Rust хранит достигнутый `CMoveShape` отдельным base-полем и делегирует ему
//! унаследованные ID/region API без вторых `id` или `region_id`. Точный PDB
//! задаёт размер старого `CPlayer` `0x9B8`, а raw-конструктор подтверждает
//! единственный base-вызов `CMoveShape` по offset `0`. `CPlayer` по-прежнему
//! не объявлен полным: исходный порядок содержит недостигнутые поля. Для
//! достигнутого `CloneMapPlayer` добавлен только
//! `with_clone_decode_constructor_state`: он создаёт все уже материализованные
//! base/container/string/collection owner-ы, ставит type `400`, два default
//! `dwFosterNum/dwHatcherNum = 1` и точные container volume. PDB-поля
//! `m_btCountry +0x844` и `m_lContribute +0x848` raw constructor не назначает;
//! до обязательного полного decoder-а они представлены `None`, а encoder
//! останавливает локальную safe-границу вместо выдуманного нуля. Rust layout
//! не объявляется копией старого ABI. Полный raw-конструктор сохранён ради
//! незаменённой семантики; остальные поля `CPlayer` не моделируются до
//! достижения их владельцев.
//!
//! Полная PDB-запись `CPlayer` type `0xC8FE` задаёт размер `0x9B8`, а член
//! `m_bGetFactionData` — как `T_BOOL08` (`0x30`) по offset `+0x868`; следующий
//! signed `m_lFactionID` начинается по `+0x86C`. Конструктор присваивает флагу
//! `false`. `UpdateFactionInfo` сбрасывает его при faction ID `0`, offline-load
//! также сбрасывает, а organizing callbacks выставляют `true` только после
//! отправки полного faction snapshot и снова снимают при выходе. Partial Rust
//! owner хранит этот reached-state как обычный `bool`; методы чтения/записи
//! заменяют прямой доступ к полю, не объявляя Rust layout старым ABI и не
//! придумывая узкий constructor в обход остальных полей. Exact consumer
//! `CFaction::UpdateMemberInfoToClient` проверяет `byte ptr [player+0x868]` в
//! диапазоне `0x004BA82D..0x004BA834`, поэтому offset дополнительно имеет
//! статус `VERIFIED_DISASSEMBLY`.
//!
//! `GetNetExID` не читает player: он pre-increment-ит process-static signed
//! DWORD по VA `0x006BAD98` с x86 wrapping и возвращает новое значение. В
//! `ApplyForJoin` этот эффект происходит после online lookup, но до проверки
//! лимита union, поэтому даже отказ по лимиту расходует ID. `AtomicI32`
//! устраняет исходную data race, сохраняя process lifetime и 32-битный шаблон.
//!
//! `CFaction::Demise` дважды читает `m_bFactionWarOperator` по PDB-offset
//! `CPlayer+0x8ED`; exact диапазон `0x004BFF25..0x004BFF39` подтверждает оба
//! сравнения именно с `true`. Raw constructor назначает `false`, поэтому
//! reached-state хранится отдельным Rust `bool`, не расширяя это до заявления
//! о полном layout `CPlayer`.
//!
//! `ChangeCountry` exact `0x0045EA30..0x0045EA81` сначала сравнивает unsigned
//! byte `m_btCountry +0x844`, затем требует signed `m_lFactionID +0x86C == 0`,
//! проверяет новый byte через `CCountryHandler::GetCountry`, пишет только
//! `m_btCountry` и возвращает новый country как unsigned integer. Ошибки
//! соответственно `-1`, `-3`, `-5`; дополнительных DB/faction side effects
//! нет. Rust получает результат проверки country-owner явным аргументом,
//! заменяя только process-singleton lookup. `Option<u8>` остаётся safe-
//! проекцией ещё не декодированного partial player-state, а не частью старого
//! ABI.
//!
//! Достигнутый через `CGame::ResetHonorElimilateInfo` reset напрямую меняет
//! три DWORD `tagBaseProperty`: day обнуляется безусловно, week только при
//! `mask & 2`, month только при `mask & 4`; накопительный total не меняется.
//! Rust пишет уже подтверждённые wire-offsets вместо воспроизведения старого
//! object-layout.
//! `CHonorRanks::PushToRanks` читает из того же base-owner уровень,
//! occupation и appellation ID; узкие getters публикуют значения без копии
//! всего `tagBaseProperty` и без объявления Rust layout старым ABI.
//!
//! Деструктор сначала вызывает virtual slot `+0x24` у шестнадцати container-
//! подобъектов. Точный PDB исправляет ошибочную первоначальную классификацию:
//! у `CContainer` slot `+0x14` — `Clear(void *)`, а `+0x24` — `Release()`.
//! Exact `0x0045F047..0x0045F133` подтверждает receiver offsets `+0x80`,
//! `+0x154`, `+0xE0`, `+0x3A0`, `+0x414`, `+0x184`, `+0x488`, повторно
//! `+0x184`, `+0x1AC`, `+0x1D4`, `+0x228`, `+0x1FC`, `+0x2A0`, `+0x328`,
//! `+0x4B0` и `+0x524`. Они точно совпадают с PDB-полями `m_cHand`,
//! `m_cEquipment`, `m_cPacket`, двумя auction-container, `m_cWallet`,
//! `m_cAuctionWallet`, повторным `m_cWallet`, `m_cYuanBao`, `m_cJiFen`,
//! `m_cDepot`, `m_cBank`, `m_cFairy`, `m_cBF`, `m_cCiQing` и
//! `m_cComposeCiQing`.
//!
//! Связанные `Release` удаляют owned `CGoods`, очищают собственные map/vector
//! storage, сбрасывают owner/lock state и список внешних listeners; callbacks
//! `OnObjectRemoved` они не вызывают. Amount/equipment containers после reset
//! регистрируют только собственный embedded listener, который затем исчезает
//! вместе с container. Повторный `m_cWallet::Release` наблюдаемого эффекта не
//! добавляет: первый `CGoodsFactory::GarbageCollect` virtual-удаляет gold-coins
//! и зануляет pointer, второй видит `nullptr`. После container-ов старый owner
//! освобождает byte-buffer и только STL-строки/коллекции; `CGoods::~CGoods`
//! также очищает лишь собственные addon/string/base-shape данные. Поэтому
//! структурное уничтожение Rust-полей является совместимой заменой: каждый
//! достигнутый owner освобождается своим `Drop`, отдельный callback или ручной
//! `Drop for CPlayer` не нужен. Все пятнадцать container-полей теперь являются
//! отдельными Rust-owner-ами; повторный wallet-release не требует второго
//! Rust-действия, поскольку первый уже оставляет null slot. Полный raw-
//! деструктор и его STL/EH noise удалены.
//!
//! `CheckGoodsInPacket` создаёт точный `CSeekGoodsListener`, назначает target
//! через original-name index, обходит `m_cPacket`, затем для каждого GUID снова
//! вызывает у packet-а `Find(700, GUID)`. Эта вторая lookup-фаза существенна:
//! traversal видит locked-товары, а `Find` их отбрасывает. Exact
//! `0x0045BB40..0x0045BB47` складывает unsigned amount инструкцией `ADD EBP,EAX`;
//! Rust сохраняет 32-битное переполнение через `i32::wrapping_add`. Type `700`
//! не переносится, потому что safe packet API уже возвращает только `CGoods`.
//! Null `char*` выражен `Option<&CStr>` и по-прежнему возвращает `0` до
//! создания listener-а. MSVC vector/RTTI/SEH заменены `Vec`, enum и `Drop`.

//! Frozen DB-проекция теперь также `IMPLEMENTED`. Она читает byte-exact
//! `tagBaseProperty[0x194]`, shape, JJC, skills/friends/things/quests и все
//! пятнадцать container-owner-ов, а затем строит полные
//! `PlayerCreationSnapshot`/`PlayerSaveSnapshot` из одного `CPlayer`. Exact PDB
//! подтвердил offsets scalar-полей, порядок equipment `0,1,3,4,2,9,10,12,13,14,15`
//! и `GAP_WEAPON_LEVEL = 0x30`. Временные массивы принадлежат
//! `PlayerDbProjection`, поэтому DB-await не заимствует stack-temporary.
//! Неинициализированные country/contribute, отрицательная/короткая variable-
//! data, embedded NUL friend-name и недоказанные goods-границы остаются
//! локальными typed `BLOCKED_MISSING_FACT`, а не получают значения по умолчанию.

//! PDB задаёт `SaveData` как public `bool`-метод с единственным connection
//! `cn`, RVA `0x0005B4E0`, длина `0xA0`; исходный владелец —
//! `player.cpp:187`. Согласованный raw получает singleton `CGame`, читает его
//! `m_pRsPlayer` и один раз вызывает `CRsPlayer::SavePlayer(this, cn)`, после
//! чего возвращает вложенный bool без дополнительной ветви. PDB подтверждает
//! `CGame::m_pRsPlayer: CRsPlayer*` по `+0x138`. Собственного null-connection
//! check, catch, лога, begin, commit или rollback у `CPlayer::SaveData` нет.
//! Неоднозначности для машинного кода не осталось, поэтому дополнительный
//! reverse не выполнялся.
//!
//! Rust передаёт `RsPlayerOwner`, JJc/goods owners и уже собранный
//! `PlayerSaveSnapshot` явно вместо process-global `GetGame` и чтения ещё не
//! материализованного полного layout `CPlayer`. Живая `&self` исключает
//! недоказанный вызов member-функции через null player, а caller обязан собрать
//! snapshot именно из этой же player-сущности. Connection остаётся `Option`:
//! `None` доходит до `CRsPlayer::SavePlayer` и даёт исходный `false`.
//! `PlayerSaveOutcome` расширяет старый bool только локальной goods-
//! неизвестностью, которую безопасный Rust не вправе назначить `true` или
//! `false`. AddRef/Release COM-копии заменены обычным reborrow; raw wrapper и
//! compiler cleanup удалены.
//!
//! Три собственных подслоя будущего clone-codec также имеют статус
//! `IMPLEMENTED`: `AddByteCiQing/DeByteCiQing` RVA
//! `0x0005BB80/0x0005D9E0`, `AddByteArrayLeiTing/DecodeByteArrayLeiTing` RVA
//! `0x0005B690/0x0005DE40` и
//! `AddQuestDataToByteArray/DecordQuestDataFromByteArray` RVA
//! `0x0005BC10/0x0005E970`. `BTreeSet` сохраняет unsigned порядок tattoo ID,
//! `VecDeque` — порядок восьмибайтовых `tagThing`, а `BTreeMap` — unsigned
//! порядок quest-key при отдельном mapped `wQuestID/byComplete`.
//!
//! Container-сегмент `CPlayer::AddToByteArray/DecordFromByteArray` RVA
//! `0x0005BDA0/0x0005F520` также `IMPLEMENTED` между готовым LeiTing-prefix и
//! следующим variable-data suffix. Exact PDB type `0xC8FE` задаёт все
//! пятнадцать nominal полей и offsets: `m_cHand +0x80`, `m_cPacket +0xE0`,
//! `m_cEquipment +0x154`, `m_cWallet +0x184`, `m_cYuanBao +0x1AC`,
//! `m_cJiFen +0x1D4`, `m_cBank +0x1FC`, `m_cDepot +0x228`, `m_cFairy +0x2A0`,
//! `m_cBF +0x328`, `m_cAuctionGoodsContainer +0x3A0`,
//! `m_cAuctionContainer +0x414`, `m_cAuctionWallet +0x488`, `m_cCiQing +0x4B0`
//! и `m_cComposeCiQing +0x524`; `m_strDepotPassword` — отдельный string-owner
//! по `+0x800`.
//!
//! Encoder вызывает container-ы не в layout-order, а в точном wire-order:
//! hand, equipment, packet, два auction volume, wallet, auction wallet,
//! yuanbao, jifen, depot-password, bank, depot, fairy, battle-fairy, ciqing и
//! compose-ciqing. Каждый получает literal `include_child=true`; bool-results
//! игнорируются, а общий player-result остаётся будущей обязанностью полного
//! метода. Fairy encoder сохраняет свой наблюдаемый reset пяти hatch-time.
//!
//! Decoder перед каждым owner-ом выполняет исходный virtual `Release`, затем
//! назначает только доказанные limit/volume: hand `1`, packet `8*0x0C`, auction
//! goods `0x12`, auction container `2`, depot `0xA1`, fairy `0x0E`, battle-
//! fairy `0x11`, ciqing `8`, compose-ciqing `3`. Depot-password читается между
//! jifen и bank через legacy buffer `0x6C`. Повторный release внутри setter-а
//! или decoder-а не свёрнут: порядок и partial effects сохранены буквально.
//! Safe ошибка останавливает только конкретный безразмерный overread после уже
//! выполненных releases, volume/limit mutations и decoded records.
//!
//! Первые пять LeiTing scalar живут внутри byte-exact первых `0x194` байт
//! PDB-структуры `tagBaseProperty`; account/title остаются следующими двумя
//! `std::string` за этой wire-частью. Rust хранит их раздельно и не объявляет
//! собственный layout копией `tagBaseProperty`. Exact
//! `0x0045D9E0..0x0045DA5A` закрыл потерянный raw-факт decoder-а CiQing:
//! после каждого cursor `+4` значение загружается из source как `u32` и
//! вставляется в set; count также обрабатывается как unsigned. После ответа
//! reverse прекращён.
//!
//! Safe slice/cursor сохраняет исходную раннюю очистку коллекций и уже
//! выполненные scalar/entry-мутации. Короткий source останавливает только
//! конкретную границу с `BLOCKED_MISSING_FACT`: старые helpers длину source
//! не получали, а воспроизводить overread через `unsafe` запрещено. Количество
//! Rust-элементов вне диапазона 32-битного `_Mysize` также возвращается как
//! typed ошибка до записи соответствующего count и элементов.
//!
//! `AddOrgSysToByteArray` RVA `0x0005B870` также `IMPLEMENTED`. Exact PDB
//! сообщает для него `VirtualBaseOffset 0xA4`, поэтому потерянный virtual-call
//! в хвосте `AddToByteArray` точно является этим owner-ом. Перед первой записью
//! он по-прежнему выполняет `COrganizingCtrl::SetPlayerOrganizing`; process-
//! global singleton заменён явным `PlayerOrganizingUpdater`, а `CGame` создаёт
//! его из concrete `COrganizingCtrl` и reached region-map projection. Typed
//! ошибка сохраняет уже выполненные мутации до локальной неизвестности. При
//! `faction_id <= 0` wire заканчивается единственным signed ID.
//!
//! Положительный faction ID открывает точный порядок logo, `u16` level,
//! experience, signed force, расширенного до `u32` contribute-флага, двух
//! ANSI C-строк, трёх signed ID, двух signed-order set и списка регионов.
//! `tagOwnedReg` PDB задаёт поля `long +0` и `unsigned short +4`, но исходник
//! копирует все восемь байт; Rust поэтому хранит полный wire-record вместе с
//! наблюдаемыми двумя padding-байтами. Соседний GameServer
//! `DecordOrgSysFromByteArray` подтверждает ту же ширину и порядок. STL-tree/
//! list traversal заменены `BTreeSet<i32>` и `VecDeque` без изменения порядка.
//!
//! Suffix тех же `AddToByteArray/DecordFromByteArray` от variable-data до
//! `m_strSessionID` теперь также `IMPLEMENTED`. PDB задаёт signed variable
//! count/length по `+0x838/+0x840`, `m_lSilienceTime +0x858`, unsigned
//! murderer-time `+0x860`, signed fight-state `+0x864`, pet-vector `+0x900`,
//! carriage `+0x910`, fixed session buffer `[0x40] +0x950`, JJC bool `+0x990`
//! и шестнадцатибайтовый `tagPlayerJJcData +0x992`. Variable pointer остаётся
//! отдельным `Option<Vec<u8>>`, чтобы не смешивать исходные null/non-null
//! состояния; declared length сохраняется отдельно и может оставить partial
//! state после безопасной ошибки.
//!
//! Потерянный raw bool установлен точечно: exact инструкции encoder-а
//! `0x0045BFEF` и decoder-а `0x0045FB3C` читают/пишут `CPlayer+0x7C`, а PDB
//! называет это inherited `CMoveShape::m_bIsGod`. После ответа disassembly
//! прекращён. Decoder буквально потребляет входной fight-state DWORD, но
//! независимо от него назначает live `m_lFightStateCount = 2`. Отрицательный
//! pet count сохранил локальный `BLOCKED_MISSING_FACT`: старый цикл с условием
//! `count != 0` уходил в signed overflow/overread, поэтому безопасный Rust не
//! назначает ему придуманное завершение. Pet/carriage strings ограничены
//! исходными scratch-буферами `0x94`, session ID — фиксированными `0x40`.
//!
//! Decoder по-прежнему очищает pet-vector, carriage original-name/hp и recreate
//! flag до base decode, но не очищает carriage-script, что сохраняет исходный
//! partial путь при `include_child=false`. Organization block присутствует
//! только в WorldServer encoder-варианте. Обязательный завершающий
//! `UpdateProperty` не входит в suffix и реализован отдельной следующей
//! owner-границей.
//!
//! `UpdateProperty` RVA `0x0005AFC0` теперь имеет статус
//! `VERIFIED_DISASSEMBLY, IMPLEMENTED`, поэтому полные
//! `AddToByteArray/DecordFromByteArray` также замкнуты. Exact PDB задаёт все
//! читаемые offsets `tagBaseProperty`, записываемые offsets `tagProperty` и
//! девять `float[3]` из `CGlobeSetup::tagSetup`. Explicit
//! `PlayerPropertyCoefficients` заменяет только process-global static; порядок
//! чтений, записей и вызов после codec-а, включая `include_child=false`, не
//! меняются. Occupation вне `0..3` оставляет уже скопированные Str/Dex/Con/Int
//! и останавливается локальным `BLOCKED_MISSING_FACT`, потому что оригинал
//! индексировал соседнюю static memory за пределами PDB-массива.
//!
//! Exact `0x0045AFC0..0x0045B259` восстановил потерянный x87 stack: burden
//! использует `Str*fStr2Burden`, defense — `Con*fCon2Def`, resistant/element —
//! сохранённый в binary32 `Int`, а re-ank — сохранённый binary32
//! `Dex*fDex2Stiff`. Все `fistp` временно ставят RC=truncate; `__ftol2` также
//! усекает к нулю. Первые MaxHp/MaxMp/MinAtk/MaxAtk и Def используют точный
//! `u32 × binary32` в 64-битной x87 significand; поздние Int/Dex выражения
//! сначала сохраняют stat в binary32, как `fst dword` оригинала.
//!
//! Safe `softfloat-wrapper` не предоставляет `extFloat80`, а доступные raw
//! extFloat80 bindings требуют широкого `unsafe` FFI. Узкий compatibility-
//! helper не реализует общий soft-float runtime: он раскладывает только два
//! достигнутых binary произведения на IEEE-754 significand/exponent. Их максимум
//! 56 значащих бит, поэтому результат точно помещается в 64-битную x87
//! significand; truncation, signed overflow и integer-indefinite сохраняются
//! без `unsafe` и без недостаточного промежуточного `f64`. Остальные байты
//! `tagProperty[0x9c]` не меняются.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::sync::atomic::{AtomicI32, Ordering};

static NEXT_NET_EXCHANGE_ID: AtomicI32 = AtomicI32::new(0);

use crate::dbaccess::worlddb::dbgoods::{DbGoodsOwner, PlayerGoodsFiledSnapshot};
use crate::dbaccess::worlddb::goodslistener::{GoodsContainerTraversalSnapshot, TraversedGoods};
use crate::dbaccess::worlddb::rsjjcsys::{PlayerJjcDataSnapshot, RsJjcSysOwner};
use crate::dbaccess::worlddb::rsplayer::{
    EmbeddedFriendNameNul, PlayerAbilityCreationSnapshot, PlayerAbilitySaveSnapshot,
    PlayerAbilityScalarSnapshot, PlayerAbilitySkill, PlayerBaseSaveSnapshot,
    PlayerCreationBaseSnapshot, PlayerCreationSnapshot, PlayerFriendName, PlayerQuestSaveEntry,
    PlayerQuestSaveSnapshot, PlayerSaveOutcome, PlayerSaveSnapshot, PlayerScriptFlagSnapshot,
    PlayerThing as DbPlayerThing, RsPlayerOwner,
};
use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

use super::container::camountlimitgoodscontainer::{
    AmountContainerCodecError, CAmountLimitGoodsContainer,
};
use super::container::cbank::CBank;
use super::container::cbattlefairycontainer::CBattleFairyContainer;
use super::container::cdepot::CDepot;
use super::container::cequipmentcontainer::{CEquipmentContainer, EquipmentContainerCodecError};
use super::container::cfairycontainer::CFairyContainer;
use super::container::cjifen::CJiFen;
use super::container::cvolumelimitgoodscontainer::{
    CVolumeLimitGoodsContainer, VolumeContainerCodecError,
};
use super::container::cwallet::CWallet;
use super::container::cyuanbao::CYuanBao;
use super::goods::cgoods::{GoodsCodecError, GoodsDbSnapshotBlock};
use super::goods::cgoodsbaseproperties::GAP_WEAPON_LEVEL;
use super::goods::cgoodsfactory::{GoodsBasePropertiesRegistry, GoodsOriginalNameIndex};
use super::listener::cseekgoodslistener::CSeekGoodsListener;
use super::moveshape::CMoveShape;
use super::shape::{ShapeDecodeError, ShapeTileCoordinateBlock};

const BASE_PROPERTY_WIRE_LEN: usize = 0x194;
const BASE_PROPERTY_LEVEL_OFFSET: usize = 0x04;
const BASE_PROPERTY_EXP_OFFSET: usize = 0x08;
const BASE_PROPERTY_HEAD_PIC_OFFSET: usize = 0x0C;
const BASE_PROPERTY_FACE_PIC_OFFSET: usize = 0x0D;
const BASE_PROPERTY_FY_ENERGY_OFFSET: usize = 0x17C;
const BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET: usize = 0x180;
const BASE_PROPERTY_LT_UP_60_COUNT_OFFSET: usize = 0x184;
const BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET: usize = 0x186;
const BASE_PROPERTY_LT_60_STAMP_OFFSET: usize = 0x188;
const BASE_PROPERTY_FOSTER_NUM_OFFSET: usize = 0x120;
const BASE_PROPERTY_HATCHER_NUM_OFFSET: usize = 0x124;
const BASE_PROPERTY_OCCUPATION_OFFSET: usize = 0x0E;
const BASE_PROPERTY_SEX_OFFSET: usize = 0x0F;
const BASE_PROPERTY_SPOUSE_ID_OFFSET: usize = 0x10;
const BASE_PROPERTY_UNION_ID_OFFSET: usize = 0x14;
const BASE_PROPERTY_PK_COUNT_OFFSET: usize = 0x1C;
const BASE_PROPERTY_KILL_COUNT_OFFSET: usize = 0x20;
const BASE_PROPERTY_HIT_TOP_LOG_OFFSET: usize = 0x24;
const BASE_PROPERTY_HOT_HIT_OFFSET: usize = 0x28;
const BASE_PROPERTY_LOAN_MAX_OFFSET: usize = 0x2C;
const BASE_PROPERTY_LOAN_OFFSET: usize = 0x30;
const BASE_PROPERTY_LOAN_TIME_OFFSET: usize = 0x34;
const BASE_PROPERTY_IS_CHARGED_OFFSET: usize = 0x38;
const BASE_PROPERTY_REMAIN_POINT_OFFSET: usize = 0x3A;
const BASE_PROPERTY_HOT_KEYS_OFFSET: usize = 0x3C;
const BASE_PROPERTY_PK_NORMAL_OFFSET: usize = 0x9C;
const BASE_PROPERTY_PK_TEAM_OFFSET: usize = 0x9D;
const BASE_PROPERTY_PK_UNION_OFFSET: usize = 0x9E;
const BASE_PROPERTY_PK_BADMAN_OFFSET: usize = 0x9F;
const BASE_PROPERTY_PK_COUNTRY_OFFSET: usize = 0xA0;
const BASE_PROPERTY_YP_OFFSET: usize = 0xA2;
const BASE_PROPERTY_HP_OFFSET: usize = 0xA4;
const BASE_PROPERTY_MP_OFFSET: usize = 0xA8;
const BASE_PROPERTY_RP_OFFSET: usize = 0xAC;
const BASE_PROPERTY_MAX_HP_OFFSET: usize = 0xB0;
const BASE_PROPERTY_MAX_MP_OFFSET: usize = 0xB4;
const BASE_PROPERTY_MAX_YP_OFFSET: usize = 0xB8;
const BASE_PROPERTY_MAX_RP_OFFSET: usize = 0xBA;
const BASE_PROPERTY_STR_OFFSET: usize = 0xBC;
const BASE_PROPERTY_DEX_OFFSET: usize = 0xC0;
const BASE_PROPERTY_CON_OFFSET: usize = 0xC4;
const BASE_PROPERTY_INT_OFFSET: usize = 0xC8;
const BASE_PROPERTY_MIN_ATK_OFFSET: usize = 0xCC;
const BASE_PROPERTY_MAX_ATK_OFFSET: usize = 0xD0;
const BASE_PROPERTY_HIT_OFFSET: usize = 0xD4;
const BASE_PROPERTY_BURDEN_OFFSET: usize = 0xD6;
const BASE_PROPERTY_CCH_OFFSET: usize = 0xD8;
const BASE_PROPERTY_DEF_OFFSET: usize = 0xDC;
const BASE_PROPERTY_DODGE_OFFSET: usize = 0xE0;
const BASE_PROPERTY_ATC_SPEED_OFFSET: usize = 0xE2;
const BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET: usize = 0xE4;
const BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET: usize = 0xE8;
const BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET: usize = 0xEA;
const BASE_PROPERTY_VIGOUR_OFFSET: usize = 0xEC;
const BASE_PROPERTY_MAX_VIGOUR_OFFSET: usize = 0xF0;
const BASE_PROPERTY_ENERGY_OFFSET: usize = 0xF4;
const BASE_PROPERTY_MAX_ENERGY_OFFSET: usize = 0xF8;
const BASE_PROPERTY_CREDIT_OFFSET: usize = 0xFC;
const BASE_PROPERTY_EXALT_OFFSET: usize = 0x100;
const BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET: usize = 0x104;
const BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET: usize = 0x108;
const BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET: usize = 0x10C;
const BASE_PROPERTY_QUEST_OFFSET: usize = 0x110;
const BASE_PROPERTY_EXPLOIT_OFFSET: usize = 0x114;
const BASE_PROPERTY_KUDOS_OFFSET: usize = 0x118;
const BASE_PROPERTY_FAIRY_ENABLED_OFFSET: usize = 0x11C;
const BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET: usize = 0x128;
const BASE_PROPERTY_DAYS_HONOR_OFFSET: usize = 0x140;
const BASE_PROPERTY_WEEKS_HONOR_OFFSET: usize = 0x144;
const BASE_PROPERTY_MONTHS_HONOR_OFFSET: usize = 0x148;
const BASE_PROPERTY_TOTAL_HONOR_OFFSET: usize = 0x14C;
const BASE_PROPERTY_RANK_NOBILITY_OFFSET: usize = 0x150;
const BASE_PROPERTY_APPELLATION_OFFSET: usize = 0x154;
const BASE_PROPERTY_MODE_OFFSET: usize = 0x158;
const BASE_PROPERTY_FETCH_POWER_OFFSET: usize = 0x164;
const BASE_PROPERTY_MAX_FETCH_POWER_OFFSET: usize = 0x168;
const BASE_PROPERTY_AUCTION_SPACE_OFFSET: usize = 0x170;
const BASE_PROPERTY_JJC_LEVEL_OFFSET: usize = 0x174;
const BASE_PROPERTY_JJC_SCORE_OFFSET: usize = 0x178;
const BASE_PROPERTY_SZL_OFFSET: usize = 0x18C;
const BASE_PROPERTY_GODS_BATTLE_FACTION_OFFSET: usize = 0x190;

const PROPERTY_MAX_HP_OFFSET: usize = 0x00;
const PROPERTY_MAX_MP_OFFSET: usize = 0x04;
const PROPERTY_MAX_YP_OFFSET: usize = 0x08;
const PROPERTY_MAX_RP_OFFSET: usize = 0x0A;
const PROPERTY_STR_OFFSET: usize = 0x0C;
const PROPERTY_DEX_OFFSET: usize = 0x10;
const PROPERTY_CON_OFFSET: usize = 0x14;
const PROPERTY_INT_OFFSET: usize = 0x18;
const PROPERTY_MIN_ATK_OFFSET: usize = 0x1C;
const PROPERTY_MAX_ATK_OFFSET: usize = 0x20;
const PROPERTY_HIT_OFFSET: usize = 0x24;
const PROPERTY_BURDEN_OFFSET: usize = 0x26;
const PROPERTY_CCH_OFFSET: usize = 0x28;
const PROPERTY_DEF_OFFSET: usize = 0x2C;
const PROPERTY_DODGE_OFFSET: usize = 0x30;
const PROPERTY_ATC_SPEED_OFFSET: usize = 0x32;
const PROPERTY_ELEMENT_RESISTANT_OFFSET: usize = 0x34;
const PROPERTY_HP_RECOVER_SPEED_OFFSET: usize = 0x38;
const PROPERTY_MP_RECOVER_SPEED_OFFSET: usize = 0x3A;
const PROPERTY_ELEMENT_MODIFY_OFFSET: usize = 0x48;
const PROPERTY_RE_ANK_OFFSET: usize = 0x4C;

/// Ошибка безопасной границы собственных player byte-array owner-ов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCodecError {
    Shape(ShapeDecodeError),
    Goods(GoodsCodecError),
    AmountContainer(AmountContainerCodecError),
    VolumeContainer(VolumeContainerCodecError),
    EquipmentContainer(EquipmentContainerCodecError),
    OrganizingUpdate(PlayerOrganizingUpdateError),
    CollectionLengthOutsideLegacyRange {
        field: &'static str,
        count: usize,
    },
    BufferShorterThanDeclaredLength {
        field: &'static str,
        declared: usize,
        available: usize,
    },
    StringOutsideLegacyCapacity {
        field: &'static str,
        length: usize,
        capacity: usize,
    },
    OccupationOutsidePropertyCoefficientRange {
        occupation: u8,
    },
    UninitializedWireField {
        field: &'static str,
    },
    NegativeLength {
        field: &'static str,
        value: i32,
    },
    UnterminatedString {
        field: &'static str,
        offset: usize,
        capacity: usize,
        available: usize,
    },
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for PlayerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape(error) => error.fmt(formatter),
            Self::Goods(error) => error.fmt(formatter),
            Self::AmountContainer(error) => error.fmt(formatter),
            Self::VolumeContainer(error) => error.fmt(formatter),
            Self::EquipmentContainer(error) => error.fmt(formatter),
            Self::OrganizingUpdate(error) => error.fmt(formatter),
            Self::CollectionLengthOutsideLegacyRange { field, count } => write!(
                formatter,
                "коллекция {field} содержит {count} элементов вне 32-битного legacy-диапазона"
            ),
            Self::BufferShorterThanDeclaredLength {
                field,
                declared,
                available,
            } => write!(
                formatter,
                "буфер {field} объявляет {declared} байт, но содержит только {available}"
            ),
            Self::StringOutsideLegacyCapacity {
                field,
                length,
                capacity,
            } => write!(
                formatter,
                "строка {field} длиной {length} не помещается в legacy-буфер {capacity} байт с NUL"
            ),
            Self::OccupationOutsidePropertyCoefficientRange { occupation } => write!(
                formatter,
                "occupation {occupation} выходит за три PDB-коэффициента UpdateProperty"
            ),
            Self::UninitializedWireField { field } => write!(
                formatter,
                "constructor CPlayer не назначил wire-поле {field} до сериализации"
            ),
            Self::NegativeLength { field, value } => {
                write!(
                    formatter,
                    "поле {field} содержит отрицательную длину {value}"
                )
            }
            Self::UnterminatedString {
                field,
                offset,
                capacity,
                available,
            } => write!(
                formatter,
                "строка {field} с offset {offset} не завершена NUL в legacy-буфере {capacity} байт; доступно {available}"
            ),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for PlayerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Shape(error) => Some(error),
            Self::Goods(error) => Some(error),
            Self::AmountContainer(error) => Some(error),
            Self::VolumeContainer(error) => Some(error),
            Self::EquipmentContainer(error) => Some(error),
            Self::OrganizingUpdate(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ShapeDecodeError> for PlayerCodecError {
    fn from(error: ShapeDecodeError) -> Self {
        Self::Shape(error)
    }
}

impl From<GoodsCodecError> for PlayerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

impl From<AmountContainerCodecError> for PlayerCodecError {
    fn from(error: AmountContainerCodecError) -> Self {
        Self::AmountContainer(error)
    }
}

impl From<VolumeContainerCodecError> for PlayerCodecError {
    fn from(error: VolumeContainerCodecError) -> Self {
        Self::VolumeContainer(error)
    }
}

impl From<EquipmentContainerCodecError> for PlayerCodecError {
    fn from(error: EquipmentContainerCodecError) -> Self {
        Self::EquipmentContainer(error)
    }
}

impl From<PlayerOrganizingUpdateError> for PlayerCodecError {
    fn from(error: PlayerOrganizingUpdateError) -> Self {
        Self::OrganizingUpdate(error)
    }
}

/// Первые byte-exact `0x194` байт `tagBaseProperty` и два следующих string-owner-а.
struct PlayerBaseProperty {
    wire: [u8; BASE_PROPERTY_WIRE_LEN],
    account: Vec<u8>,
    title: Vec<u8>,
}

/// Наблюдаемый результат двух прямых мутаций `tagBaseProperty` из
/// `OnServerMessage(0x5FA06)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurderCounterUpdate {
    pub(crate) previous_kill_count: u32,
    pub(crate) kill_count: u32,
    pub(crate) previous_pk_count: u16,
    pub(crate) pk_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerMurderCounterReset {
    pub(crate) previous_kill_count: u32,
    pub(crate) previous_pk_count: u16,
}

/// Результат native-width прибавления к `tagBaseProperty::dwExploit`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerExploitUpdate {
    pub(crate) previous_exploit: u32,
    pub(crate) increment: i32,
    pub(crate) exploit: u32,
}

impl PlayerBaseProperty {
    fn read_u8(&self, offset: usize) -> u8 {
        self.wire[offset]
    }

    fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes(
            self.wire[offset..offset + 2]
                .try_into()
                .expect("PDB-offset находится внутри tagBaseProperty wire-prefix"),
        )
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes(
            self.wire[offset..offset + 4]
                .try_into()
                .expect("PDB-offset находится внутри tagBaseProperty wire-prefix"),
        )
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// Byte-exact `tagProperty`; reached owner меняет только доказанные offsets.
struct PlayerProperty {
    wire: [u8; 0x9C],
}

impl PlayerProperty {
    fn write_u16(&mut self, offset: usize, value: u16) {
        self.wire[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.wire[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// Точный восьмибайтовый mapped value `tagThing`.
#[derive(Clone, Copy)]
struct PlayerThing {
    thing_id: u16,
    count: u16,
    max_count: u16,
    point: u16,
}

/// Точный четырёхбайтовый value `CPlayer::tagSkill`.
#[derive(Clone, Copy)]
struct PlayerSkill {
    skill_id: u16,
    level: u16,
}

/// Собственные данные одного list-элемента `CPlayer::tagFriend`.
struct PlayerFriend {
    name: Vec<u8>,
    online: bool,
}

/// Отдельный mapped value `CPlayer::tagPlayerQuest`.
#[derive(Clone, Copy)]
struct PlayerQuest {
    quest_id: u16,
    complete: u8,
}

/// Собственный wire-набор одного `CPlayer::tagPetInformation`.
struct PlayerPetInformation {
    original_name: Vec<u8>,
    hp: u32,
    level: u32,
    experience: u32,
}

/// Собственные достигнутые поля `CPlayer::tagCarriageInfo`.
struct PlayerCarriageInformation {
    original_name: Vec<u8>,
    hp: u32,
    carriage_script: Vec<u8>,
}

/// Достигнутые массивы `CGlobeSetup::tagSetup`, каждый строго для трёх occupations.
pub(crate) struct PlayerPropertyCoefficients {
    pub(crate) str_to_max_attack: [f32; 3],
    pub(crate) str_to_burden: [f32; 3],
    pub(crate) dex_to_min_attack: [f32; 3],
    pub(crate) dex_to_stiff: [f32; 3],
    pub(crate) con_to_max_hp: [f32; 3],
    pub(crate) con_to_defense: [f32; 3],
    pub(crate) int_to_element: [f32; 3],
    pub(crate) int_to_max_mp: [f32; 3],
    pub(crate) int_to_resistant: [f32; 3],
}

/// Достигнутые organization-поля `CPlayer`, обновляемые `SetPlayerOrganizing`.
pub(crate) struct PlayerOrganizingState {
    pub(crate) faction_id: i32,
    pub(crate) faction_logo_id: i32,
    pub(crate) faction_name: Vec<u8>,
    pub(crate) faction_title: Vec<u8>,
    pub(crate) faction_master_id: i32,
    pub(crate) union_id: i32,
    pub(crate) union_master_id: i32,
    pub(crate) faction_level: u16,
    pub(crate) faction_experience: i32,
    pub(crate) force: i32,
    pub(crate) faction_contribute: bool,
    pub(crate) enemy_factions: BTreeSet<i32>,
    pub(crate) city_war_enemy_factions: BTreeSet<i32>,
    /// Полные восемь wire-байт `tagOwnedReg`, включая наблюдаемый padding.
    pub(crate) owned_regions: VecDeque<[u8; 8]>,
}

/// Safe-границы точного `COrganizingCtrl::SetPlayerOrganizing`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerOrganizingUpdateError {
    NullFactionDuringMembershipScan {
        map_key: i32,
    },
    NullConfederationDuringMembershipScan {
        map_key: i32,
    },
    UninitializedFactionField {
        faction_id: i32,
        field: &'static str,
    },
    UnterminatedFactionMemberTitle {
        faction_id: i32,
        player_id: i32,
    },
    UninitializedRegionType {
        region_id: i32,
    },
    UninitializedOwnedRegionPadding {
        region_id: i32,
    },
}

impl fmt::Display for PlayerOrganizingUpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullFactionDuringMembershipScan { map_key } => write!(
                formatter,
                "faction-map содержит null по ключу {map_key} во время IsFreePlayer"
            ),
            Self::NullConfederationDuringMembershipScan { map_key } => write!(
                formatter,
                "confederation-map содержит null по ключу {map_key} во время IsFreeFaction"
            ),
            Self::UninitializedFactionField { faction_id, field } => write!(
                formatter,
                "faction {faction_id} не материализовала поле {field} до SetPlayerOrganizing"
            ),
            Self::UnterminatedFactionMemberTitle {
                faction_id,
                player_id,
            } => write!(
                formatter,
                "title игрока {player_id} во faction {faction_id} не завершён NUL"
            ),
            Self::UninitializedRegionType { region_id } => write!(
                formatter,
                "регион {region_id} не материализовал REGION_TYPE до SetPlayerOrganizing"
            ),
            Self::UninitializedOwnedRegionPadding { region_id } => write!(
                formatter,
                "для owned-региона {region_id} неизвестны два наблюдаемых padding-байта tagOwnedReg"
            ),
        }
    }
}

impl Error for PlayerOrganizingUpdateError {}

/// Явная замена process-global `COrganizingCtrl::getInstance()`.
pub(crate) trait PlayerOrganizingUpdater {
    /// Выполняет исходный `SetPlayerOrganizing` перед чтением organization-полей.
    fn set_player_organizing(
        &mut self,
        player_id: i32,
        organizing: &mut PlayerOrganizingState,
    ) -> Result<(), PlayerOrganizingUpdateError>;
}

/// Owned DB-проекция пятнадцати frozen container-ов одного `CPlayer`.
pub(crate) struct PlayerGoodsDbProjection {
    packet: Vec<TraversedGoods>,
    equipment: Vec<TraversedGoods>,
    hand: Vec<TraversedGoods>,
    wallet: Vec<TraversedGoods>,
    yuan_bao: Vec<TraversedGoods>,
    ji_fen: Vec<TraversedGoods>,
    bank: Vec<TraversedGoods>,
    depot: Vec<TraversedGoods>,
    fairy: Vec<TraversedGoods>,
    battle_fairy: Vec<TraversedGoods>,
    auction_goods: Vec<TraversedGoods>,
    auction_wallet: Vec<TraversedGoods>,
    auction: Vec<TraversedGoods>,
    ci_qing: Vec<TraversedGoods>,
    compose_ci_qing: Vec<TraversedGoods>,
}

/// Локальная граница материализации frozen `CPlayer` для WorldDB.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PlayerDbProjectionBlock {
    Goods(GoodsDbSnapshotBlock),
    UninitializedField { field: &'static str },
    NegativeVariableDataLength { value: i32 },
    VariableDataShorterThanDeclared { declared: usize, available: usize },
    FriendNameContainsNul { friend_index: usize, offset: usize },
}

impl From<GoodsDbSnapshotBlock> for PlayerDbProjectionBlock {
    fn from(block: GoodsDbSnapshotBlock) -> Self {
        Self::Goods(block)
    }
}

/// Полный owned/borrowed compatibility-layer одного frozen DB lifecycle.
pub(crate) struct PlayerDbProjection<'player> {
    player: &'player CPlayer,
    country: u8,
    contribute: i32,
    goods: PlayerGoodsDbProjection,
    hot_keys: [u32; 24],
    skills: Vec<PlayerAbilitySkill>,
    friend_names: Vec<PlayerFriendName<'player>>,
    things: Vec<DbPlayerThing>,
    quests: BTreeMap<u16, PlayerQuestSaveEntry>,
    variable_data: &'player [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerCountryChangeDisposition {
    SameCountry,
    FactionMember { faction_id: i32 },
    CountryMissing,
    Changed { previous_country: Option<u8> },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerCountryChangeReport {
    pub(crate) requested_country: u8,
    pub(crate) legacy_result: i32,
    pub(crate) disposition: PlayerCountryChangeDisposition,
}

impl PlayerGoodsDbProjection {
    /// Заимствует все containers в exact `SaveGoodsFiled` place-порядке.
    pub(crate) fn snapshot(&self, player_id: i32) -> PlayerGoodsFiledSnapshot<'_> {
        let traversal = |objects| GoodsContainerTraversalSnapshot { objects };
        PlayerGoodsFiledSnapshot {
            player_id,
            packet: traversal(&self.packet),
            equipment: traversal(&self.equipment),
            hand: traversal(&self.hand),
            wallet: traversal(&self.wallet),
            yuan_bao: traversal(&self.yuan_bao),
            ji_fen: traversal(&self.ji_fen),
            bank: traversal(&self.bank),
            depot: traversal(&self.depot),
            fairy: traversal(&self.fairy),
            battle_fairy: traversal(&self.battle_fairy),
            auction_goods: traversal(&self.auction_goods),
            auction_wallet: traversal(&self.auction_wallet),
            auction: traversal(&self.auction),
            ci_qing: traversal(&self.ci_qing),
            compose_ci_qing: traversal(&self.compose_ci_qing),
        }
    }
}

/// Достигнутая часть `CPlayer` с base-owner-ом и clone-кодеком через containers.
pub(crate) struct CPlayer {
    move_shape_base: CMoveShape,
    hand: CAmountLimitGoodsContainer,
    packet: CVolumeLimitGoodsContainer,
    equipment: CEquipmentContainer,
    wallet: CWallet,
    yuan_bao: CYuanBao,
    ji_fen: CJiFen,
    bank: CBank,
    depot: CDepot,
    fairy: CFairyContainer,
    battle_fairy: CBattleFairyContainer,
    auction_goods_container: CVolumeLimitGoodsContainer,
    auction_container: CVolumeLimitGoodsContainer,
    auction_wallet: CWallet,
    ci_qing: CVolumeLimitGoodsContainer,
    compose_ci_qing: CVolumeLimitGoodsContainer,
    depot_password: Vec<u8>,
    base_property: PlayerBaseProperty,
    property: PlayerProperty,
    team_id: i32,
    ci_qing_ids: BTreeSet<u32>,
    new_skills: VecDeque<PlayerSkill>,
    friends: VecDeque<PlayerFriend>,
    daily_things: VecDeque<PlayerThing>,
    player_quests: BTreeMap<u16, PlayerQuest>,
    variable_num: i32,
    variable_data: Option<Vec<u8>>,
    variable_data_length: i32,
    silience_time: i32,
    murderer_time: u32,
    fight_state_count: i32,
    uncreated_pets: Vec<PlayerPetInformation>,
    uncreated_carriage: PlayerCarriageInformation,
    recreate_carriage: bool,
    login: bool,
    city_war_died_state_time: i32,
    country: Option<u8>,
    contribute: Option<i32>,
    jjc_data: [u8; 0x10],
    jjc_pk_state: bool,
    session_id: Vec<u8>,
    organizing: PlayerOrganizingState,
    faction_data_received: bool,
    faction_war_operator: bool,
}

impl PlayerDbProjection<'_> {
    fn equipment_fields(&self) -> Result<([u32; 11], [i32; 11]), PlayerDbProjectionBlock> {
        const SQL_EQUIPMENT_ORDER: [u32; 11] = [0, 1, 3, 4, 2, 9, 10, 12, 13, 14, 15];
        let mut ids = [0; 11];
        let mut levels = [0; 11];
        for (index, position) in SQL_EQUIPMENT_ORDER.into_iter().enumerate() {
            let Some(goods) = self.player.equipment.get_goods(position) else {
                continue;
            };
            ids[index] = goods
                .get_base_properties_index()
                .ok_or(GoodsDbSnapshotBlock::MissingBasePropertiesIndex)?;
            levels[index] = goods.get_addon_property_value(GAP_WEAPON_LEVEL, 1);
        }
        Ok((ids, levels))
    }

    fn jjc_snapshot(&self) -> PlayerJjcDataSnapshot {
        let word = |offset: usize| {
            u16::from_le_bytes(
                self.player.jjc_data[offset..offset + 2]
                    .try_into()
                    .expect("tagPlayerJJcData содержит восемь полных u16"),
            )
        };
        PlayerJjcDataSnapshot {
            id: self.player.get_id(),
            jjc_level: self
                .player
                .base_property
                .read_u32(BASE_PROPERTY_JJC_LEVEL_OFFSET),
            jjc_score: self
                .player
                .base_property
                .read_u32(BASE_PROPERTY_JJC_SCORE_OFFSET),
            week_join: word(0),
            week_win: word(2),
            week_lose: word(4),
            week_tie: word(6),
            season_join: word(8),
            season_win: word(10),
            season_lose: word(12),
            season_tie: word(14),
        }
    }

    fn scalar_snapshot(&self) -> PlayerAbilityScalarSnapshot<'_> {
        let base = &self.player.base_property;
        let flag = |offset| base.read_u8(offset) != 0;
        PlayerAbilityScalarSnapshot {
            id: self.player.get_id(),
            name: self.player.get_name(),
            region_id: self.player.get_region_id(),
            pos_x: self.player.move_shape_base.get_pos_x(),
            pos_y: self.player.move_shape_base.get_pos_y(),
            dir: self.player.move_shape_base.get_direction(),
            account: &base.account,
            title: &base.title,
            level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
            exp: base.read_u32(BASE_PROPERTY_EXP_OFFSET),
            head_pic: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
            face_pic: base.read_u8(BASE_PROPERTY_FACE_PIC_OFFSET),
            occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
            sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
            spouse_id: base.read_u32(BASE_PROPERTY_SPOUSE_ID_OFFSET),
            union_id: base.read_u32(BASE_PROPERTY_UNION_ID_OFFSET),
            murderer_time: self.player.murderer_time,
            pk_count: base.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET),
            kill_count: base.read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET),
            hit_top_log: base.read_u16(BASE_PROPERTY_HIT_TOP_LOG_OFFSET),
            hot_hit: base.read_u32(BASE_PROPERTY_HOT_HIT_OFFSET),
            loan_max: base.read_u32(BASE_PROPERTY_LOAN_MAX_OFFSET),
            loan: base.read_u32(BASE_PROPERTY_LOAN_OFFSET),
            loan_time: base.read_u32(BASE_PROPERTY_LOAN_TIME_OFFSET) as i32,
            remain_point: base.read_u16(BASE_PROPERTY_REMAIN_POINT_OFFSET),
            pk_normal: flag(BASE_PROPERTY_PK_NORMAL_OFFSET),
            pk_team: flag(BASE_PROPERTY_PK_TEAM_OFFSET),
            pk_union: flag(BASE_PROPERTY_PK_UNION_OFFSET),
            pk_badman: flag(BASE_PROPERTY_PK_BADMAN_OFFSET),
            pk_country: flag(BASE_PROPERTY_PK_COUNTRY_OFFSET),
            yp: base.read_u16(BASE_PROPERTY_YP_OFFSET),
            hp: base.read_u32(BASE_PROPERTY_HP_OFFSET),
            mp: base.read_u32(BASE_PROPERTY_MP_OFFSET),
            rp: base.read_u16(BASE_PROPERTY_RP_OFFSET),
            base_max_hp: base.read_u32(BASE_PROPERTY_MAX_HP_OFFSET),
            base_max_mp: base.read_u32(BASE_PROPERTY_MAX_MP_OFFSET),
            base_max_yp: base.read_u16(BASE_PROPERTY_MAX_YP_OFFSET),
            base_max_rp: base.read_u16(BASE_PROPERTY_MAX_RP_OFFSET),
            base_str: base.read_u32(BASE_PROPERTY_STR_OFFSET),
            base_dex: base.read_u32(BASE_PROPERTY_DEX_OFFSET),
            base_con: base.read_u32(BASE_PROPERTY_CON_OFFSET),
            base_int: base.read_u32(BASE_PROPERTY_INT_OFFSET),
            base_min_atk: base.read_u32(BASE_PROPERTY_MIN_ATK_OFFSET),
            base_max_atk: base.read_u32(BASE_PROPERTY_MAX_ATK_OFFSET),
            base_hit: base.read_u16(BASE_PROPERTY_HIT_OFFSET),
            base_burden: base.read_u16(BASE_PROPERTY_BURDEN_OFFSET),
            base_cch: base.read_u16(BASE_PROPERTY_CCH_OFFSET),
            base_def: base.read_u32(BASE_PROPERTY_DEF_OFFSET),
            base_dodge: base.read_u16(BASE_PROPERTY_DODGE_OFFSET),
            base_atc_speed: base.read_u16(BASE_PROPERTY_ATC_SPEED_OFFSET),
            base_element_resistant: base.read_u32(BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET),
            base_hp_recover_speed: base.read_u16(BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET),
            base_mp_recover_speed: base.read_u16(BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET),
            base_vigour: base.read_u32(BASE_PROPERTY_VIGOUR_OFFSET),
            base_max_vigour: base.read_u32(BASE_PROPERTY_MAX_VIGOUR_OFFSET),
            base_energy: base.read_u32(BASE_PROPERTY_ENERGY_OFFSET),
            base_max_energy: base.read_u32(BASE_PROPERTY_MAX_ENERGY_OFFSET),
            base_credit: base.read_u32(BASE_PROPERTY_CREDIT_OFFSET),
            display_head_piece: base.read_u8(BASE_PROPERTY_DISPLAY_HEAD_PIECE_OFFSET),
            country: self.country,
            contribute: self.contribute,
            is_charged: flag(BASE_PROPERTY_IS_CHARGED_OFFSET),
            quest_time_begin: base.read_u32(BASE_PROPERTY_QUEST_TIME_BEGIN_OFFSET) as i32,
            quest_time_limit: base.read_u32(BASE_PROPERTY_QUEST_TIME_LIMIT_OFFSET) as i32,
            quest: flag(BASE_PROPERTY_QUEST_OFFSET),
            depot_password: &self.player.depot_password,
            exploit: base.read_u32(BASE_PROPERTY_EXPLOIT_OFFSET),
            kudos: base.read_u32(BASE_PROPERTY_KUDOS_OFFSET),
            mode: base.read_u32(BASE_PROPERTY_MODE_OFFSET),
            fairy_enabled: flag(BASE_PROPERTY_FAIRY_ENABLED_OFFSET),
            foster_num: base.read_u32(BASE_PROPERTY_FOSTER_NUM_OFFSET),
            hatcher_num: base.read_u32(BASE_PROPERTY_HATCHER_NUM_OFFSET),
            battle_fairy_enabled: flag(BASE_PROPERTY_BATTLE_FAIRY_ENABLED_OFFSET),
            fetch_power: base.read_u32(BASE_PROPERTY_FETCH_POWER_OFFSET),
            max_fetch_power: base.read_u32(BASE_PROPERTY_MAX_FETCH_POWER_OFFSET),
            auction_space: base.read_u32(BASE_PROPERTY_AUCTION_SPACE_OFFSET),
            exalt: base.read_u32(BASE_PROPERTY_EXALT_OFFSET),
            szl: base.read_u32(BASE_PROPERTY_SZL_OFFSET),
            gods_battle_faction: base.read_u32(BASE_PROPERTY_GODS_BATTLE_FACTION_OFFSET) as i32,
            base_fy_energy: base.read_u32(BASE_PROPERTY_FY_ENERGY_OFFSET),
            base_bl_fy_energy: base.read_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET),
            lt_up_60_count: base.read_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET),
            remain_jl_dan_count: base.read_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET),
            lt_60_stamp: base.read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET),
        }
    }

    fn ability_creation_snapshot(&self) -> PlayerAbilityCreationSnapshot<'_> {
        PlayerAbilityCreationSnapshot {
            scalar: self.scalar_snapshot(),
            hot_keys: &self.hot_keys,
            skills: &self.skills,
            script_flag: PlayerScriptFlagSnapshot {
                variable_num: self.player.variable_num,
                variable_data: self.variable_data,
            },
            ex_states: self.player.move_shape_base.ex_states(),
            friend_names: &self.friend_names,
            ci_qing_ids: &self.player.ci_qing_ids,
            things: &self.things,
            jjc: self.jjc_snapshot(),
        }
    }

    /// Строит полные три стадии `CRsPlayer::CreatePlayer` из одного owner-а.
    pub(crate) fn creation_snapshot(
        &self,
    ) -> Result<PlayerCreationSnapshot<'_, '_>, PlayerDbProjectionBlock> {
        let base = &self.player.base_property;
        let (equipment_ids, equipment_levels) = self.equipment_fields()?;
        Ok(PlayerCreationSnapshot {
            base: PlayerCreationBaseSnapshot {
                id: self.player.get_id(),
                name: self.player.get_name().to_vec(),
                account: base.account.clone(),
                level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
                occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
                sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
                country: self.country,
                head: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
                equipment_ids,
                equipment_levels: equipment_levels.map(|level| level as u8),
                region_id: self.player.get_region_id(),
            },
            abilities: self.ability_creation_snapshot(),
            goods: self.goods.snapshot(self.player.get_id()),
        })
    }

    /// Строит полные четыре стадии `CRsPlayer::SavePlayer` из одного owner-а.
    pub(crate) fn save_snapshot(
        &self,
    ) -> Result<PlayerSaveSnapshot<'_, '_, '_>, PlayerDbProjectionBlock> {
        let base = &self.player.base_property;
        let (equipment_ids, equipment_levels) = self.equipment_fields()?;
        Ok(PlayerSaveSnapshot {
            base: PlayerBaseSaveSnapshot {
                id: self.player.get_id(),
                name: self.player.get_name(),
                level: base.read_u8(BASE_PROPERTY_LEVEL_OFFSET),
                occupation: base.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET),
                sex: base.read_u8(BASE_PROPERTY_SEX_OFFSET),
                country: self.country,
                head: base.read_u8(BASE_PROPERTY_HEAD_PIC_OFFSET),
                equipment_ids,
                equipment_levels,
                region_id: self.player.get_region_id(),
            },
            abilities: PlayerAbilitySaveSnapshot {
                ability: self.ability_creation_snapshot(),
                silence_time: self.player.silience_time,
                days_honor_eliminate_num: base.read_u32(BASE_PROPERTY_DAYS_HONOR_OFFSET),
                weeks_honor_eliminate_num: base.read_u32(BASE_PROPERTY_WEEKS_HONOR_OFFSET),
                months_honor_eliminate_num: base.read_u32(BASE_PROPERTY_MONTHS_HONOR_OFFSET),
                total_honor_eliminate_num: base.read_u32(BASE_PROPERTY_TOTAL_HONOR_OFFSET),
                rank_of_nobility_id: base.read_u32(BASE_PROPERTY_RANK_NOBILITY_OFFSET),
                appellation_id: base.read_u32(BASE_PROPERTY_APPELLATION_OFFSET),
            },
            quest: PlayerQuestSaveSnapshot {
                player_id: self.player.get_id(),
                quests: &self.quests,
            },
            goods: self.goods.snapshot(self.player.get_id()),
        })
    }
}

impl CPlayer {
    /// Создаёт constructor-state, необходимый точному `CloneMapPlayer` decoder-у.
    ///
    /// Это не объявление полного старого constructor-а: ещё не достигнутые поля
    /// остаются в raw-корпусе. `m_btCountry/m_lContribute` исходник не
    /// инициализировал, поэтому до обязательного полного decode они равны
    /// `None`, а не придуманному нулю.
    pub(crate) fn with_clone_decode_constructor_state() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(400);

        let mut hand = CAmountLimitGoodsContainer::with_constructor_defaults();
        let mut packet = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let equipment = CEquipmentContainer::with_constructor_defaults();
        let wallet = CWallet::with_constructor_defaults();
        let yuan_bao = CYuanBao::with_constructor_defaults();
        let ji_fen = CJiFen::with_constructor_defaults();
        let bank = CBank::with_constructor_defaults();
        let mut depot = CDepot::with_constructor_defaults();
        let mut fairy = CFairyContainer::with_constructor_defaults();
        let mut battle_fairy = CBattleFairyContainer::with_constructor_defaults();
        let mut auction_goods_container = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let mut auction_container = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let auction_wallet = CWallet::with_constructor_defaults();
        let mut ci_qing = CVolumeLimitGoodsContainer::with_constructor_defaults();
        let mut compose_ci_qing = CVolumeLimitGoodsContainer::with_constructor_defaults();

        let mut base_property = PlayerBaseProperty {
            wire: [0; BASE_PROPERTY_WIRE_LEN],
            account: Vec::new(),
            title: Vec::new(),
        };
        base_property.write_u32(BASE_PROPERTY_FOSTER_NUM_OFFSET, 1);
        base_property.write_u32(BASE_PROPERTY_HATCHER_NUM_OFFSET, 1);

        hand.set_goods_amount_limit(1);
        packet.set_container_volume_2d(8, 0x0C);
        auction_goods_container.set_container_volume(0x12);
        auction_container.set_container_volume(2);
        depot.set_container_volume(0xA1);
        fairy.set_container_volume(0x0E);
        battle_fairy.set_container_volume(0x11);
        ci_qing.set_container_volume(8);
        compose_ci_qing.set_container_volume(3);

        Self {
            move_shape_base,
            hand,
            packet,
            equipment,
            wallet,
            yuan_bao,
            ji_fen,
            bank,
            depot,
            fairy,
            battle_fairy,
            auction_goods_container,
            auction_container,
            auction_wallet,
            ci_qing,
            compose_ci_qing,
            depot_password: Vec::new(),
            base_property,
            property: PlayerProperty { wire: [0; 0x9C] },
            team_id: 0,
            ci_qing_ids: BTreeSet::new(),
            new_skills: VecDeque::new(),
            friends: VecDeque::new(),
            daily_things: VecDeque::new(),
            player_quests: BTreeMap::new(),
            variable_num: 0,
            variable_data: None,
            variable_data_length: 0,
            silience_time: 0,
            murderer_time: 0,
            fight_state_count: 0,
            uncreated_pets: Vec::new(),
            uncreated_carriage: PlayerCarriageInformation {
                original_name: Vec::new(),
                hp: 0,
                carriage_script: Vec::new(),
            },
            recreate_carriage: false,
            login: false,
            city_war_died_state_time: 0,
            country: None,
            contribute: None,
            jjc_data: [0; 0x10],
            jjc_pk_state: false,
            session_id: Vec::new(),
            organizing: PlayerOrganizingState {
                faction_id: 0,
                faction_logo_id: 0,
                faction_name: Vec::new(),
                faction_title: Vec::new(),
                faction_master_id: 0,
                union_id: 0,
                union_master_id: 0,
                faction_level: 0,
                faction_experience: 0,
                force: 0,
                faction_contribute: false,
                enemy_factions: BTreeSet::new(),
                city_war_enemy_factions: BTreeSet::new(),
                owned_regions: VecDeque::new(),
            },
            faction_data_received: false,
            faction_war_operator: false,
        }
    }

    /// Повторяет `CPlayer::GetMoney` RVA `0x0005AEA0` через wallet-owner.
    pub(crate) const fn money(&self) -> u32 {
        self.wallet.get_gold_coins_amount()
    }

    /// Считает unlocked amount товаров packet-а с точным original-name.
    pub(crate) fn check_goods_in_packet(
        &self,
        original_name: Option<&CStr>,
        original_name_index: &GoodsOriginalNameIndex,
    ) -> i32 {
        let Some(original_name) = original_name else {
            return 0;
        };

        let mut listener = CSeekGoodsListener::new();
        listener.set_target(Some(original_name), original_name_index);
        self.packet.traversing_container(Some(&mut listener));

        listener.goods_ids().iter().fold(0i32, |amount, ex_id| {
            self.packet.find(ex_id).map_or(amount, |goods| {
                amount.wrapping_add(goods.get_amount() as i32)
            })
        })
    }

    /// Возвращает сохранённые account-байты без придуманной перекодировки.
    pub(crate) fn get_account(&self) -> &[u8] {
        &self.base_property.account
    }

    /// Повторяет две последовательные native-width мутации ветки `0x5FA06`.
    pub(crate) fn increment_murder_counters(&mut self) -> PlayerMurderCounterUpdate {
        let previous_kill_count = self
            .base_property
            .read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET);
        let kill_count = previous_kill_count.wrapping_add(1);
        self.base_property
            .write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, kill_count);

        let previous_pk_count = self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET);
        let pk_count = previous_pk_count.wrapping_add(1);
        self.base_property
            .write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, pk_count);

        PlayerMurderCounterUpdate {
            previous_kill_count,
            kill_count,
            previous_pk_count,
            pk_count,
        }
    }

    /// Обнуляет exact `wPkCount/dwKillCount` в порядке исходного `Absolve`.
    pub(crate) fn reset_murder_counters(&mut self) -> PlayerMurderCounterReset {
        let previous_pk_count = self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET);
        self.base_property.write_u16(BASE_PROPERTY_PK_COUNT_OFFSET, 0);
        let previous_kill_count = self.base_property.read_u32(BASE_PROPERTY_KILL_COUNT_OFFSET);
        self.base_property.write_u32(BASE_PROPERTY_KILL_COUNT_OFFSET, 0);
        PlayerMurderCounterReset { previous_kill_count, previous_pk_count }
    }

    /// Возвращает exact unsigned `m_BaseProperty.wPkCount`.
    pub(crate) fn pk_count(&self) -> u16 {
        self.base_property.read_u16(BASE_PROPERTY_PK_COUNT_OFFSET)
    }

    /// Возвращает унаследованный exact `CMoveShape::m_bIsGod`.
    pub(crate) const fn is_god(&self) -> bool {
        self.move_shape_base.is_god()
    }

    /// Повторяет unsigned 32-bit сложение исходного `dwExploit += long`.
    pub(crate) fn add_exploit_wrapping(&mut self, increment: i32) -> PlayerExploitUpdate {
        let previous_exploit = self.base_property.read_u32(BASE_PROPERTY_EXPLOIT_OFFSET);
        let exploit = previous_exploit.wrapping_add(increment as u32);
        self.base_property
            .write_u32(BASE_PROPERTY_EXPLOIT_OFFSET, exploit);
        PlayerExploitUpdate {
            previous_exploit,
            increment,
            exploit,
        }
    }

    /// Сообщает, был ли игроку уже отправлен полный faction snapshot.
    pub(crate) const fn faction_data_received(&self) -> bool {
        self.faction_data_received
    }

    /// Сохраняет доказанную прямую мутацию `m_bGetFactionData`.
    pub(crate) const fn set_faction_data_received(&mut self, received: bool) {
        self.faction_data_received = received;
    }

    /// Возвращает exact transient-флаг, блокирующий смену главы faction.
    pub(crate) const fn faction_war_operator(&self) -> bool {
        self.faction_war_operator
    }

    pub(crate) const fn set_faction_war_operator(&mut self, enabled: bool) {
        self.faction_war_operator = enabled;
    }

    /// Возвращает signed ID через унаследованный `CBaseObject` owner.
    pub(crate) const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    /// Возвращает следующий process-wide ID старого inline `GetNetExID`.
    pub(crate) fn get_net_exchange_id(&self) -> i32 {
        NEXT_NET_EXCHANGE_ID
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1)
    }

    /// Возвращает signed type через унаследованный `CBaseObject` owner.
    pub(crate) const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    /// Присваивает signed ID через унаследованный `CBaseObject` owner.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    /// Заимствует byte-exact имя через унаследованный `CBaseObject` owner.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

    /// Заменяет exact `m_lSilienceTime`, возвращая прежнее значение.
    pub(crate) fn replace_silience_time(&mut self, silience_time: i32) -> i32 {
        std::mem::replace(&mut self.silience_time, silience_time)
    }

    /// Возвращает region ID через унаследованный shape-owner.
    pub(crate) const fn get_region_id(&self) -> i32 {
        self.move_shape_base.get_region_id()
    }

    /// Возвращает X-клетку через унаследованный shape-owner.
    pub(crate) fn get_tile_x(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.move_shape_base.get_tile_x()
    }

    /// Возвращает Y-клетку через унаследованный shape-owner.
    pub(crate) fn get_tile_y(&self) -> Result<i32, ShapeTileCoordinateBlock> {
        self.move_shape_base.get_tile_y()
    }

    /// Присваивает обе координаты через унаследованный shape-owner.
    pub(crate) const fn set_pos_xy(&mut self, pos_x: f32, pos_y: f32) {
        self.move_shape_base.set_pos_xy(pos_x, pos_y);
    }

    pub(crate) const fn set_direction(&mut self, direction: i32) -> bool {
        self.move_shape_base.set_direction(direction)
    }

    /// Ставит игрока в центр signed tile через унаследованный shape-owner.
    pub(crate) fn set_tile_xy(&mut self, tile_x: i32, tile_y: i32) {
        self.move_shape_base.set_tile_xy(tile_x, tile_y);
    }

    /// Возвращает country только после материализации player-state.
    pub(crate) const fn country(&self) -> Option<u8> {
        self.country
    }

    /// Повторяет exact `CPlayer::ChangeCountry`, получая singleton lookup явно.
    pub(crate) fn change_country(
        &mut self,
        requested_country: u8,
        country_exists: impl FnOnce(u8) -> bool,
    ) -> PlayerCountryChangeReport {
        let (legacy_result, disposition) = if self.country == Some(requested_country) {
            (-1, PlayerCountryChangeDisposition::SameCountry)
        } else if self.organizing.faction_id != 0 {
            (
                -3,
                PlayerCountryChangeDisposition::FactionMember {
                    faction_id: self.organizing.faction_id,
                },
            )
        } else if !country_exists(requested_country) {
            (-5, PlayerCountryChangeDisposition::CountryMissing)
        } else {
            let previous_country = self.country.replace(requested_country);
            (
                i32::from(requested_country),
                PlayerCountryChangeDisposition::Changed { previous_country },
            )
        };
        PlayerCountryChangeReport {
            requested_country,
            legacy_result,
            disposition,
        }
    }

    /// Возвращает достигнутый signed faction ID без преобразования.
    pub(crate) const fn faction_id(&self) -> i32 {
        self.organizing.faction_id
    }

    /// Возвращает signed `m_lTeamID` без изменения его bit-pattern.
    pub(crate) const fn get_team_id(&self) -> i32 {
        self.team_id
    }

    /// Возвращает полный восьмибитный уровень игрока.
    pub(crate) fn get_level(&self) -> u8 {
        self.base_property.read_u8(BASE_PROPERTY_LEVEL_OFFSET)
    }

    /// Возвращает exact unsigned `m_BaseProperty.dwCredit`.
    pub(crate) fn credit(&self) -> u32 {
        self.base_property.read_u32(BASE_PROPERTY_CREDIT_OFFSET)
    }

    /// Возвращает occupation из exact base-property offset `+0x0E`.
    pub(crate) fn get_occupation(&self) -> u8 {
        self.base_property
            .read_u8(BASE_PROPERTY_OCCUPATION_OFFSET)
    }

    /// Возвращает appellation ID из exact base-property offset `+0x154`.
    pub(crate) fn get_appellation_id(&self) -> u32 {
        self.base_property
            .read_u32(BASE_PROPERTY_APPELLATION_OFFSET)
    }

    /// Повторяет прямые honor-eliminate записи `CGame` в player base-owner.
    pub(crate) fn reset_honor_eliminate_info(&mut self, rank_mask: u32) {
        if rank_mask & 0x02 != 0 {
            self.base_property
                .write_u32(BASE_PROPERTY_WEEKS_HONOR_OFFSET, 0);
        }
        if rank_mask & 0x04 != 0 {
            self.base_property
                .write_u32(BASE_PROPERTY_MONTHS_HONOR_OFFSET, 0);
        }
        self.base_property
            .write_u32(BASE_PROPERTY_DAYS_HONOR_OFFSET, 0);
    }

    /// Возвращает текущее число элементов `m_listFriend`.
    pub(crate) fn friend_count(&self) -> usize {
        self.friends.len()
    }

    /// Заимствует имя friend-элемента в исходном list-order.
    pub(crate) fn friend_name(&self, index: usize) -> Option<&[u8]> {
        self.friends.get(index).map(|friend| friend.name.as_slice())
    }

    /// Присваивает `bOnline` выбранного friend-элемента.
    pub(crate) fn set_friend_online(&mut self, index: usize, online: bool) -> bool {
        let Some(friend) = self.friends.get_mut(index) else {
            return false;
        };
        friend.online = online;
        true
    }

    /// Обновляет organizing-состояние игрока через переданного владельца.
    pub(crate) fn set_player_organizing<U: PlayerOrganizingUpdater>(
        &mut self,
        updater: &mut U,
    ) -> Result<(), PlayerOrganizingUpdateError> {
        let player_id = self.get_id();
        updater.set_player_organizing(player_id, &mut self.organizing)
    }

    /// Присваивает state унаследованного shape-owner-а.
    pub(crate) const fn set_state(&mut self, state: u16) {
        self.move_shape_base.set_state(state);
    }

    /// Замораживает все пятнадцать goods-owner-ов для одного DB lifecycle.
    pub(crate) fn db_goods_projection(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<PlayerGoodsDbProjection, GoodsDbSnapshotBlock> {
        Ok(PlayerGoodsDbProjection {
            packet: self.packet.db_save_entries(registry)?,
            equipment: self.equipment.db_save_entries(registry)?,
            hand: self.hand.db_save_entries(registry)?,
            wallet: self.wallet.db_save_entries(registry)?,
            yuan_bao: self.yuan_bao.db_save_entries(registry)?,
            ji_fen: self.ji_fen.db_save_entries(registry)?,
            bank: self.bank.db_save_entries(registry)?,
            depot: self.depot.db_save_entries(registry)?,
            fairy: self.fairy.db_save_entries(registry)?,
            battle_fairy: self.battle_fairy.db_save_entries(registry)?,
            auction_goods: self.auction_goods_container.db_save_entries(registry)?,
            auction_wallet: self.auction_wallet.db_save_entries(registry)?,
            auction: self.auction_container.db_save_entries(registry)?,
            ci_qing: self.ci_qing.db_save_entries(registry)?,
            compose_ci_qing: self.compose_ci_qing.db_save_entries(registry)?,
        })
    }

    /// Материализует все временные массивы, которые DB-owner читал синхронно.
    pub(crate) fn db_projection(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<PlayerDbProjection<'_>, PlayerDbProjectionBlock> {
        let country = self
            .country
            .ok_or(PlayerDbProjectionBlock::UninitializedField {
                field: "m_btCountry",
            })?;
        let contribute = self
            .contribute
            .ok_or(PlayerDbProjectionBlock::UninitializedField {
                field: "m_lContribute",
            })?;

        let declared = usize::try_from(self.variable_data_length).map_err(|_| {
            PlayerDbProjectionBlock::NegativeVariableDataLength {
                value: self.variable_data_length,
            }
        })?;
        let available = self.variable_data.as_deref().unwrap_or_default();
        let variable_data = available.get(..declared).ok_or(
            PlayerDbProjectionBlock::VariableDataShorterThanDeclared {
                declared,
                available: available.len(),
            },
        )?;

        let mut hot_keys = [0u32; 24];
        for (index, hot_key) in hot_keys.iter_mut().enumerate() {
            *hot_key = self
                .base_property
                .read_u32(BASE_PROPERTY_HOT_KEYS_OFFSET + index * 4);
        }
        let skills = self
            .new_skills
            .iter()
            .map(|skill| PlayerAbilitySkill {
                id: skill.skill_id,
                level: skill.level,
            })
            .collect();
        let friend_names = self
            .friends
            .iter()
            .enumerate()
            .map(|(friend_index, friend)| {
                PlayerFriendName::from_legacy_bytes(&friend.name).map_err(
                    |EmbeddedFriendNameNul { offset }| {
                        PlayerDbProjectionBlock::FriendNameContainsNul {
                            friend_index,
                            offset,
                        }
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let things = self
            .daily_things
            .iter()
            .map(|thing| DbPlayerThing {
                tid: thing.thing_id,
                count: thing.count,
                max_count: thing.max_count,
                point: thing.point,
            })
            .collect();
        let quests = self
            .player_quests
            .iter()
            .map(|(key, quest)| {
                (
                    *key,
                    PlayerQuestSaveEntry {
                        quest_id: quest.quest_id,
                        complete: quest.complete,
                    },
                )
            })
            .collect();

        Ok(PlayerDbProjection {
            player: self,
            country,
            contribute,
            goods: self.db_goods_projection(registry)?,
            hot_keys,
            skills,
            friend_names,
            things,
            quests,
            variable_data,
        })
    }

    /// Присваивает region ID через унаследованный shape-owner.
    pub(crate) const fn set_region_id(&mut self, region_id: i32) {
        self.move_shape_base.set_region_id(region_id);
    }

    /// Кодирует полный достигнутый player-wire и затем пересчитывает property.
    pub(crate) fn add_to_byte_array<U: PlayerOrganizingUpdater>(
        &mut self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        updater: &mut U,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.add_to_byte_array_before_containers(destination, include_child)?;
        if include_child {
            let _ = self.add_containers_to_byte_array(destination, registry)?;
            let _ = self.add_to_byte_array_after_containers(destination, updater)?;
        }
        self.update_property(coefficients)?;
        Ok(true)
    }

    /// Декодирует полный достигнутый player-wire и затем пересчитывает property.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.decord_from_byte_array_before_containers(source, cursor, include_child)?;
        if include_child {
            let _ = self.decord_containers_from_byte_array(source, cursor, registry)?;
            let _ = self.decord_from_byte_array_after_containers(source, cursor)?;
        }
        self.update_property(coefficients)?;
        Ok(true)
    }

    /// Пересчитывает доказанный WorldServer `tagProperty` из base и setup.
    pub(crate) fn update_property(
        &mut self,
        coefficients: &PlayerPropertyCoefficients,
    ) -> Result<(), PlayerCodecError> {
        let strength = self.base_property.read_u32(BASE_PROPERTY_STR_OFFSET);
        let dexterity = self.base_property.read_u32(BASE_PROPERTY_DEX_OFFSET);
        let constitution = self.base_property.read_u32(BASE_PROPERTY_CON_OFFSET);
        let intelligence = self.base_property.read_u32(BASE_PROPERTY_INT_OFFSET);
        let saved_dexterity = dexterity as f32;
        let saved_intelligence = intelligence as f32;

        self.property.write_u32(PROPERTY_STR_OFFSET, strength);
        self.property.write_u32(PROPERTY_DEX_OFFSET, dexterity);
        self.property.write_u32(PROPERTY_CON_OFFSET, constitution);
        self.property.write_u32(PROPERTY_INT_OFFSET, intelligence);

        let occupation = self.base_property.read_u8(BASE_PROPERTY_OCCUPATION_OFFSET);
        let occupation = usize::from(occupation);
        if occupation >= 3 {
            // BLOCKED_MISSING_FACT: original индексировал соседнюю static
            // память за `float[3]`; safe Rust не назначает ей коэффициент.
            return Err(
                PlayerCodecError::OccupationOutsidePropertyCoefficientRange {
                    occupation: occupation as u8,
                },
            );
        }

        let max_hp = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_HP_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                constitution,
                coefficients.con_to_max_hp[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_HP_OFFSET, max_hp);

        let max_mp = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_MP_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                intelligence,
                coefficients.int_to_max_mp[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_MP_OFFSET, max_mp);
        self.property.write_u16(
            PROPERTY_MAX_YP_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_MAX_YP_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_MAX_RP_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_MAX_RP_OFFSET),
        );

        let min_attack = self
            .base_property
            .read_u32(BASE_PROPERTY_MIN_ATK_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                dexterity,
                coefficients.dex_to_min_attack[occupation],
            ));
        self.property.write_u32(PROPERTY_MIN_ATK_OFFSET, min_attack);

        let max_attack = self
            .base_property
            .read_u32(BASE_PROPERTY_MAX_ATK_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                strength,
                coefficients.str_to_max_attack[occupation],
            ));
        self.property.write_u32(PROPERTY_MAX_ATK_OFFSET, max_attack);
        self.property.write_u16(
            PROPERTY_HIT_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_HIT_OFFSET),
        );
        let burden = self
            .base_property
            .read_u16(BASE_PROPERTY_BURDEN_OFFSET)
            .wrapping_add(legacy_x87_i64_low_u16(
                strength,
                coefficients.str_to_burden[occupation],
            ));
        self.property.write_u16(PROPERTY_BURDEN_OFFSET, burden);
        self.property.write_u16(
            PROPERTY_CCH_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_CCH_OFFSET),
        );

        let defense = self
            .base_property
            .read_u32(BASE_PROPERTY_DEF_OFFSET)
            .wrapping_add(legacy_x87_i32_bits(
                constitution,
                coefficients.con_to_defense[occupation],
            ));
        self.property.write_u32(PROPERTY_DEF_OFFSET, defense);
        self.property.write_u16(
            PROPERTY_DODGE_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_DODGE_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_ATC_SPEED_OFFSET,
            self.base_property.read_u16(BASE_PROPERTY_ATC_SPEED_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_HP_RECOVER_SPEED_OFFSET,
            self.base_property
                .read_u16(BASE_PROPERTY_HP_RECOVER_SPEED_OFFSET),
        );
        self.property.write_u16(
            PROPERTY_MP_RECOVER_SPEED_OFFSET,
            self.base_property
                .read_u16(BASE_PROPERTY_MP_RECOVER_SPEED_OFFSET),
        );

        let element_resistant = self
            .base_property
            .read_u32(BASE_PROPERTY_ELEMENT_RESISTANT_OFFSET)
            .wrapping_add(legacy_x87_f32_i32_bits(
                saved_intelligence,
                coefficients.int_to_resistant[occupation],
            ));
        self.property
            .write_u32(PROPERTY_ELEMENT_RESISTANT_OFFSET, element_resistant);
        self.property.write_u32(
            PROPERTY_ELEMENT_MODIFY_OFFSET,
            legacy_x87_f32_i32_bits(saved_intelligence, coefficients.int_to_element[occupation]),
        );
        self.property.write_u16(
            PROPERTY_RE_ANK_OFFSET,
            legacy_x87_f32_i64_low_u16(saved_dexterity, coefficients.dex_to_stiff[occupation]),
        );
        Ok(())
    }

    /// Кодирует непрерывный доказанный префикс до первого container-owner-а.
    pub(crate) fn add_to_byte_array_before_containers(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self
            .move_shape_base
            .add_shape_to_byte_array(destination, include_child);
        if !include_child {
            return Ok(true);
        }

        destination.extend_from_slice(&self.base_property.wire);
        append_player_c_string(destination, &self.base_property.account);
        append_player_c_string(destination, &self.base_property.title);
        destination.extend_from_slice(&self.property.wire);
        destination.extend_from_slice(&self.team_id.to_le_bytes());
        self.add_byte_ci_qing(destination)?;

        append_player_count(destination, "m_listNewSkillID", self.new_skills.len())?;
        for skill in &self.new_skills {
            destination.extend_from_slice(&skill.skill_id.to_le_bytes());
            destination.extend_from_slice(&skill.level.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_vExStates",
            self.move_shape_base.ex_states().len(),
        )?;
        destination.extend_from_slice(self.move_shape_base.ex_states());

        append_player_count(destination, "m_listFriend", self.friends.len())?;
        for friend in &self.friends {
            append_player_c_string(destination, &friend.name);
            destination.push(u8::from(friend.online));
        }
        self.add_byte_array_lei_ting(destination)?;
        Ok(true)
    }

    /// Декодирует достигнутый префикс и оставляет cursor перед `m_cHand`.
    pub(crate) fn decord_from_byte_array_before_containers(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, PlayerCodecError> {
        self.new_skills.clear();
        self.move_shape_base.clear_ex_states();
        self.friends.clear();
        self.daily_things.clear();
        self.uncreated_pets.clear();
        self.uncreated_carriage.original_name.clear();
        self.uncreated_carriage.hp = 0;
        self.recreate_carriage = false;

        let _ = self
            .move_shape_base
            .decord_shape_from_byte_array(source, cursor, include_child)?;
        if !include_child {
            return Ok(true);
        }

        self.base_property.wire = read_player_array(source, cursor, "m_BaseProperty[0x194]")?;
        self.base_property.account =
            read_player_c_string(source, cursor, "m_BaseProperty.strAccount", 0x100)?;
        self.base_property.title =
            read_player_c_string(source, cursor, "m_BaseProperty.strTitle", 0x100)?;
        self.property.wire = read_player_array(source, cursor, "m_Property[0x9c]")?;
        self.team_id = read_player_i32(source, cursor, "m_lTeamID")?;
        self.de_byte_ci_qing(source, cursor)?;

        let skill_count = read_player_i32(source, cursor, "m_listNewSkillID count")?;
        if skill_count > 0 {
            for _ in 0..skill_count {
                let bytes = read_player_array::<4>(source, cursor, "CPlayer::tagSkill")?;
                self.new_skills.push_back(PlayerSkill {
                    skill_id: u16::from_le_bytes([bytes[0], bytes[1]]),
                    level: u16::from_le_bytes([bytes[2], bytes[3]]),
                });
            }
        }

        let ex_state_length = read_player_i32(source, cursor, "m_vExStates length")?;
        if ex_state_length < 0 {
            // BLOCKED_MISSING_FACT: старый signed length уходил в pointer
            // arithmetic и `_AddToByteArray`; результат для high-bit wire не
            // определён согласованным псевдокодом.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_vExStates length",
                value: ex_state_length,
            });
        }
        if ex_state_length != 0 {
            let length = ex_state_length as usize;
            let states = read_player_slice(source, cursor, "m_vExStates bytes", length)?;
            self.move_shape_base.set_ex_states(states);
        }

        let friend_count = read_player_i32(source, cursor, "m_listFriend count")?;
        if friend_count > 0 {
            for _ in 0..friend_count {
                let name = read_player_c_string(source, cursor, "tagFriend.strName", 0x94)?;
                let online = read_player_u8(source, cursor, "tagFriend.bOnline")? != 0;
                self.friends.push_back(PlayerFriend { name, online });
            }
        }
        self.decode_byte_array_lei_ting(source, cursor)?;
        Ok(true)
    }

    /// Кодирует пятнадцать container-owner-ов и depot-password в exact wire-order.
    pub(crate) fn add_containers_to_byte_array(
        &mut self,
        destination: &mut Vec<u8>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, PlayerCodecError> {
        let _ = self.hand.serialize(destination, true, registry)?;
        let _ = self.equipment.serialize(destination, true, registry)?;
        let _ = self.packet.serialize(destination, true, registry)?;
        let _ = self
            .auction_goods_container
            .serialize(destination, true, registry)?;
        let _ = self
            .auction_container
            .serialize(destination, true, registry)?;
        let _ = self.wallet.serialize(destination, true)?;
        let _ = self.auction_wallet.serialize(destination, true)?;
        let _ = self.yuan_bao.serialize(destination, true)?;
        let _ = self.ji_fen.serialize(destination, true)?;
        append_player_c_string(destination, &self.depot_password);
        let _ = self.bank.serialize(destination, true)?;
        let _ = self.depot.serialize(destination, true, registry)?;
        let _ = self.fairy.serialize(destination, true, registry)?;
        let _ = self.battle_fairy.serialize(destination, true, registry)?;
        let _ = self.ci_qing.serialize(destination, true, registry)?;
        let _ = self
            .compose_ci_qing
            .serialize(destination, true, registry)?;
        Ok(true)
    }

    /// Декодирует container-сегмент с точными `Release` и limit/volume setup.
    pub(crate) fn decord_containers_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, PlayerCodecError> {
        self.hand.release();
        self.hand.set_goods_amount_limit(1);
        let _ = self.hand.unserialize(source, cursor, registry)?;

        self.equipment.release();
        let _ = self.equipment.unserialize(source, cursor, true, registry)?;

        self.packet.release();
        self.packet.set_container_volume_2d(8, 0x0C);
        let _ = self.packet.unserialize(source, cursor, registry)?;

        self.auction_goods_container.release();
        self.auction_goods_container.set_container_volume(0x12);
        let _ = self
            .auction_goods_container
            .unserialize(source, cursor, registry)?;

        self.auction_container.release();
        self.auction_container.set_container_volume(2);
        let _ = self
            .auction_container
            .unserialize(source, cursor, registry)?;

        self.wallet.release();
        let _ = self.wallet.unserialize(source, cursor, true, registry)?;

        self.auction_wallet.release();
        let _ = self
            .auction_wallet
            .unserialize(source, cursor, true, registry)?;

        self.yuan_bao.release();
        let _ = self.yuan_bao.unserialize(source, cursor, true, registry)?;

        self.ji_fen.release();
        let _ = self.ji_fen.unserialize(source, cursor, true, registry)?;

        self.depot_password = read_player_c_string(source, cursor, "m_strDepotPassword", 0x6C)?;

        self.bank.release();
        let _ = self.bank.unserialize(source, cursor, true, registry)?;

        self.depot.release();
        self.depot.set_container_volume(0xA1);
        let _ = self.depot.unserialize(source, cursor, true, registry)?;

        self.fairy.release();
        self.fairy.set_container_volume(0x0E);
        let _ = self.fairy.unserialize(source, cursor, true, registry)?;

        self.battle_fairy.release();
        self.battle_fairy.set_container_volume(0x11);
        let _ = self
            .battle_fairy
            .unserialize(source, cursor, true, registry)?;

        self.ci_qing.release();
        self.ci_qing.set_container_volume(8);
        let _ = self.ci_qing.unserialize(source, cursor, registry)?;

        self.compose_ci_qing.release();
        self.compose_ci_qing.set_container_volume(3);
        let _ = self.compose_ci_qing.unserialize(source, cursor, registry)?;
        Ok(true)
    }

    /// Кодирует достигнутый suffix после containers и оставляет owner готовым к `UpdateProperty`.
    pub(crate) fn add_to_byte_array_after_containers<U: PlayerOrganizingUpdater>(
        &mut self,
        destination: &mut Vec<u8>,
        updater: &mut U,
    ) -> Result<bool, PlayerCodecError> {
        destination.extend_from_slice(&self.variable_num.to_le_bytes());
        destination.extend_from_slice(&self.variable_data_length.to_le_bytes());
        if let Some(variable_data) = &self.variable_data {
            if self.variable_data_length < 0 {
                return Err(PlayerCodecError::NegativeLength {
                    field: "m_lVariableDataLength",
                    value: self.variable_data_length,
                });
            }
            let declared = self.variable_data_length as usize;
            if variable_data.len() < declared {
                return Err(PlayerCodecError::BufferShorterThanDeclaredLength {
                    field: "m_pVariableData",
                    declared,
                    available: variable_data.len(),
                });
            }
            destination.extend_from_slice(&variable_data[..declared]);
        }

        destination.extend_from_slice(&self.silience_time.to_le_bytes());
        destination.push(u8::from(self.move_shape_base.is_god()));
        destination.extend_from_slice(&self.murderer_time.to_le_bytes());
        destination.extend_from_slice(&self.fight_state_count.to_le_bytes());

        append_player_count(destination, "m_vUncreatedPets", self.uncreated_pets.len())?;
        for pet in &self.uncreated_pets {
            append_player_c_string(destination, &pet.original_name);
            destination.extend_from_slice(&pet.hp.to_le_bytes());
            destination.extend_from_slice(&pet.level.to_le_bytes());
            destination.extend_from_slice(&pet.experience.to_le_bytes());
        }

        append_player_c_string(destination, &self.uncreated_carriage.original_name);
        append_player_c_string(destination, &self.uncreated_carriage.carriage_script);
        destination.extend_from_slice(&self.uncreated_carriage.hp.to_le_bytes());
        destination.push(u8::from(self.recreate_carriage));
        destination.push(u8::from(self.login));
        destination.extend_from_slice(&self.city_war_died_state_time.to_le_bytes());
        let _ = self.add_quest_data_to_byte_array(destination)?;
        let country = self
            .country
            .ok_or(PlayerCodecError::UninitializedWireField {
                field: "m_btCountry",
            })?;
        destination.push(country);
        let contribute = self
            .contribute
            .ok_or(PlayerCodecError::UninitializedWireField {
                field: "m_lContribute",
            })?;
        destination.extend_from_slice(&contribute.to_le_bytes());
        destination.extend_from_slice(&self.jjc_data);
        destination.push(u8::from(self.jjc_pk_state));
        let _ = self.add_org_sys_to_byte_array(destination, updater)?;
        append_player_fixed_c_string(destination, "m_strSessionID", &self.session_id, 0x40)?;
        Ok(true)
    }

    /// Декодирует WorldServer suffix после containers; `UpdateProperty` остаётся следующим owner-ом.
    pub(crate) fn decord_from_byte_array_after_containers(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.variable_num = read_player_i32(source, cursor, "m_lVariableNum")?;
        self.variable_data_length = read_player_i32(source, cursor, "m_lVariableDataLength")?;
        if self.variable_data_length < 0 {
            // BLOCKED_MISSING_FACT: original передавал high-bit length в
            // `operator new` и безразмерный `_GetBufferFromByteArray`.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_lVariableDataLength",
                value: self.variable_data_length,
            });
        }
        let variable_data = read_player_slice(
            source,
            cursor,
            "m_pVariableData",
            self.variable_data_length as usize,
        )?;
        self.variable_data = Some(variable_data.to_vec());

        self.silience_time = read_player_i32(source, cursor, "m_lSilienceTime")?;
        let is_god = read_player_u8(source, cursor, "m_bIsGod")? != 0;
        self.move_shape_base.set_is_god(is_god);
        self.murderer_time = read_player_u32(source, cursor, "m_dwMurdererTime")?;
        let _wire_fight_state_count = read_player_i32(source, cursor, "m_lFightStateCount wire")?;
        self.fight_state_count = 2;

        let pet_count = read_player_i32(source, cursor, "m_vUncreatedPets count")?;
        if pet_count < 0 {
            // BLOCKED_MISSING_FACT: исходный `for (count; count != 0; --count)`
            // для отрицательного значения уходит в signed overflow/overread.
            return Err(PlayerCodecError::NegativeLength {
                field: "m_vUncreatedPets count",
                value: pet_count,
            });
        }
        for _ in 0..pet_count {
            self.uncreated_pets.push(PlayerPetInformation {
                original_name: read_player_c_string(
                    source,
                    cursor,
                    "tagPetInformation.strOriginalName",
                    0x94,
                )?,
                hp: read_player_u32(source, cursor, "tagPetInformation.dwHp")?,
                level: read_player_u32(source, cursor, "tagPetInformation.dwLevel")?,
                experience: read_player_u32(source, cursor, "tagPetInformation.dwExperience")?,
            });
        }

        self.uncreated_carriage.original_name =
            read_player_c_string(source, cursor, "tagCarriageInfo.strOriginalName", 0x94)?;
        self.uncreated_carriage.carriage_script =
            read_player_c_string(source, cursor, "tagCarriageInfo.strCarriageScript", 0x94)?;
        self.uncreated_carriage.hp = read_player_u32(source, cursor, "tagCarriageInfo.dwHp")?;
        self.recreate_carriage = read_player_u8(source, cursor, "m_bReCreateCarriage")? != 0;
        self.login = read_player_u8(source, cursor, "m_bLogin")? != 0;
        self.city_war_died_state_time = read_player_i32(source, cursor, "m_lCityWarDiedStateTime")?;
        let _ = self.decord_quest_data_from_byte_array(source, cursor)?;
        self.country = Some(read_player_u8(source, cursor, "m_btCountry")?);
        self.contribute = Some(read_player_i32(source, cursor, "m_lContribute")?);
        self.jjc_data = read_player_array(source, cursor, "m_jjcData[0x10]")?;
        self.jjc_pk_state = read_player_u8(source, cursor, "bJJcPkState")? != 0;
        self.session_id = read_player_c_string(source, cursor, "m_strSessionID", 0x40)?;
        Ok(true)
    }

    /// Обновляет и кодирует organization-блок в точном WorldServer wire-order.
    pub(crate) fn add_org_sys_to_byte_array<U: PlayerOrganizingUpdater>(
        &mut self,
        destination: &mut Vec<u8>,
        updater: &mut U,
    ) -> Result<bool, PlayerCodecError> {
        updater.set_player_organizing(self.get_id(), &mut self.organizing)?;

        let organizing = &self.organizing;
        destination.extend_from_slice(&organizing.faction_id.to_le_bytes());
        if organizing.faction_id <= 0 {
            return Ok(true);
        }

        destination.extend_from_slice(&organizing.faction_logo_id.to_le_bytes());
        destination.extend_from_slice(&organizing.faction_level.to_le_bytes());
        destination.extend_from_slice(&organizing.faction_experience.to_le_bytes());
        destination.extend_from_slice(&organizing.force.to_le_bytes());
        destination.extend_from_slice(&u32::from(organizing.faction_contribute).to_le_bytes());
        append_player_c_string(destination, &organizing.faction_name);
        append_player_c_string(destination, &organizing.faction_title);
        destination.extend_from_slice(&organizing.faction_master_id.to_le_bytes());
        destination.extend_from_slice(&organizing.union_id.to_le_bytes());
        destination.extend_from_slice(&organizing.union_master_id.to_le_bytes());

        append_player_count(
            destination,
            "m_EnemyFactions",
            organizing.enemy_factions.len(),
        )?;
        for &faction_id in &organizing.enemy_factions {
            destination.extend_from_slice(&faction_id.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_CityWarEnemyFactions",
            organizing.city_war_enemy_factions.len(),
        )?;
        for &faction_id in &organizing.city_war_enemy_factions {
            destination.extend_from_slice(&faction_id.to_le_bytes());
        }

        append_player_count(
            destination,
            "m_OwnedRegions",
            organizing.owned_regions.len(),
        )?;
        for owned_region in &organizing.owned_regions {
            destination.extend_from_slice(owned_region);
        }
        Ok(true)
    }

    /// Дописывает tattoo ID в unsigned set-order исходного `AddByteCiQing`.
    pub(crate) fn add_byte_ci_qing(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerCodecError> {
        append_player_count(destination, "m_setCiQingList", self.ci_qing_ids.len())?;
        for &id in &self.ci_qing_ids {
            destination.extend_from_slice(&id.to_le_bytes());
        }
        Ok(())
    }

    /// Очищает и восстанавливает tattoo set, сохраняя частичные вставки.
    pub(crate) fn de_byte_ci_qing(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerCodecError> {
        self.ci_qing_ids.clear();
        let count = read_player_u32(source, cursor, "m_setCiQingList count")?;
        for _ in 0..count {
            let id = read_player_u32(source, cursor, "m_setCiQingList value")?;
            self.ci_qing_ids.insert(id);
        }
        Ok(())
    }

    /// Дописывает LeiTing scalar и `tagThing` в исходном deque-order.
    pub(crate) fn add_byte_array_lei_ting(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerCodecError> {
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_FY_ENERGY_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET)
                .to_le_bytes(),
        );
        destination.extend_from_slice(
            &self
                .base_property
                .read_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET)
                .to_le_bytes(),
        );
        append_player_count(destination, "m_listThing", self.daily_things.len())?;
        for thing in &self.daily_things {
            destination.extend_from_slice(&thing.thing_id.to_le_bytes());
            destination.extend_from_slice(&thing.count.to_le_bytes());
            destination.extend_from_slice(&thing.max_count.to_le_bytes());
            destination.extend_from_slice(&thing.point.to_le_bytes());
        }
        Ok(())
    }

    /// Читает LeiTing scalar, затем очищает и наполняет deque по порядку.
    pub(crate) fn decode_byte_array_lei_ting(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<(), PlayerCodecError> {
        let enable_flags = read_player_u32(source, cursor, "dwfyenFlag")?;
        self.base_property
            .write_u32(BASE_PROPERTY_FY_ENABLE_FLAGS_OFFSET, enable_flags);
        let energy = read_player_u32(source, cursor, "dwfyEnergy")?;
        self.base_property
            .write_u32(BASE_PROPERTY_FY_ENERGY_OFFSET, energy);
        let stamp = read_player_u32(source, cursor, "dwLT60Stamp")?;
        self.base_property
            .write_u32(BASE_PROPERTY_LT_60_STAMP_OFFSET, stamp);
        let up_60_count = read_player_u16(source, cursor, "wLTUp60Cnt")?;
        self.base_property
            .write_u16(BASE_PROPERTY_LT_UP_60_COUNT_OFFSET, up_60_count);
        let remaining = read_player_u16(source, cursor, "wRemainJingLiDanCnt")?;
        self.base_property
            .write_u16(BASE_PROPERTY_REMAIN_JING_LI_DAN_COUNT_OFFSET, remaining);

        self.daily_things.clear();
        let count = read_player_u32(source, cursor, "m_listThing count")?;
        for _ in 0..count {
            self.daily_things.push_back(PlayerThing {
                thing_id: read_player_u16(source, cursor, "tagThing.wTID")?,
                count: read_player_u16(source, cursor, "tagThing.wCnt")?,
                max_count: read_player_u16(source, cursor, "tagThing.wMaxCnt")?,
                point: read_player_u16(source, cursor, "tagThing.wPoint")?,
            });
        }
        Ok(())
    }

    /// Дописывает mapped quest values в unsigned key-order исходной map.
    pub(crate) fn add_quest_data_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<bool, PlayerCodecError> {
        append_player_count(destination, "m_PlayerQuests", self.player_quests.len())?;
        for quest in self.player_quests.values() {
            destination.extend_from_slice(&quest.quest_id.to_le_bytes());
            destination.push(quest.complete);
        }
        Ok(true)
    }

    /// Очищает quest map и восстанавливает mapped value по его же quest ID.
    pub(crate) fn decord_quest_data_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<bool, PlayerCodecError> {
        self.player_quests.clear();
        let count = read_player_i32(source, cursor, "m_PlayerQuests count")?;
        if count > 0 {
            for _ in 0..count {
                let quest_id = read_player_u16(source, cursor, "tagPlayerQuest.wQuestID")?;
                let complete = read_player_u8(source, cursor, "tagPlayerQuest.byComplete")?;
                self.player_quests
                    .insert(quest_id, PlayerQuest { quest_id, complete });
            }
        }
        Ok(true)
    }

    /// Передаёт полный снимок игрока DB-owner-у и возвращает его точный итог.
    ///
    /// `snapshot` должен быть материализован из этой же player-сущности.
    pub(crate) async fn save_data<P, J, G>(
        &self,
        snapshot: &PlayerSaveSnapshot<'_, '_, '_>,
        connection: Option<&mut WorldTdsClient>,
        player_database: &mut P,
        jjc_database: &mut J,
        goods_database: &mut G,
    ) -> PlayerSaveOutcome
    where
        P: RsPlayerOwner,
        J: RsJjcSysOwner,
        G: DbGoodsOwner,
    {
        player_database
            .save_player(Some(snapshot), connection, jjc_database, goods_database)
            .await
    }
}

fn append_player_count(
    destination: &mut Vec<u8>,
    field: &'static str,
    count: usize,
) -> Result<(), PlayerCodecError> {
    let count = u32::try_from(count)
        .map_err(|_| PlayerCodecError::CollectionLengthOutsideLegacyRange { field, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_player_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let visible = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..visible]);
    destination.push(0);
}

fn append_player_fixed_c_string(
    destination: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
    capacity: usize,
) -> Result<(), PlayerCodecError> {
    let length = value
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(value.len());
    if length >= capacity {
        return Err(PlayerCodecError::StringOutsideLegacyCapacity {
            field,
            length,
            capacity,
        });
    }
    append_player_c_string(destination, value);
    Ok(())
}

/// Возвращает точный усечённый модуль `u32 * binary32` для x87 integer-convert.
///
/// Safe `softfloat-wrapper` не предоставляет `extFloat80`, а найденные
/// extFloat80 bindings требуют широкого `unsafe` FFI. Здесь общий soft-float
/// runtime не нужен: произведение 32-битного integer и 24-битной significand
/// содержит не более 56 значащих бит и потому точно помещается в 64-битную
/// x87 significand. Разбор IEEE-754 bits сохраняет это единственное достигнутое
/// действие без округления через недостаточный `f64`.
fn legacy_x87_truncated_magnitude(
    value: u32,
    coefficient: f32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    let bits = coefficient.to_bits();
    let negative = bits >> 31 != 0;
    let exponent_bits = (bits >> 23) & 0xFF;
    let fraction = bits & 0x7F_FFFF;
    if exponent_bits == 0xFF {
        return None;
    }

    let (significand, exponent) = if exponent_bits == 0 {
        (u128::from(fraction), -149)
    } else {
        (
            u128::from((1 << 23) | fraction),
            exponent_bits as i32 - 127 - 23,
        )
    };
    let product = u128::from(value) * significand;
    legacy_x87_truncated_parts(negative, product, exponent, positive_limit, negative_limit)
}

fn legacy_binary32_parts(value: f32) -> Option<(bool, u128, i32)> {
    let bits = value.to_bits();
    let negative = bits >> 31 != 0;
    let exponent_bits = (bits >> 23) & 0xFF;
    let fraction = bits & 0x7F_FFFF;
    if exponent_bits == 0xFF {
        return None;
    }
    if exponent_bits == 0 {
        Some((negative, u128::from(fraction), -149))
    } else {
        Some((
            negative,
            u128::from((1 << 23) | fraction),
            exponent_bits as i32 - 127 - 23,
        ))
    }
}

fn legacy_x87_truncated_f32_magnitude(
    value: f32,
    coefficient: f32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    let (value_negative, value_significand, value_exponent) = legacy_binary32_parts(value)?;
    let (coefficient_negative, coefficient_significand, coefficient_exponent) =
        legacy_binary32_parts(coefficient)?;
    legacy_x87_truncated_parts(
        value_negative != coefficient_negative,
        value_significand * coefficient_significand,
        value_exponent + coefficient_exponent,
        positive_limit,
        negative_limit,
    )
}

fn legacy_x87_truncated_parts(
    negative: bool,
    product: u128,
    exponent: i32,
    positive_limit: u128,
    negative_limit: u128,
) -> Option<(bool, u128)> {
    if product == 0 {
        return Some((negative, 0));
    }
    let limit = if negative {
        negative_limit
    } else {
        positive_limit
    };

    let magnitude = if exponent >= 0 {
        let shift = exponent as u32;
        if shift >= u128::BITS || product > (limit >> shift) {
            return None;
        }
        product << shift
    } else {
        let shift = exponent.unsigned_abs();
        if shift >= u128::BITS {
            0
        } else {
            product >> shift
        }
    };
    (magnitude <= limit).then_some((negative, magnitude))
}

/// Эффект x87 `fistp dword` с RC=truncate, включая integer-indefinite.
fn legacy_x87_i32_bits(value: u32, coefficient: f32) -> u32 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_magnitude(value, coefficient, i32::MAX as u128, 1_u128 << 31)
    else {
        return i32::MIN as u32;
    };
    if negative {
        0_u32.wrapping_sub(magnitude as u32)
    } else {
        magnitude as u32
    }
}

/// Младшие 16 бит результата MSVC `__ftol2` после x87 truncation.
fn legacy_x87_i64_low_u16(value: u32, coefficient: f32) -> u16 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_magnitude(value, coefficient, i64::MAX as u128, 1_u128 << 63)
    else {
        return 0;
    };
    let bits = if negative {
        0_u64.wrapping_sub(magnitude as u64)
    } else {
        magnitude as u64
    };
    bits as u16
}

fn legacy_x87_f32_i32_bits(value: f32, coefficient: f32) -> u32 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_f32_magnitude(value, coefficient, i32::MAX as u128, 1_u128 << 31)
    else {
        return i32::MIN as u32;
    };
    if negative {
        0_u32.wrapping_sub(magnitude as u32)
    } else {
        magnitude as u32
    }
}

fn legacy_x87_f32_i64_low_u16(value: f32, coefficient: f32) -> u16 {
    let Some((negative, magnitude)) =
        legacy_x87_truncated_f32_magnitude(value, coefficient, i64::MAX as u128, 1_u128 << 63)
    else {
        return 0;
    };
    let bits = if negative {
        0_u64.wrapping_sub(magnitude as u64)
    } else {
        magnitude as u64
    };
    bits as u16
}

fn read_player_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, PlayerCodecError> {
    Ok(i32::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, PlayerCodecError> {
    Ok(u32::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u16(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u16, PlayerCodecError> {
    Ok(u16::from_le_bytes(read_player_array(
        source, cursor, field,
    )?))
}

fn read_player_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, PlayerCodecError> {
    Ok(read_player_array::<1>(source, cursor, field)?[0])
}

fn read_player_c_string(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
    capacity: usize,
) -> Result<Vec<u8>, PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let searchable = available.min(capacity);
    let Some(bytes) = source.get(offset..offset.saturating_add(searchable)) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 1,
            available,
        });
    };
    if let Some(length) = bytes.iter().position(|&byte| byte == 0) {
        *cursor = offset + length + 1;
        return Ok(bytes[..length].to_vec());
    }

    // BLOCKED_MISSING_FACT: legacy helper писал до NUL в фиксированный stack
    // buffer. Ни overflow, ни чтение за source безопасный Rust не имитирует.
    Err(PlayerCodecError::UnterminatedString {
        field,
        offset,
        capacity,
        available,
    })
}

fn read_player_slice<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    field: &'static str,
    length: usize,
) -> Result<&'a [u8], PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(length) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: length,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

fn read_player_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], PlayerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // BLOCKED_MISSING_FACT: legacy helper не получал длину source и читал
        // дальше. Safe Rust останавливает только эту локальную границу.
        return Err(PlayerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: N,
            available,
        });
    };
    *cursor = end;
    Ok(bytes
        .try_into()
        .expect("slice содержит ровно запрошенное число байт"))
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp

// ============================================================================
// FUNCTION: CPlayer::ClearOwnedRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.h:533
// RVA: 0x00033B50
// ADDRESS: 00433b50
// PROTOTYPE: void __thiscall ClearOwnedRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::GetMoney
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:786
// RVA: 0x0005AEA0
// ADDRESS: 0045aea0
// PROTOTYPE: ulong __thiscall GetMoney(void)
//
// IMPLEMENTED_OWNER: `money` делегирует точному `CWallet::GetGoldCoinsAmount`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFairyContainerEnabled
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:947
// RVA: 0x0005AEB0
// ADDRESS: 0045aeb0
// PROTOTYPE: void __thiscall SetFairyContainerEnabled(bool param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetFosterNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:952
// RVA: 0x0005AEC0
// ADDRESS: 0045aec0
// PROTOTYPE: void __thiscall SetFosterNum(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::SetHatcherNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:959
// RVA: 0x0005AEE0
// ADDRESS: 0045aee0
// PROTOTYPE: void __thiscall SetHatcherNum(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateProperty
// STATUS: VERIFIED_DISASSEMBLY, IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:312
// RVA: 0x0005AFC0
// ADDRESS: 0045afc0
// PROTOTYPE: void __thiscall UpdateProperty(void)
//
// Реализация находится в `CPlayer::update_property`; full codec вызывает её
// после encoder/decoder даже при `include_child=false`.

// ============================================================================
// FUNCTION: CPlayer::ReSetHonorElimilateNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:1009
// RVA: 0x0005B260
// ADDRESS: 0045b260
// PROTOTYPE: bool __thiscall ReSetHonorElimilateNum(tagTime param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::AddByteArrayLeiTing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:1129
// RVA: 0x0005B690
// ADDRESS: 0045b690
// PROTOTYPE: void __thiscall AddByteArrayLeiTing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED выше; пять scalar, unsigned count и восемь bytes каждого
// `tagThing` сохраняются в исходном deque-order.

// ============================================================================
// FUNCTION: CPlayer::AddOrgSysToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:482
// RVA: 0x0005B870
// ADDRESS: 0045b870
// PROTOTYPE: bool __thiscall AddOrgSysToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Реализация находится в `CPlayer::add_org_sys_to_byte_array`; singleton
// organizing-controller передаётся через `PlayerOrganizingUpdater`.

// ============================================================================
// FUNCTION: CPlayer::CheckGoodsInPacket
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:766
// RVA: 0x0005BA90
// ADDRESS: 0045ba90
// PROTOTYPE: long __thiscall CheckGoodsInPacket(char * param_1)
//
// IMPLEMENTED выше; locked-фильтр остаётся во второй packet `Find` фазе.

// ============================================================================
// FUNCTION: CPlayer::AddByteCiQing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:985
// RVA: 0x0005BB80
// ADDRESS: 0045bb80
// PROTOTYPE: void __thiscall AddByteCiQing(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED выше; `BTreeSet<u32>` заменяет только STL tree/storage.

// ============================================================================
// FUNCTION: CPlayer::AddQuestDataToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:808
// RVA: 0x0005BC10
// ADDRESS: 0045bc10
// PROTOTYPE: bool __thiscall AddQuestDataToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// IMPLEMENTED выше; mapped values идут по unsigned key-order и normal tail
// возвращает `true`.

// ============================================================================
// FUNCTION: CPlayer::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:343
// RVA: 0x0005BD10
// ADDRESS: 0045bd10
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, bool param_2)
//
// Полная реализация находится в `CPlayer::add_to_byte_array`; partial owner-ы
// сохранены отдельными методами только по исходным wire-границам.

// ============================================================================
// FUNCTION: CPlayer::UpdateFactionInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:715
// RVA: 0x0005C1D0
// ADDRESS: 0045c1d0
// PROTOTYPE: void __thiscall UpdateFactionInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:849
// RVA: 0x0005D1C0
// ADDRESS: 0045d1c0
// PROTOTYPE: int __thiscall ChangeName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DeByteCiQing
// STATUS: VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:997
// RVA: 0x0005D9E0
// ADDRESS: 0045d9e0
// PROTOTYPE: void __thiscall DeByteCiQing(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше. Exact `0x0045DA1D..0x0045DA49` подтверждает unsigned
// count, source load каждого `u32` и передачу именно этого value в set insert.

// ============================================================================
// FUNCTION: CPlayer::AddOwnedRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:758
// RVA: 0x0005DD10
// ADDRESS: 0045dd10
// PROTOTYPE: void __thiscall AddOwnedRegion(long param_1, ushort param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::UpdateLeiTing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:1053
// RVA: 0x0005DD80
// ADDRESS: 0045dd80
// PROTOTYPE: void __thiscall UpdateLeiTing(ulong param_1, tm * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecodeByteArrayLeiTing
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:1146
// RVA: 0x0005DE40
// ADDRESS: 0045de40
// PROTOTYPE: void __thiscall DecodeByteArrayLeiTing(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше; scalar mutations предшествуют clear deque, затем count и
// восемь bytes каждого `tagThing` читаются последовательно.

// ============================================================================
// FUNCTION: CPlayer::LoadData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:145
// RVA: 0x0005E390
// ADDRESS: 0045e390
// PROTOTYPE: bool __thiscall LoadData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::LoadDefaultProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:194
// RVA: 0x0005E560
// ADDRESS: 0045e560
// PROTOTYPE: void __thiscall LoadDefaultProperty(uchar param_1, uchar param_2, uchar param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordQuestDataFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:820
// RVA: 0x0005E970
// ADDRESS: 0045e970
// PROTOTYPE: bool __thiscall DecordQuestDataFromByteArray(uchar * param_1, long * param_2)
//
// IMPLEMENTED выше; signed `count > 0`, key/value quest ID и overwrite
// duplicate semantics исходного `map::operator[]` сохранены.

// ============================================================================
// FUNCTION: CPlayer::AddQuestFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:842
// RVA: 0x0005EA00
// ADDRESS: 0045ea00
// PROTOTYPE: void __thiscall AddQuestFromDB(ushort param_1, uchar param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::ChangeCountry
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:884
// RVA: 0x0005EA30
// ADDRESS: 0045ea30
// PROTOTYPE: int __thiscall ChangeCountry(uchar param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::CPlayer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:26
// RVA: 0x0005EB10
// ADDRESS: 0045eb10
// PROTOTYPE: undefined __thiscall CPlayer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::DecordFromByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp:521
// RVA: 0x0005F520
// ADDRESS: 0045f520
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полная реализация находится в `CPlayer::decord_from_byte_array`; ранние
// очистки и partial mutations принадлежат достигнутым wire-owner-ам.

// ============================================================================
// FUNCTION: Catch@004d72e3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\player.cpp
// RVA: 0x000D72E3
// ADDRESS: 004d72e3
// PROTOTYPE: undefined Catch@004d72e3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
