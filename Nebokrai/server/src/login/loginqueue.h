#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <deque>
#include <filesystem>
#include <functional>
#include <map>
#include <mutex>
#include <optional>
#include <set>
#include <span>
#include <string>
#include <vector>

class ClientSendQueue;

namespace LoginNet
{
class CMessage;
}

/*
 * Owner: loginserver/loginqueue.cpp / loginqueue.h
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходные пути PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\server\\loginserver\\loginserver\\loginqueue.cpp
 * и loginqueue.h. Для материализованной части подтверждены RVA:
 * AddQuestCdkey 0x0001CAD0, AddQuestPlayerList 0x0001E740,
 * AddQuestPlayerData 0x0001E880, OnClientLost 0x0001A800,
 * PushBackPwdChecked 0x00019880, CheckMsgInfo 0x00016200,
 * ChangeValidCode 0x00016300, ValidateValidCode 0x000184B0,
 * matrix_add 0x0001B870, matrix_register 0x0001B950,
 * matirx_validate 0x00019800, LoadNoQueueCdkeyList 0x000195E0,
 * HandlePwdChecked 0x0001BB20, OnQuestPlayerData 0x0001B3F0,
 * IsValidQuest 0x000172C0, PushLoginList 0x0001AAB0,
 * ClearTimeoutList 0x00017330, IsValidErrManyTimes 0x000163E0,
 * matrices_timeout 0x000183D0, ValidCodeOvertime 0x00018610,
 * CheckValidErr 0x00018700, AddGasQueue 0x000199A0 и
 * OnQuestCdkey 0x0001A130.
 *
 * TagPwdChecked сохраняет signed socket ID, исходный IPv4, byte-exact account,
 * World и matrix-флаг. Duplicate password-result ищется под тем же lock;
 * KickOut вызывается до удаления прежней записи и всё ещё под lock, после чего
 * новая запись всегда попадает в хвост.
 *
 * Client request queues сохраняют исходное разделение: обычная/no-queue FIFO
 * CD-key и ordered map<World, FIFO> для player-list/player-data. AddQuestCdkey
 * не получает донорских duplicate/capacity-check. AddQuestPlayerList выполняет
 * только три исходные проверки World/login-state/player-count; PlayerData —
 * только наличие текущего World account. OnClientLost чистит исключительно
 * первые совпадения в ОБЫЧНЫХ очередях: один CD-key и не более одного элемента
 * каждого World FIFO player-list/player-data. No-queue/GAS/password/valid-code/
 * matrix исходная функция не трогала.
 *
 * Valid-code map хранит endpoint, added/change boot tick, code, World и matrix.
 * CheckMsgInfo различает missing/socket/frequent с точным условием
 * now < change_time + 1000. Endpoint mismatch ValidateValidCode удаляет запись,
 * неверный code оставляет её, успех удаляет и возвращает World/matrix. Caller
 * исходно повторно вызывает DelValidCode после успеха — безопасный no-op тоже
 * сохраняется. Valid-error счётчик использует 32-битный wrapping increment и
 * каждый раз продлевает deadline.
 *
 * Matrix — одноразовая account-запись endpoint + три позиции 0..79. Endpoint
 * mismatch удаляет её. Совпавшая запись передаёт позиции/ответ фактическому
 * CRsCDKey-owner через ILoginQueueContext и удаляется после доказанного C/D;
 * отсутствие DB-owner либо недостаточная длина его blob остаются явной
 * неизвестной границей и не превращаются в придуманный отказ. matrix_register
 * заменяет старую запись с кодом F, выбирает три modulo-0x50 позиции системным
 * RNG, игнорирует исходный result matrix_add и всегда после него отправляет B.
 * Прямой disassembly EXE 0x0041BAA7..0x0041BAD2 уточняет wire B: account идёт
 * через CMessage::Add(const char*) как raw C-string+NUL, позиции — через
 * CMessage::Add(void*,3) как raw три байта, без length-prefix. Это место
 * намеренно не повторяет ошибочный helper поздней Rust-реконструкции.
 * Direct CGame::PrepareEnter сам вызывает matrix_register; пока CGame-owner ещё
 * не материализован, MatrixRegistrationRequired является узкой переходной
 * границей интерфейса. После восстановления CGame этот side effect вернётся
 * внутрь PrepareEnter без изменения внешней state-machine.
 *
 * HandlePwdChecked полностью дренирует password-result FIFO под ОДНИМ
 * исходным lockPwdChecked. Invalid endpoint — только socket==0 либо IP==0;
 * пустой std::string-account исходник отдельно не отбрасывал. Valid-error
 * upper limit даёт Q. При включённой captcha любая уже существующая запись
 * account сначала получает N на её сохранённый socket — даже если socket тот
 * же самый, donor-проверка `old != new` в EXE отсутствует. Затем запись
 * безусловно заменяется новым code/world/matrix и отправляется exact
 * J + account + long 0x70B6 + raw 0x70B6 BMP. Ошибка библиотечного генератора
 * остаётся typed technical boundary без придуманного client-кода. При
 * выключенной captcha выполняется PrepareEnter -> EnterGame; временный
 * MatrixRegistrationRequired компенсирует ещё не материализованный side effect
 * точного CGame::PrepareEnter через уже восстановленный matrix_register.
 *
 * Player-data detail имеет отдельный cooldown-map player_id -> boot tick.
 * IsValidQuest разрешает отсутствующий id, запрещает его при
 * now <= added + interval и при строго просроченной записи сначала удаляет её.
 * PushLoginList вставляет только отсутствующий id и никогда не обновляет время
 * существующего. OnQuestPlayerData при разрешении делает void World-send и
 * ВСЕГДА затем PushLoginList, независимо от внутреннего результата routing;
 * повторный id получает 0xAF503 + byte 0x1C + account через identity-send.
 * ClearTimeoutList проверяет boot tick отдельно для каждой записи и удаляет
 * только при added + interval < now. std::map/mutex заменяют старый raw tree;
 * 32-bit tick arithmetic остаётся wrapping.
 *
 * OnQuestCdkey сохраняет исходный authentication split. Непустой World
 * немедленно идёт PrepareEnter/EnterGame и обходит password/DB. При пустом
 * World inside-mode 0 кладёт owned-копию в отдельную GAS FIFO, mode !=0/1 —
 * точный no-op, mode 1 выполняет numeric FixPtAcc -> ban -> allow -> forbid ->
 * between -> matrix -> password. Первые ровно 16 digest bytes кодируются в 32
 * uppercase hex; только AuthServer-ветка lowercases account и hex перед уже
 * восстановленным AuthManager. Local password success кладёт canonical userid
 * в PwdChecked и сохраняет matrix flag. Короткий digest, отсутствующий
 * CRsCDKey/setup/Auth transport и ошибка чтения local time остаются typed
 * technical boundaries вместо OOB/ложного успеха. Ban wire exact:
 * 0x10, 0, затем пять short year/month/day/hour/minute. std::chrono + системный
 * localtime заменяют GetLocalTime; raw OLE DATE остаётся результатом DB-owner,
 * а его DecodeVariantTime отдан compatibility adapter/library. Порядок exact:
 * raw ban -> local-now -> decode OLE -> compare local-now < expiry.
 *
 * Timeout helpers сохраняют отдельные one-shot maps. Matrix timeout истекает
 * только при matrix_timeout < now - added и отправляет E; valid-code — при
 * valid_overtime < now - added и отправляет M. В обоих случаях boot tick
 * читается заново на каждой записи, затем запись удаляется. Valid-error
 * считается заблокированным только при существующей записи и
 * upper_limit <= error_times. CheckValidErr удаляет строго при
 * next_login_time < now: равенство ещё живо. Старые nullable heap pointers
 * valid-error заменены value-map, поэтому исходная ветка null-entry становится
 * невозможным техническим состоянием, а не отдельной игровой семантикой.
 *
 * NoQueueAccounts.conf остаётся owner-данными CLoginQueue. std::filesystem и
 * owned STL-контейнеры заменяют Win32 case-insensitive filesystem, char[0x100],
 * std::list/map/set и ручное владение. Пустой файл, token >= 0x100 и ANSI
 * lowercase high-bit bytes помечаются безопасной неизвестной границей вместо
 * воспроизведения старого uninitialized/overflow/locale поведения.
 *
 * Полный Run, GAS worker и cadence ещё не материализованы: их нельзя закрывать
 * заглушками только ради сборки. Queue-side OnQuestCdkey/AddGasQueue уже есть,
 * но фактический CGasThread/CGasOperator остаётся отдельным owner-узлом.
 */
