#pragma once

#include <cstddef>
#include <cstdint>
#include <optional>
#include <span>
#include <vector>

class CGUID;

/*
 * Owner: nets/basemessage.h + nets/basemessage.cpp
 *
 * Источник: точные EXE/PDB AuthServer, LoginServer, BillingServer, MiscServer,
 * GameServer и Nworldserver. Во всех шести вариантах подтверждён один базовый
 * 16-байтовый wire-буфер: +0 хранит 32-битную длину, +4 — тип сообщения,
 * +8 и +0xC остаются зарезервированными словами без придуманной семантики.
 * Числа кодируются как x86 little-endian, курсор чтения начинается с offset 16.
 *
 * Подтверждённый legacy RLE общий для вариантов, где соответствующие функции
 * присутствуют. Старые глобальные scratch-буферы и Initial/Release не являются
 * частью wire-контракта: поздний аудит send-path показал, что отправители
 * копируют переданный буфер до возврата. Здесь их заменяют владеющие std::vector.
 *
 * Остаются отдельными неизвестностями malformed-пути ограниченного GetStr,
 * GetEx и RLE marker без следующего байта. Им не назначается новая реакция
 * только ради удобства безопасного C++ API.
 */
class CBaseMessage
{
public:
    static constexpr std::size_t kHeaderSize = 16;
    static constexpr std::size_t kGuidSize = 16;

    CBaseMessage();
    CBaseMessage(std::span<const std::uint8_t, kHeaderSize> header,
                 std::span<const std::uint8_t> payload);
    virtual ~CBaseMessage() = default;

    [[nodiscard]] static std::optional<std::vector<std::uint8_t>> DoRLE(std::span<const std::uint8_t> source);
    [[nodiscard]] static bool DecodeRLE_SAFE(std::span<const std::uint8_t> source,
                                             std::span<std::uint8_t> destination,
                                             std::size_t& decodedSize);

    void Update();

    [[nodiscard]] char GetChar();
    [[nodiscard]] std::uint8_t GetByte();
    [[nodiscard]] std::int16_t GetShort();
    [[nodiscard]] std::uint16_t GetWord();
    [[nodiscard]] std::int32_t GetLong();
    [[nodiscard]] std::uint32_t GetULong();
    [[nodiscard]] std::int64_t GetLONG64();
    [[nodiscard]] float GetFloat();
    [[nodiscard]] void* Get(void* destination, std::int32_t size);
    [[nodiscard]] bool GetGUID(CGUID& guid);
    [[nodiscard]] std::vector<std::uint8_t> GetCStringBytes();

    void Add(char value);
    void Add(std::uint8_t value);
    void Add(std::int16_t value);
    void Add(std::uint16_t value);
    void Add(std::int32_t value);
    void Add(std::uint32_t value);
    void Add(std::int64_t value);
    void Add(float value);
    void Add(const char* value);
    void Add(const void* source, std::int32_t size);
    void AddEx(const void* source, std::int32_t size);
    void Add(const CGUID& guid);

    [[nodiscard]] const std::vector<std::uint8_t>& MessageData() const noexcept;
    [[nodiscard]] std::vector<std::uint8_t>& MessageData() noexcept;
    [[nodiscard]] const std::uint8_t* Data() const noexcept;
    [[nodiscard]] std::uint8_t* Data() noexcept;
    [[nodiscard]] std::size_t DataSize() const noexcept;

    void ResetReadPtr() noexcept;
    void SetReadPtr(std::int32_t offset) noexcept;
    [[nodiscard]] std::int32_t GetReadPtr() const noexcept;

    [[nodiscard]] std::uint32_t GetSize() const noexcept;
    [[nodiscard]] std::uint32_t GetType() const noexcept;
    void SetType(std::uint32_t value) noexcept;

protected:
    [[nodiscard]] bool CanRead(std::size_t size) const noexcept;

private:
    void AppendBytes(const void* source, std::size_t size);
    void SetSize(std::uint32_t value) noexcept;
    [[nodiscard]] std::uint32_t ReadHeaderField(std::size_t offset) const noexcept;
    void WriteHeaderField(std::size_t offset, std::uint32_t value) noexcept;

    std::vector<std::uint8_t> m_MsgData;
    std::int32_t m_lPtr;
};
