//! Страны, фракции, союзы, членство и заявления Realm.

pub mod checkpoint; // исходный CGame::CheckPoint: удвоение кавычек SQL-строки.
pub mod country; // CCountry: данные и правила государства.
pub mod countryhandler; // CCountryHandler: карта государств мира.
pub mod countryidentity; // базовая идентичность страны.
pub mod dbcountry; // DB-владелец стран: save-проекции, трейт DbCountryOwner и Tiberius-реализация.
pub mod faction; // CFaction: базовые свойства и wire-контракты owned-cities фракции.
pub mod factionenemyblock; // отчёт отказа мутации enemy-связи фракций.
pub mod goodswarmember; // CGoodsWarMember: участники Goods War.
pub mod king; // CKing: владелец очков контроля/материалов/войны страны.
pub mod minister; // CMinister: номинальный minister owner.
pub mod officer; // COfficer: четыре officer-поля country owner-а.
pub mod organizingctrl; // COrganizingCtrl: центральный контроллер организаций (Registry, операции, staging) и его wire-контракты/отчёты.
pub mod organizingparam; // COrganizingParam: параметры организаций.
pub mod rsenemyfactions; // CRsEnemyFactions: World DB-владелец enemy-связей.
pub mod rsfaction; // CRsFaction: World DB-владелец фракций, его трейт и Tiberius-реализация.
pub mod rsunion; // CRsUnion: World DB-владелец союзов, его трейт и Tiberius-реализация.
pub mod union; // CUnion: owner союза и контракты owned-city мутаций.
