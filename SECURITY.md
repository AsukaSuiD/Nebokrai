# Сообщения о безопасности / Security

**English summary.** Nebokrai has no stable public releases yet. Report vulnerabilities privately through [GitHub Private Vulnerability Reporting](https://github.com/AsukaSuiD/Nebokrai/security/advisories/new). Do not post vulnerabilities, exploit details, credentials, account data, or private captures in public issues or pull requests. There is currently no response-time commitment.

Nebokrai находится в разработке. Поддерживаемых стабильных публичных релизов и гарантированного срока ответа пока нет. Предмет разбора — текущая основная реализация и отслеживаемые инструменты; состояние проверок описано в [кратком статусе](docs/overview/status.md).

## Как сообщить об уязвимости

Используйте [GitHub Private Vulnerability Reporting](https://github.com/AsukaSuiD/Nebokrai/security/advisories/new): в разделе Security выберите «Report a vulnerability». Сообщение поступит сопровождающему приватно. Не отправляйте сведения об уязвимостях, эксплуатационные payload, credentials, пользовательские данные, оригинальные бинарники и дампы через открытые issue или PR.

## Что подготовить для приватного сообщения

Commit, затронутая служба, платформа, класс воздействия и минимальные шаги воспроизведения на собственной среде. Отделяйте наблюдение от предположения; исключите чужие данные и секреты из материалов. Не прикладывайте действующие секреты или оригинальные proprietary-материалы; используйте минимальный пример без чужих данных.

При утечке credentials отзовите или замените их у соответствующего поставщика. Удаление строки из Git не отзывает credential и не удаляет уже сделанные копии.
