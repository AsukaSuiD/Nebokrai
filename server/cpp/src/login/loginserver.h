#pragma once

#include <cstdint>
#include <filesystem>
#include <memory>
#include <string>

/*
 * Исходный компонентный владелец: LoginServer WinMain/GameThreadFunc и
 * CGame::Init/MainLoop/Release из LoginServer/loginserver.exe + PDB.
 *
 * Этот файл не вводит новый игровой сервис: он соединяет восстановленные
 * owner-классы LoginServer в исходный процесс. Windows thread/message-pump
 * заменены пошаговым Linux runtime на Asio; message routing, AuthManager,
 * CLoginQueue, GAS, account/server logs, World ping, Login DB и порядок
 * освобождения остаются у своих владельцев.
 */
namespace Login
{
enum class LoginServerStatus
{
    Ok,
    ConfigurationError,
    DatabaseError,
    NetworkError,
    RuntimeError,
};

struct LoginServerResult
{
    LoginServerStatus status{LoginServerStatus::Ok};
    std::string detail;

    [[nodiscard]] explicit operator bool() const noexcept
    {
        return status == LoginServerStatus::Ok;
    }
};

class LoginServer
{
public:
    LoginServer();
    ~LoginServer();

    LoginServer(const LoginServer&) = delete;
    LoginServer& operator=(const LoginServer&) = delete;

    [[nodiscard]] LoginServerResult Initialize(
        const std::filesystem::path& runtimeDirectory);
    [[nodiscard]] LoginServerResult RunTurn();
    [[nodiscard]] LoginServerResult Release();

private:
    class Impl;
    std::unique_ptr<Impl> m_Impl;
};
}