namespace Login
{
class AuthManager;
class IAuthListener;

class TagPwdChecked
{
public:
    TagPwdChecked(std::int32_t socketId,
                  std::uint32_t clientIp,
                  std::vector<std::uint8_t> account,
                  std::vector<std::uint8_t> worldServer,
                  bool hasMatrix);

    [[nodiscard]] std::int32_t SocketID() const noexcept;
    [[nodiscard]] std::uint32_t ClientIP() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> Account() const noexcept;
    [[nodiscard]] std::span<const std::uint8_t> WorldServer() const noexcept;
    [[nodiscard]] bool HasMatrix() const noexcept;

private:
    std::uint32_t m_ClientIP{};
    std::int32_t m_SocketID{};
    std::vector<std::uint8_t> m_Account;
    std::vector<std::uint8_t> m_WorldServer;
    bool m_HasMatrix{};
};

enum class PrepareEnterOutcome
{
    Continue,
    Finished,
    MatrixRegistrationRequired,
};

enum class MatrixCardValidationKind
{
    Compared,
    OwnerMissing,
    ValueTooShort,
};

struct MatrixCardValidation
{
    MatrixCardValidationKind kind{MatrixCardValidationKind::Compared};
    bool accepted{};
    std::size_t actualLength{};
    std::size_t requiredLength{};
};

struct LocalDateTime
{
    std::uint16_t year{};
    std::uint16_t month{};
    std::uint16_t day{};
    std::uint16_t hour{};
    std::uint16_t minute{};
    std::uint16_t second{};
    std::uint16_t milliseconds{};
};

enum class QuestCdkeyErrorKind
{
    InsideModeMissing,
    DatabaseOwnerMissing,
    PasswordDigestTooShort,
    AuthTransportMissing,
    LocalTimeUnavailable,
    VariantTimeConversionFailed,
};

struct QuestCdkeyError
{
    QuestCdkeyErrorKind kind{};
    std::size_t actualLength{};
    std::string detail;
};

class ILoginQueueContext
{
public:
    virtual ~ILoginQueueContext() = default;

