//! Четыре навыка Swordship (0x6F/0xE0/0xE8/0xE9).
//! Источник: gameserver.exe/GameServer.pdb (пара `4F5C98E0…`, RSDS match),
//! appserver/skills/swordship{,2,3,4}.cpp/.h.
//! Весь класс делегирован: общий зарегистрированный Begin/AI/End —
//! `immediatestate.rs`, ID-карта из четырёх вариантов и чтение MIN перед MAX —
//! zone rules `skills/immediate.rs` и `effects/swordship.rs`, замена в прежней
//! позиции — `immediatestateinstallation.rs`, живой property/Begin/End —
//! `swordshipstate.rs`.
//! MATCH по машинной разведке порции №4: AI читает свойства и требует именно
//! GetU, без подстановки GetS; MIN читается перед MAX — тот же порядок у
//! property callback VA `0x005F8870` и writer VA `0x005ECE70` состояний.
//! Новый Begin(U,U) предшествует поиску первого того же ID без фильтра
//! RTTI/ended. Нет отдельного visual, OnChangeStates, MP, delay или reuse;
//! успешный и отказной AI заканчиваются End(0). Player и monster используют
//! один зарегистрированный экземпляр и общий lifecycle.
