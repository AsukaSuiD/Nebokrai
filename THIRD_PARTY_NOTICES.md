# Сторонние компоненты / Third-party components

**English summary.** Nebokrai's own code is licensed under AGPL-3.0-only. Dependencies retain their own licenses and copyright notices. Cargo and vcpkg obtain them separately; the repository is not a bundle of those dependencies. Binary and container distributions must include the notices and other materials required by the components actually shipped.

## Rust

Прямые зависимости объявлены в [Cargo.toml](server/rust/Cargo.toml), точные версии и контрольные суммы — в [Cargo.lock](server/rust/Cargo.lock). Lockfile также содержит пакеты для других платформ; это не список библиотек, обязательно включённых в каждый бинарник. Выбранный граф для Linux можно получить командой:

```sh
cd server/rust
cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu
```

Условия пакета находятся в его `Cargo.toml`, `LICENSE`, `COPYING` и дополнительных notices. Лицензия обёртки не заменяет условия вложенного native-кода. В частности:

- `lzo` 0.1.3 распространяется по Apache-2.0; это зависимость, выбранная в Cargo.lock, а не утверждение о лицензии любой реализации LZO.
- `ring` 0.17.14 указывает Apache-2.0 **и** ISC; его `LICENSE` перечисляет отдельные notices для исходных компонентов.
- `aws-lc-sys` 0.44.0 содержит компоненты с Apache-2.0, ISC, MIT, MIT-0 и BSD-3-Clause; полный состав условий указан в поставляемом пакете. Сводить его к одной лицензии обёртки нельзя.

## FreeType

Rust-сборка включает feature `bundled` у `freetype-rs` 0.38.0. Через `freetype-sys` 0.23.0 он использует FreeType 2.13.2. Обёртки имеют MIT-лицензию; для вложенного FreeType используется **FreeType License (FTL)**. Исходный текст условий находится в `freetype2/docs/FTL.TXT` внутри пакета `freetype-sys`; дополнительные условия отдельных файлов сохраняются.

Portions of this software are copyright © 1996–2024 The FreeType Project (https://freetype.org/). All rights reserved.

## Отдельный C++-проход и будущие поставки

Зависимости C++ и baseline vcpkg перечислены в [vcpkg.json](server/cpp/vcpkg.json). При подготовке бинарной поставки собирайте уведомления из `share/<package>/copyright` установленного vcpkg и учитывайте используемые системные библиотеки. На Linux некоторые порты используют системную реализацию вместо собственной копии библиотеки.

Перед публикацией бинарников или контейнеров нужно собрать тексты лицензий, copyright/notices и требуемые исходные материалы для фактического состава поставки, включая native-библиотеки и пакеты базового образа. Эта страница — описание используемых компонентов и необходимые credits; она не заменяет полный комплект материалов бинарного релиза. Оригинальные игровые материалы не входят в такую поставку и не покрываются лицензией Nebokrai.
