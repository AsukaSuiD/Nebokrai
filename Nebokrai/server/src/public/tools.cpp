#include "tools.h"

#include <cstdint>
#include <cstdio>
#include <limits>
#include <string>

namespace
{
char LowerAscii(char value) noexcept
{
    return value >= 'A' && value <= 'Z'
               ? static_cast<char>(value - 'A' + 'a')
               : value;
}

bool EqualAsciiCaseInsensitive(std::string_view left,
                               std::string_view right) noexcept
{
    if (left.size() != right.size()) {
        return false;
    }
    for (std::size_t index = 0; index < left.size(); ++index) {
        if (LowerAscii(left[index]) != LowerAscii(right[index])) {
            return false;
        }
    }
    return true;
}
}

int GetFileLength(char* name)
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00420710: null -> 0, ошибка открытия -> -1;
    // иначе возвращаются длина открытого файла и результат закрытия.
    if (name == nullptr) {
        return 0;
    }

    std::FILE* file = std::fopen(name, "rb");
    if (file == nullptr) {
        return -1;
    }
    if (std::fseek(file, 0, SEEK_END) != 0) {
        std::fclose(file);
        return -1;
    }
    const long length = std::ftell(file);
    std::fclose(file);
    if (length < 0 ||
        static_cast<unsigned long>(length) >
            static_cast<unsigned long>(std::numeric_limits<int>::max())) {
        // Старый 32-битный _filelength тоже не мог представить setup.dat
        // большего размера.
        return -1;
    }
    return static_cast<int>(length);
}

void IniDecoder(char* input, char* output, int length) noexcept
{
    if (input == nullptr || output == nullptr || length <= 0) {
        // Прямой вызывающий код всегда передаёт допустимые буферы; эта проверка
        // лишь заменяет старое неопределённое поведение null/OOB на технической
        // границе.
        return;
    }

    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00420A90:
    //   dl = input[i]; sub dl,0x0C; not dl; output[i] = dl.
    for (int index = 0; index < length; ++index) {
        const std::uint8_t source =
            static_cast<std::uint8_t>(static_cast<unsigned char>(input[index]));
        const std::uint8_t subtracted =
            static_cast<std::uint8_t>(source - std::uint8_t{0x0C});
        output[index] = static_cast<char>(
            static_cast<std::uint8_t>(~subtracted));
    }
}

std::optional<std::filesystem::path> ResolveLegacyFileAsciiCase(
    const std::filesystem::path& directory,
    std::string_view requestedName,
    std::error_code& error)
{
    error.clear();
    const auto direct = directory / std::string(requestedName);
    if (std::filesystem::exists(direct, error)) {
        if (std::filesystem::is_regular_file(direct, error)) {
            return direct;
        }
    }
    if (error) {
        return std::nullopt;
    }

    std::filesystem::directory_iterator iterator(directory, error);
    const std::filesystem::directory_iterator end;
    while (!error && iterator != end) {
        const auto filename = iterator->path().filename().string();
        if (EqualAsciiCaseInsensitive(filename, requestedName)) {
            if (iterator->is_regular_file(error)) {
                return iterator->path();
            }
            if (error) {
                return std::nullopt;
            }
        }
        iterator.increment(error);
    }
    if (error) {
        return std::nullopt;
    }

    // Сохраняем обычную ошибку открытия/чтения на прямом пути, как поздняя
    // реконструкция: отсутствие файла не превращается в новый тип результата.
    return direct;
}