    [[nodiscard]] virtual bool
    IsExitWorld(std::span<const std::uint8_t> worldName) const = 0;
    [[nodiscard]] virtual std::optional<std::vector<std::uint8_t>>
    LoginCdkeyWorldServer(std::span<const std::uint8_t> account) const = 0;
    [[nodiscard]] virtual std::int32_t
    LoginWorldPlayerNumByName(std::span<const std::uint8_t> worldName) const = 0;
    [[nodiscard]] virtual bool ValidCodeEnabled() const noexcept = 0;
    [[nodiscard]] virtual std::int32_t ValidErrorUpperLimit() const noexcept = 0;
    [[nodiscard]] virtual std::uint32_t QuestPlayerDataIntervalMs() const noexcept = 0;
    [[nodiscard]] virtual std::uint32_t MatrixTimeoutMs() const noexcept = 0;
    [[nodiscard]] virtual std::uint32_t ValidCodeOvertimeMs() const noexcept = 0;
    [[nodiscard]] virtual std::optional<std::int32_t> InsideUseMode() const noexcept = 0;

    [[nodiscard]] virtual bool HasRsCdKeyOwner() const noexcept = 0;
    [[nodiscard]] virtual std::vector<std::uint8_t>
    FixPtAccount(std::span<const std::uint8_t> account) = 0;
    [[nodiscard]] virtual double
    BanVariantTime(std::span<const std::uint8_t> account) = 0;
    [[nodiscard]] virtual std::optional<LocalDateTime>
    DecodeVariantTime(double variantTime) = 0;
    [[nodiscard]] virtual bool IpIsAllowed(std::uint32_t clientIp) = 0;
    [[nodiscard]] virtual bool IpIsForbidden(std::uint32_t clientIp) = 0;
    [[nodiscard]] virtual bool IsBetweenIp(std::span<const std::uint8_t> account,
                                           std::uint32_t clientIp) = 0;
    [[nodiscard]] virtual bool MatrixUsed(std::span<const std::uint8_t> account) = 0;
    [[nodiscard]] virtual std::optional<std::vector<std::uint8_t>>
    ValidateLocalPassword(std::span<const std::uint8_t> account,
                          std::span<const std::uint8_t> passwordHex) = 0;

