#include "readwrite.h"

#include <filesystem>
#include <fstream>
#include <mutex>
#include <string>

namespace
{
std::mutex g_AppendFileMutex;
}

bool ReadTo(std::istream& stream, const char* name)
{
    // ПОДТВЕРЖДЕНО АССЕМБЛЕРОМ 0x00420AC0: первый токен читается безусловно.
    std::string token;
    if (!(stream >> token)) {
        return false;
    }

    while (token != name) {
        // Прямой владелец проверяет stream.eof() перед буквальным терминатором.
        // Проверка полного stream-state — техническая Linux-граница против
        // бесконечного цикла при badbit без EOF; на допустимом вводе порядок тот же.
        if (token == "<end>" || !(stream >> token)) {
            return false;
        }
    }
    return true;
}

void PutStringToFile(std::string_view filePrefix, std::string_view text)
{
    if (filePrefix.empty()) {
        return;
    }

    std::lock_guard<std::mutex> lock(g_AppendFileMutex);
    std::error_code error;
    std::filesystem::create_directories("log", error);
    std::ofstream output(std::filesystem::path("log") /
                             (std::string(filePrefix) + ".txt"),
                         std::ios::app | std::ios::binary);
    if (output.is_open()) {
        output << text << '\n';
    }
}
