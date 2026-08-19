#pragma once

#include "../../loginqueue.h"
#include "../validcode.h"
#include "../../../nets/netlogin/message.h"

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <string>
#include <vector>

/*
 * Owner: loginserver/applogin/message/logmessage.cpp
 *
 * Точная пара: LoginServer/loginserver.exe + LoginServer/LoginServer.pdb.
 * Исходный путь PDB:
 * d:\\complite_version\\fengyun_russia\\trunk\\server\\loginserver\\applogin\\message\\logmessage.cpp
 * OnLogMessage RVA 0x0007F3F0.
 *
 * Восстановлены все двадцать достигнутых case: synthetic disconnect 0x10001,
 * World 0x1FF01..0x1FF07 и client 0x2FD01..0x2FD0C. Неизвестный opcode
 * остаётся исходным default no-op. Linux donor используется только как C++-
 * подсказка: его payload validators, ownership checks, pending-created map,
 * launcher-ticket special case, debug trace и capacity guards exact RU owner
 * не имел и сюда они не переносятся.
 *
 * 0x2FD01 сохраняет порядок marker/version -> account(max 0x20) -> удаление
 * пробелов -> длина 1..31 -> exact 16-byte GetEx digest -> marker/version/
 * account checks -> short client-code -> long ekey -> unused string(max 0x40)
 * -> World(max 0x14) -> только затем ASCII lowercase и AddQuestCdkey(type=0).
 * Неверная длина account/digest — тихий return; marker/version дают client 4,
 * quote/equal/space — client 5. Старый GetEx при declared=16 и всего 16..19
 * оставшихся bytes достигал OOB; безопасная реконструкция возвращает typed
 * неизвестность вместо воспроизведения unsafe чтения.
 *
 * 0x2FD0B сохраняет другой layout и подтверждённый дефект digest: compacted
 * length равна числу non-space bytes password-source, но копируется prefix
 * ИСХОДНОГО buffer. NUL добавляется только если пробелов не было. Account после
 * удаления пробелов НЕ lower-case. Marker/version проверяются уже после digest
 * и отказывают через 0xAF50A + long(6) + две пустые строки.
 *
 * 0x2FD02/03 только ставят player-list/player-data в CLoginQueue. Create/
 * delete/restore сначала берут nullable World по metadata CD-key; delete/
 * restore лишь затем читают player ID. Donor ownership validation отсутствует.
 * 0x2FD07 при status==0 сохраняет World notify -> ClearCDKey -> OnClientLost ->
 * AccountLeaveLog; ненулевой status — no-op.
 *
 * 0x2FD08 lower-case account и одноразовая matrix: C удаляет valid-code и
 * входит только при существующей login/world записи, причём EnterGame получает
 * намеренно пустой World; D сначала AddValidErr, затем client D. 0x2FD09
 * выполняет CheckMsgInfo ДО lowercase и без ответа отсекает только socket-
 * mismatch; успех повторно DelValidCode и продолжает PrepareEnter -> EnterGame/
 * matrix_register. 0x2FD0A НЕ меняет регистр account: success меняет code/
 * change-time и отправляет J + account + explicit 0x70B6 + RAW BMP; frequent/
 * missing дают O/P, socket mismatch — no-op. Важно: текущий CBaseMessage::AddEx
 * сам пишет length-prefix, поэтому после explicit 0x70B6 здесь используется
 * обычный Add(raw), иначе wire получил бы лишние четыре байта. Это подтверждено
 * непосредственно EXE: 0x0041BE99/0x00480671 пишут long 0x70B6 через Add(long),
 * а 0x0041BEBE/0x00480696 вызывают raw Add(void*,0x70B6). Поздняя Rust-карта,
 * использовавшая здесь AddEx после explicit long, в этом месте была неточна.
 *
 * World 0x1FF02/03/04/05/07 меняют opcode ТОГО ЖЕ CMessage и пересылают его
 * client identity: cursor-чтение payload не удаляет. 0x1FF01 при status 0x1D
 * выполняет AddCdkey, читает unused string/long, role name/level и пишет
 * RoleEnterLog(world=MapID&0xFF); relay 0xAF503 идёт при любом status. 0x1FF06
 * делает ClearCDKey -> consume unused role/status -> LeaveLog без client send.
 *
 * Synthetic 0x10001 сохраняет странный exact порядок: ClearLoginCdkey и QUIT
 * до FindCdkey; найденный World-CD-key даёт ранний return. Иначе World-name
 * запрашивается уже после удаления login map, затем всегда AccountLeaveLog ->
 * ClearCDKey -> OnClientLost.
 *
 * ILogMessageContext выражает только достигнутые CGame calls. Очереди/valid/
 * matrix остаются у CLoginQueue; routing, account maps, logs и EnterGame — у
 * будущего CGame. Direct OnLogMessage сам вызывает PrepareEnter/EnterGame; это
 * сохранено. Только matrix_register временно вызывается по outcome интерфейса,
 * потому что точный CGame::PrepareEnter (который владеет этим вызовом) ещё не
 * материализован. Это граница частичной материализации, не новая архитектура.
 */