    [[nodiscard]] virtual bool IsConnectAS() const noexcept = 0;
    [[nodiscard]] virtual ClientSendQueue* AuthSendQueue() noexcept = 0;
    [[nodiscard]] virtual IAuthListener* AuthListener() noexcept = 0;
    virtual void KickOut(std::span<const std::uint8_t> account) = 0;

    virtual void L2WQuestDetailSend(
        std::optional<std::span<const std::uint8_t>> worldServer,
        std::span<const std::uint8_t> account,
        std::int32_t playerId,
        std::uint32_t clientIp) = 0;
    virtual void SendToClientCdkey(const LoginNet::CMessage& message,
                                   std::span<const std::uint8_t> account) = 0;

    [[nodiscard]] virtual PrepareEnterOutcome
    PrepareEnter(const TagPwdChecked& checked) = 0;
    virtual void EnterGame(const TagPwdChecked& checked, bool noQueueAccount) = 0;

    [[nodiscard]] virtual MatrixCardValidation ValidateMatrixCard(
        std::span<const std::uint8_t> account,
        std::span<const std::uint8_t, 3> positions,
        std::span<const std::uint8_t, 3> answer) = 0;

    virtual void SendToClient(const LoginNet::CMessage& message,
                              std::int32_t socketId) = 0;
};

struct ClientLostCleanupReport
{
    bool cdkeyRemoved{};
    std::size_t playerListRemoved{};
    std::size_t playerDataRemoved{};
};

enum class CheckMessageInfo
{
    Success,
    Missing,
    SocketMismatch,
    Frequent,
};

struct ValidateValidCodeOutcome
{
    bool accepted{};
    std::vector<std::uint8_t> worldServer;
    bool hasMatrix{};
};

enum class MatrixValidationOutcomeKind
{
    Accepted,
    Rejected,
    OwnerMissing,
    ValueTooShort,
};

struct MatrixValidationOutcome
{
    MatrixValidationOutcomeKind kind{MatrixValidationOutcomeKind::Rejected};
    std::size_t actualLength{};
    std::size_t requiredLength{};
};

enum class MatrixRegisterErrorKind
{
    Random,
};

struct MatrixRegisterError
{
    MatrixRegisterErrorKind kind{MatrixRegisterErrorKind::Random};
    std::string detail;
};

enum class NoQueueAccountsLoadErrorKind
{
    Io,
    EmptyFileLegacyReadUnknown,
    TokenTooLongLegacyOverflow,
    NonAsciiCaseMappingUnknown,
};

struct NoQueueAccountsLoadError
{
    NoQueueAccountsLoadErrorKind kind{NoQueueAccountsLoadErrorKind::Io};
    std::size_t tokenIndex{};
    std::size_t actualLength{};
    std::string detail;
};

struct NoQueueAccountsLoadResult
{
    std::size_t extractedAccounts{};
    std::size_t uniqueAccounts{};
    std::optional<NoQueueAccountsLoadError> error;
};

enum class PwdCheckedErrorKind
{
    ValidCodeGeneration,
    MatrixRandom,
};

struct PwdCheckedError
{
    PwdCheckedErrorKind kind{};
    std::vector<std::uint8_t> account;
    std::string detail;
};

struct HandlePwdCheckedReport
{
    std::size_t processed{};
    std::size_t droppedInvalidEndpoint{};
    std::size_t rejectedByValidErrors{};
    std::size_t generatedValidCodes{};
    std::size_t enteredWithoutValidCode{};
    std::vector<PwdCheckedError> errors;
};

class CLoginQueue
{
public:
    using KickOutCallback = std::function<void(std::span<const std::uint8_t>)>;

