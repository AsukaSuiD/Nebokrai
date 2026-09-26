//! Мировая адресация общения Realm: очереди общих объявлений и ссылок на
//! предметы — primary states владельца `social` (прежние transitional-поля
//! `CGame`).

pub mod broadcasts; // системная рассылка tagSysBroadcast и её AI-отчёты: live-список с pub-полем для in-place мутаций AI-цикла app.
pub mod goodslinks; // реестр goods link мира: ctor-контракт 500 placeholder-ов, typed add/find/clear и process-global счётчик индексов.
