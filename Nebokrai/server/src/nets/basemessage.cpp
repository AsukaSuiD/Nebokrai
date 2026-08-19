#include "basemessage.h"

#include "../public/guid.h"

#include <cstring>

namespace
{
constexpr std::uint8_t kRleMarkerBase = 0xF7;
constexpr std::uint8_t kRleFirstMarker = 0xF8;
constexpr std::size_t kRleMaxRun = 8;

std::uint32_t ReadLe32(const std::uint8_t* source) noexcept
{
    return static_cast<std::uint32_t>(source[0]) |
           (static_cast<std::uint32_t>(source[1]) << 8) |
           (static_cast<std::uint32_t>(source[2]) << 16) |
           (static_cast<std::uint32_t>(source[3]) << 24);
}

void WriteLe32(std::uint8_t* destination, std::uint32_t value) noexcept
{
    destination[0] = static_cast<std::uint8_t>(value);
    destination[1] = static_cast<std::uint8_t>(value >> 8);
    destination[2] = static_cast<std::uint8_t>(value >> 16);
    destination[3] = static_cast<std::uint8_t>(value >> 24);
}

void AppendLittleEndian(std::vector<std::uint8_t>& destination,
                        std::uint64_t value,
                        std::size_t width)
{
    for (std::size_t index = 0; index < width; ++index) {
        destination.push_back(static_cast<std::uint8_t>(value >> (index * 8)));
    }
}

void AppendRleRun(std::vector<std::uint8_t>& output,
                  std::uint8_t value,
                  std::size_t count)
{
    if (count == 1) {
        if (value >= kRleFirstMarker) {
            output.push_back(kRleFirstMarker);
        }
        output.push_back(value);
        return;
    }

    while (count > kRleMaxRun) {
        output.push_back(0xFF);
        output.push_back(value);
        count -= kRleMaxRun;
    }

    output.push_back(static_cast<std::uint8_t>(kRleMarkerBase + count));
    output.push_back(value);
}
}

CBaseMessage::CBaseMessage()
    : m_MsgData(kHeaderSize, 0),
      m_lPtr(static_cast<std::int32_t>(kHeaderSize))
{
    SetSize(static_cast<std::uint32_t>(kHeaderSize));
}

std::optional<std::vector<std::uint8_t>> CBaseMessage::DoRLE(std::span<const std::uint8_t> source)
{
    // Оригинальный DoRLE разыменовывал первый байт и не определял безопасную
    // реакцию для пустого входа. Пустой span поэтому не кодируется в wire-данные.
    if (source.empty()) {
        return std::nullopt;
    }

    std::vector<std::uint8_t> output;
    output.reserve(source.size() * 2);

    std::uint8_t runValue = source.front();
    std::size_t runLength = 1;

    for (const std::uint8_t value : source.subspan(1)) {
        if (value == runValue) {
            ++runLength;
            continue;
        }

        AppendRleRun(output, runValue, runLength);
        runValue = value;
        runLength = 1;
    }

    AppendRleRun(output, runValue, runLength);
    return output;
}

bool CBaseMessage::DecodeRLE_SAFE(std::span<const std::uint8_t> source,
                                  std::span<std::uint8_t> destination,
                                  std::size_t& decodedSize)
{
    decodedSize = 0;
    if (source.empty()) {
        return false;
    }

    std::size_t inputOffset = 0;
    while (inputOffset < source.size()) {
        const std::uint8_t marker = source[inputOffset++];
        if (marker < kRleFirstMarker) {
            // Исходная функция требовала оставить минимум один свободный байт.
            if (decodedSize + 1 >= destination.size()) {
                return false;
            }
            destination[decodedSize++] = marker;
            continue;
        }

        // BLOCKED_MISSING_FACT: оригинальные варианты читают следующий байт
        // без bounds-check. Достижимая реакция на trailing marker не доказана;
        // безопасный C++ не читает за пределами входа и сообщает неуспех.
        if (inputOffset >= source.size()) {
            return false;
        }

        const std::uint8_t value = source[inputOffset++];
        const std::size_t runLength = marker - kRleMarkerBase;
        if (decodedSize + runLength >= destination.size()) {
            return false;
        }

        std::memset(destination.data() + decodedSize, value, runLength);
        decodedSize += runLength;
    }

    return true;
}