    explicit CLoginQueue(const std::filesystem::path& runtimeDirectory = ".");

    void OnInitial(std::uint32_t intervalMs,
                   std::uint32_t sendMessageIntervalMs,
                   std::uint32_t worldMaxPlayers);
    void SetWorldCount(std::uint32_t worldCount);

    [[nodiscard]] NoQueueAccountsLoadResult
    LoadNoQueueCdkeyList(const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] bool IsInNoQueueList(std::span<const std::uint8_t> account) const;

    void AddQuestCdkey(std::int32_t socketId,
                       std::uint32_t clientIp,
                       std::int32_t loginType,
                       std::int32_t version,
                       std::span<const std::uint8_t> account,
                       std::span<const std::uint8_t> passwordDigest,
                       std::int16_t clientCode,
                       std::int32_t encryptionKey,
                       std::span<const std::uint8_t> worldServer);
    [[nodiscard]] bool AddQuestPlayerList(ILoginQueueContext& context,
                                          std::int32_t socketId,
                                          std::span<const std::uint8_t> account,
                                          std::span<const std::uint8_t> worldServer);
    [[nodiscard]] bool AddQuestPlayerData(ILoginQueueContext& context,
                                          std::int32_t socketId,
                                          std::span<const std::uint8_t> account,
                                          std::int32_t playerId,
                                          std::uint32_t clientIp);
    [[nodiscard]] ClientLostCleanupReport
    OnClientLost(std::span<const std::uint8_t> account);

    void PushBackPwdChecked(TagPwdChecked checked, const KickOutCallback& kickOut);
    [[nodiscard]] std::size_t PendingPwdChecked() const;
    [[nodiscard]] HandlePwdCheckedReport HandlePwdChecked(ILoginQueueContext& context);

    [[nodiscard]] CheckMessageInfo
    CheckMsgInfo(std::span<const std::uint8_t> account, std::int32_t socketId) const;
    [[nodiscard]] ValidateValidCodeOutcome ValidateValidCode(
        std::int32_t socketId,
        std::uint32_t clientIp,
        std::span<const std::uint8_t> suppliedCode,
        std::span<const std::uint8_t> account);
    void ChangeValidCode(std::span<const std::uint8_t> account,
                         std::span<const std::uint8_t> validCode);
    void DelValidCode(std::span<const std::uint8_t> account);
    void AddValidErr(std::span<const std::uint8_t> account,
                     std::uint32_t stayTimeMs);

    [[nodiscard]] MatrixValidationOutcome ValidateMatrix(
        ILoginQueueContext& context,
        std::int32_t socketId,
        std::uint32_t clientIp,
        std::span<const std::uint8_t> account,
        std::span<const std::uint8_t, 3> answer);
    [[nodiscard]] std::optional<MatrixRegisterError>
    MatrixRegister(ILoginQueueContext& context, const TagPwdChecked& checked);

private:
    struct QuestCdkey
    {
        std::int32_t socketId{};
        std::uint32_t clientIp{};
        std::int8_t loginType{};
        std::int32_t version{};
        std::vector<std::uint8_t> account;
        std::vector<std::uint8_t> passwordDigest;
        std::int16_t clientCode{};
        std::int32_t encryptionKey{};
        std::vector<std::uint8_t> worldServer;
        std::uint32_t sendMessageTime{};
        std::vector<std::uint8_t> firstGasValue;
        std::vector<std::uint8_t> secondGasValue;
        std::vector<std::uint8_t> nickname;
    };

    struct QuestPlayerList
    {
        std::int32_t socketId{};
        std::vector<std::uint8_t> account;
        std::vector<std::uint8_t> worldServer;
        std::uint32_t sendMessageTime{};
    };

