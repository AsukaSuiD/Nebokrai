#pragma once

#include <array>
#include <atomic>
#include <bit>
#include <cstdint>

/*
 * Исходный владелец: nets/mysocket.cpp / nets/mysocket.h
 *
 * Источник истины: оригинальные EXE/PDB Auth, Login, Billing, Misc, Game и
 * World. Поздняя Rust-реконструкция сохраняет уже выполненный reverse этого
 * общего владельца; архивная Linux-попытка используется только как C++-донор.
 *
 * Все варианты подтверждают общие исходные значения CMySocket: SOCK_STREAM
 * (1), IPv4 127.0.0.1, port 5000, invalid socket, нулевой последний UDP-port и
 * пустой последний UDP-IP. Bind трактовал null IP как 0.0.0.0, сужал port до
 * unsigned 16 bit и использовал legacy inet_addr. Последний parser принимал
 * старые IPv4-формы и одновременно представлял 255.255.255.255 как INADDR_NONE;
 * поэтому строгий современный строковый parser здесь намеренно не вводится.
 *
 * Базовые Create и Send исторического CMySocket были заглушками; живой TCP
 * runtime принадлежал CClient/CServer/CServerClient. В новом Linux-коде
 * создание, bind/listen/connect, readiness, recv/send и lifetime socket-а также
 * остаются у clients/servers и будут выражены зрелым networking backend-ом.
 * Этот владелец не создаёт второй самодельный аналог WinSock поверх Linux.
 *
 * PDB также подтверждает SetNonblocking, SetNodelay, SetReuseaddr,
 * SetKeepalive, SetRecvbuf и SetSendbuf. Их контракт — FIONBIO/TCP_NODELAY/
 * SO_REUSEADDR/SO_KEEPALIVE/SO_RCVBUF/SO_SNDBUF с 32-битным legacy value.
 * Эти настройки должен применять непосредственный владелец транспорта через
 * выбранный backend; отдельные raw POSIX wrappers здесь не материализуются.
 *
 * GetSocketID увеличивал process-global signed 32-bit значение, начавшееся с
 * нуля. В старых отдельных EXE у каждого процесса был собственный счётчик.
 * Поскольку новый сервер может объединять бывшие процессы, один общий global
 * изменил бы исторические последовательности ID. Поэтому allocator является
 * отдельным объектом и создаётся каждым владельцем службы самостоятельно.
 */

using IPv4Octets = std::array<std::uint8_t, 4>;

inline constexpr std::int32_t kDefaultSocketType = 1;
inline constexpr std::uint32_t kDefaultSocketPort = 5000;
inline constexpr IPv4Octets kDefaultSocketIPv4{127, 0, 0, 1};
inline constexpr IPv4Octets kAnySocketIPv4{0, 0, 0, 0};

/// Представляет четыре network-order IPv4 bytes как исходный x86 DWORD.
[[nodiscard]] constexpr std::uint32_t LegacyIPv4Word(const IPv4Octets& address) noexcept
{
    return static_cast<std::uint32_t>(address[0]) |
           (static_cast<std::uint32_t>(address[1]) << 8U) |
           (static_cast<std::uint32_t>(address[2]) << 16U) |
           (static_cast<std::uint32_t>(address[3]) << 24U);
}

/// Сужает legacy 32-bit port так же, как исходный cast к unsigned short.
[[nodiscard]] constexpr std::uint16_t LegacySocketPort(std::uint32_t port) noexcept
{
    return static_cast<std::uint16_t>(port);
}

/// Независимый счётчик socket ID одного исторического server process.
class SocketIdAllocator
{
public:
    constexpr SocketIdAllocator() noexcept = default;

    SocketIdAllocator(const SocketIdAllocator&) = delete;
    SocketIdAllocator& operator=(const SocketIdAllocator&) = delete;

    /// Выдаёт следующий signed 32-bit ID с тем же wrapping битового значения.
    [[nodiscard]] std::int32_t Next() noexcept;

private:
    std::atomic<std::uint32_t> m_LastIssued{0U};
};