void CBaseMessage::Update()
{
    // Старый 32-битный процесс записывал младшие 32 бита размера. Размеры выше
    // UINT32_MAX для реального wire-сообщения не считаются подтверждённым путём.
    SetSize(static_cast<std::uint32_t>(m_MsgData.size()));
}

char CBaseMessage::GetChar()
{
    if (!CanRead(1)) {
        return '\0';
    }

    return static_cast<char>(m_MsgData[static_cast<std::size_t>(m_lPtr++)]);
}

std::uint8_t CBaseMessage::GetByte()
{
    if (!CanRead(1)) {
        return 0;
    }

    return m_MsgData[static_cast<std::size_t>(m_lPtr++)];
}

std::int16_t CBaseMessage::GetShort()
{
    if (!CanRead(2)) {
        return 0;
    }

    const auto offset = static_cast<std::size_t>(m_lPtr);
    m_lPtr += 2;
    const std::uint16_t value = static_cast<std::uint16_t>(m_MsgData[offset]) |
                                (static_cast<std::uint16_t>(m_MsgData[offset + 1]) << 8);
    return static_cast<std::int16_t>(value);
}

std::uint16_t CBaseMessage::GetWord()
{
    if (!CanRead(2)) {
        return 0;
    }

    const auto offset = static_cast<std::size_t>(m_lPtr);
    m_lPtr += 2;
    return static_cast<std::uint16_t>(m_MsgData[offset]) |
           (static_cast<std::uint16_t>(m_MsgData[offset + 1]) << 8);
}

std::int32_t CBaseMessage::GetLong()
{
    return static_cast<std::int32_t>(GetULong());
}

std::uint32_t CBaseMessage::GetULong()
{
    if (!CanRead(4)) {
        return 0;
    }

    const auto offset = static_cast<std::size_t>(m_lPtr);
    m_lPtr += 4;
    return ReadLe32(m_MsgData.data() + offset);
}

std::int64_t CBaseMessage::GetLONG64()
{
    if (!CanRead(8)) {
        return 0;
    }

    std::uint64_t value = 0;
    const auto offset = static_cast<std::size_t>(m_lPtr);
    for (std::size_t index = 0; index < 8; ++index) {
        value |= static_cast<std::uint64_t>(m_MsgData[offset + index]) << (index * 8);
    }

    m_lPtr += 8;
    return static_cast<std::int64_t>(value);
}

float CBaseMessage::GetFloat()
{
    if (!CanRead(sizeof(float))) {
        return 0.0F;
    }

    static_assert(sizeof(float) == 4, "Legacy wire requires 32-bit float");
    float value = 0.0F;
    std::memcpy(&value, m_MsgData.data() + m_lPtr, sizeof(value));
    m_lPtr += static_cast<std::int32_t>(sizeof(value));
    return value;
}

void* CBaseMessage::Get(void* destination, std::int32_t size)
{
    if (destination == nullptr || size < 0 || !CanRead(static_cast<std::size_t>(size))) {
        return nullptr;
    }

    if (size != 0) {
        std::memcpy(destination,
                    m_MsgData.data() + static_cast<std::size_t>(m_lPtr),
                    static_cast<std::size_t>(size));
    }
    m_lPtr += size;
    return destination;
}

bool CBaseMessage::GetGUID(CGUID& guid)
{
    if (!CanRead(1)) {
        guid = CGUID::GUID_INVALID;
        return false;
    }

    const std::uint8_t marker = GetByte();
    if (marker == 0) {
        guid = CGUID::GUID_INVALID;
        return false;
    }

    return Get(&guid, static_cast<std::int32_t>(kGuidSize)) != nullptr;
}

void CBaseMessage::Add(char value)
{
    AppendBytes(&value, sizeof(value));
}

void CBaseMessage::Add(std::uint8_t value)
{
    AppendBytes(&value, sizeof(value));
}

void CBaseMessage::Add(std::int16_t value)
{
    AppendLittleEndian(m_MsgData, static_cast<std::uint16_t>(value), 2);
    Update();
}

void CBaseMessage::Add(std::uint16_t value)
{
    AppendLittleEndian(m_MsgData, value, 2);
    Update();
}

void CBaseMessage::Add(std::int32_t value)
{
    Add(static_cast<std::uint32_t>(value));
}

void CBaseMessage::Add(std::uint32_t value)
{
    AppendLittleEndian(m_MsgData, value, 4);
    Update();
}