    struct QuestPlayerData
    {
        std::int32_t socketId{};
        std::vector<std::uint8_t> account;
        std::int32_t playerId{};
        std::uint32_t clientIp{};
        std::uint32_t sendMessageTime{};
    };

    struct ValidCodeEntry
    {
        std::int32_t socketId{};
        std::uint32_t clientIp{};
        std::uint32_t addedTime{};
        std::vector<std::uint8_t> validCode;
        std::vector<std::uint8_t> worldServer;
        bool hasMatrix{};
        std::uint32_t changeTime{};
    };

    struct ValidErrorEntry
    {
        std::int32_t errorTimes{};
        std::uint32_t nextLoginTime{};
    };

    struct MatrixEntry
    {
        std::int32_t socketId{};
        std::uint32_t clientIp{};
        std::array<std::uint8_t, 3> positions{};
        std::uint32_t addedTime{};
    };

    void AddGasQueue(const QuestCdkey& quest);
    [[nodiscard]] std::optional<QuestCdkeyError>
    OnQuestCdkey(ILoginQueueContext& context, AuthManager& authManager, const QuestCdkey& quest);
    void OnQuestPlayerData(ILoginQueueContext& context, const QuestPlayerData& quest);
    [[nodiscard]] bool IsValidQuest(ILoginQueueContext& context, std::int32_t playerId);
    [[nodiscard]] bool PushLoginList(std::int32_t playerId);
    void ClearTimeoutList(ILoginQueueContext& context);
    [[nodiscard]] bool IsValidErrManyTimes(ILoginQueueContext& context,
                                           std::span<const std::uint8_t> account) const;
    void MatricesTimeout(ILoginQueueContext& context);
    void ValidCodeOvertime(ILoginQueueContext& context);
    void CheckValidErr(std::uint32_t now);

    [[nodiscard]] bool AddMatrix(std::int32_t socketId,
                                 std::uint32_t clientIp,
                                 std::span<const std::uint8_t> account,
                                 std::span<const std::uint8_t, 3> positions);
    [[nodiscard]] std::uint32_t SendMessageIntervalMs() const;

    mutable std::mutex m_SetupMutex;
    std::uint32_t m_IntervalMs{};
    std::uint32_t m_SendMessageIntervalMs{};
    std::uint32_t m_WorldMaxPlayers{};
    std::uint32_t m_WorldCount{};
    std::uint32_t m_LogQueueTime{};
    std::uint32_t m_WorldQueueTime{};

    mutable std::mutex m_NoQueueMutex;
    std::set<std::vector<std::uint8_t>> m_NoQueueAccounts;

    mutable std::mutex m_QuestCdkeyMutex;
    std::deque<QuestCdkey> m_QuestCdkey;
    std::deque<QuestCdkey> m_NoQueueQuestCdkey;

    mutable std::mutex m_GasQuestMutex;
    std::deque<QuestCdkey> m_GasQuest;

    mutable std::mutex m_PlayerQuestMutex;
    std::map<std::vector<std::uint8_t>, std::deque<QuestPlayerList>> m_QuestPlayerList;
    std::map<std::vector<std::uint8_t>, std::deque<QuestPlayerList>> m_NoQueueQuestPlayerList;
    std::map<std::vector<std::uint8_t>, std::deque<QuestPlayerData>> m_QuestPlayerData;
    std::map<std::vector<std::uint8_t>, std::deque<QuestPlayerData>> m_NoQueueQuestPlayerData;

    mutable std::mutex m_LoginListMutex;
    std::map<std::int32_t, std::uint32_t> m_LoginList;

    mutable std::mutex m_PwdCheckedMutex;
    std::deque<TagPwdChecked> m_PwdChecked;

    mutable std::mutex m_ValidCodeMutex;
    std::map<std::vector<std::uint8_t>, ValidCodeEntry> m_ValidCodes;

    mutable std::mutex m_ValidErrMutex;
    std::map<std::vector<std::uint8_t>, ValidErrorEntry> m_ValidErrors;

    mutable std::mutex m_MatrixMutex;
    std::map<std::vector<std::uint8_t>, MatrixEntry> m_Matrices;
};
}