namespace Login
{
enum class LogMessageErrorKind
{
    LoginServerVersionMissing,
    PasswordDigestTruncatedReactionUnknown,
    MatrixAnswerTooShortReactionUnknown,
    MatrixDatabaseOwnerMissing,
    MatrixDatabaseValueTooShort,
    MatrixRandom,
    ValidCodeGeneration,
};

struct LogMessageError
{
    LogMessageErrorKind kind{};
    std::size_t actualLength{};
    std::size_t requiredLength{};
    std::string detail;
    std::optional<ValidCodeError> validCodeError;
};

class ILogMessageContext : public ILoginQueueContext
{
public:
    ~ILogMessageContext() override = default;

    [[nodiscard]] virtual std::optional<std::int32_t> ServerVersion() const = 0;
    [[nodiscard]] virtual std::uint32_t ValidErrorStayTimeMs() const noexcept = 0;

    [[nodiscard]] virtual std::int32_t
    WorldIDByName(std::span<const std::uint8_t> worldName) const = 0;
    virtual void SendToWorld(const LoginNet::CMessage& message,
                             std::int32_t worldId) = 0;
    virtual void SendToClientCdkey(const LoginNet::CMessage& message,
                                   std::span<const std::uint8_t> account) = 0;

    virtual void ClearCdkey(std::span<const std::uint8_t> account) = 0;
    [[nodiscard]] virtual bool ClearLoginCdkey(std::span<const std::uint8_t> account) = 0;
    virtual void QuitClientByCdkey(std::span<const std::uint8_t> account) = 0;
    [[nodiscard]] virtual std::int32_t
    FindCdkey(std::span<const std::uint8_t> account) const = 0;

    [[nodiscard]] virtual bool AddCdkey(std::span<const std::uint8_t> account,
                                        std::int32_t worldId) = 0;
    virtual void AccountLeaveLog(std::span<const std::uint8_t> account) = 0;
    virtual void LeaveLog(std::span<const std::uint8_t> account) = 0;
    virtual void RoleEnterLog(std::span<const std::uint8_t> account,
                              std::span<const std::uint8_t> roleName,
                              std::uint8_t level,
                              std::int32_t worldNumber) = 0;

    virtual void L2WCreateRoleSend(
        std::optional<std::span<const std::uint8_t>> worldServer,
        std::span<const std::uint8_t> account,
        LoginNet::CMessage& source) = 0;
    virtual void L2WDeleteRoleSend(
        std::optional<std::span<const std::uint8_t>> worldServer,
        std::span<const std::uint8_t> account,
        std::int32_t playerId,
        std::uint32_t clientIp) = 0;
    virtual void L2WRestoreRoleSend(
        std::optional<std::span<const std::uint8_t>> worldServer,
        std::span<const std::uint8_t> account,
        std::uint32_t playerId) = 0;

    virtual void AddWorldInfoToMsg(LoginNet::CMessage& message,
                                   std::span<const std::uint8_t> account) = 0;
};

class LogMessageHandler
{
public:
    LogMessageHandler(ILogMessageContext& context, CLoginQueue& queue) noexcept;

    [[nodiscard]] std::optional<LogMessageError>
    OnLogMessage(LoginNet::CMessage& message);

private:
    [[nodiscard]] std::optional<LogMessageError>
    OnClientLoginRequest(LoginNet::CMessage& message);
    [[nodiscard]] std::optional<LogMessageError>
    OnExtendedLoginRequest(LoginNet::CMessage& message);
    void OnPlayerListRequest(LoginNet::CMessage& message);
    void OnPlayerDataRequest(LoginNet::CMessage& message);
    void OnCreateRoleRequest(LoginNet::CMessage& message);
    void OnDeleteRoleRequest(LoginNet::CMessage& message);
    void OnRestoreRoleRequest(LoginNet::CMessage& message);
    void OnClientCleanup(LoginNet::CMessage& message);
    [[nodiscard]] std::optional<LogMessageError> OnMatrixAnswer(LoginNet::CMessage& message);
    [[nodiscard]] std::optional<LogMessageError> OnValidCodeAnswer(LoginNet::CMessage& message);
    [[nodiscard]] std::optional<LogMessageError> OnValidCodeRefresh(LoginNet::CMessage& message);
    void OnWorldListRequest(LoginNet::CMessage& message);
    void OnSyntheticDisconnect(LoginNet::CMessage& message);

    void OnWorldLoginResult(LoginNet::CMessage& message);
    void OnWorldPlayerBaseResponse(LoginNet::CMessage& message);
    void OnWorldDeleteRoleResponse(LoginNet::CMessage& message);
    void OnWorldRestoreRoleResponse(LoginNet::CMessage& message);
    void OnWorldCreateRoleResponse(LoginNet::CMessage& message);
    void OnWorldLeaveResult(LoginNet::CMessage& message);
    void OnWorldResponse1FF07(LoginNet::CMessage& message);
    void RelayWorldResponse(LoginNet::CMessage& message,
                            std::span<const std::uint8_t> account,
                            std::int32_t responseType);

    ILogMessageContext& m_Context;
    CLoginQueue& m_Queue;
};
}