void CBaseMessage::Add(std::int64_t value)
{
    AppendLittleEndian(m_MsgData, static_cast<std::uint64_t>(value), 8);
    Update();
}

void CBaseMessage::Add(float value)
{
    static_assert(sizeof(float) == 4, "Legacy wire requires 32-bit float");
    std::uint32_t bits = 0;
    std::memcpy(&bits, &value, sizeof(bits));
    Add(bits);
}

void CBaseMessage::Add(const char* value)
{
    if (value == nullptr) {
        return;
    }

    AppendBytes(value, std::strlen(value) + 1);
}

void CBaseMessage::Add(const void* source, std::int32_t size)
{
    if (source == nullptr || size <= 0) {
        return;
    }

    AppendBytes(source, static_cast<std::size_t>(size));
}

void CBaseMessage::AddEx(const void* source, std::int32_t size)
{
    Add(size);
    Add(source, size);
}

void CBaseMessage::Add(const CGUID& guid)
{
    if (guid.IsInvalided()) {
        Add(static_cast<std::uint8_t>(0));
        return;
    }

    Add(static_cast<std::uint8_t>(kGuidSize));
    Add(&guid, static_cast<std::int32_t>(kGuidSize));
}

const std::vector<std::uint8_t>& CBaseMessage::MessageData() const noexcept
{
    return m_MsgData;
}

std::vector<std::uint8_t>& CBaseMessage::MessageData() noexcept
{
    return m_MsgData;
}

const std::uint8_t* CBaseMessage::Data() const noexcept
{
    return m_MsgData.data();
}

std::uint8_t* CBaseMessage::Data() noexcept
{
    return m_MsgData.data();
}

std::size_t CBaseMessage::DataSize() const noexcept
{
    return m_MsgData.size();
}

void CBaseMessage::ResetReadPtr() noexcept
{
    m_lPtr = static_cast<std::int32_t>(kHeaderSize);
}

void CBaseMessage::SetReadPtr(std::int32_t offset) noexcept
{
    m_lPtr = offset;
}

std::int32_t CBaseMessage::GetReadPtr() const noexcept
{
    return m_lPtr;
}

std::uint32_t CBaseMessage::GetSize() const noexcept
{
    return ReadHeaderField(0);
}

std::uint32_t CBaseMessage::GetType() const noexcept
{
    return ReadHeaderField(4);
}

void CBaseMessage::SetType(std::uint32_t value) noexcept
{
    WriteHeaderField(4, value);
}

bool CBaseMessage::CanRead(std::size_t size) const noexcept
{
    if (m_lPtr < 0) {
        return false;
    }

    const auto offset = static_cast<std::size_t>(m_lPtr);
    return offset <= m_MsgData.size() && size <= m_MsgData.size() - offset;
}

void CBaseMessage::AppendBytes(const void* source, std::size_t size)
{
    if (source == nullptr || size == 0) {
        return;
    }

    const auto* first = static_cast<const std::uint8_t*>(source);
    m_MsgData.insert(m_MsgData.end(), first, first + size);
    Update();
}

void CBaseMessage::SetSize(std::uint32_t value) noexcept
{
    WriteHeaderField(0, value);
}

std::uint32_t CBaseMessage::ReadHeaderField(std::size_t offset) const noexcept
{
    if (m_MsgData.size() < offset + sizeof(std::uint32_t)) {
        return 0;
    }

    return ReadLe32(m_MsgData.data() + offset);
}

void CBaseMessage::WriteHeaderField(std::size_t offset, std::uint32_t value) noexcept
{
    if (m_MsgData.size() < offset + sizeof(std::uint32_t)) {
        return;
    }

    WriteLe32(m_MsgData.data() + offset, value);
}

/*
 * Не материализовано до отдельного доказательства:
 *
 * GetStr(char*, long): Billing/Login/Misc/Game/World читают по байту до NUL или
 * лимита. При достижении лимита без NUL зануляется destination[0], но курсор не
 * откатывается; при maxLen <= 0 исходник может обращаться к destination[-1].
 *
 * GetEx(void*, long): Login/Game сначала проверяют cursor + size против общей
 * длины, затем читают ещё 4-байтовый length prefix и копируют size байт без
 * второй эквивалентной bounds-check. Реакция на специально усечённый внешний
 * буфер не доказана. Парный корректный writer AddEx восстановлен выше.
 */
