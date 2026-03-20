#include "CustomTypes.h"
#include <cmath>
#include <iomanip>

unsigned short const& ClockTime::GetDeciseconds() const {
    return deciseconds;
}

unsigned short ClockTime::GetSeconds() const {
    return (deciseconds / 10);
}

unsigned char ClockTime::GetMinutes() const {
    return (unsigned char)(GetSeconds() / 60); // 60 seconds (realtime)
}

unsigned char ClockTime::GetHour() const {
    return (unsigned char)(GetSeconds() / 45); // 45 seconds (gametime)
}

unsigned short ClockTime::GetWholeHourDeciseconds() const {
    return ((unsigned short)GetHour() * 450);
}

unsigned short ClockTime::GetDecisecondsSinceHour() const {
    return (GetDeciseconds() - GetWholeHourDeciseconds());
}

void ClockTime::UpdateTime(unsigned short const& newTime) {
    if (newTime > deciseconds && newTime < 6000 && newTime > 0 && ((newTime - deciseconds) < 10 || pingsSinceChange > 10)) {
        deciseconds = newTime;
        pingsSinceChange = 0;
    } else ++pingsSinceChange;
}

int const& ClockTime::GetPingsSinceChange() {
    return pingsSinceChange;
}

unsigned char Color::Gray() const {
    return (unsigned char)(((int)r + (int)g + (int)b) / 3);
}

unsigned char Color::RedDev() const {
    int distFromMean = (r - Gray());
    return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
}

unsigned char Color::GreenDev() const {
    int distFromMean = (g - Gray());
    return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
}

unsigned char Color::BlueDev() const {
    int distFromMean = (b - Gray());
    return (unsigned char)sqrt((distFromMean * distFromMean) / 3);
}

void GameState::DisplayData() {
    std::cout << "\x1b[0;0HTime: "
        << (int)(gameData.time.GetMinutes())
        << ':' << (int)(gameData.time.GetSeconds() % 60)
        << '.' << (int)(gameData.time.GetDeciseconds() % 10)
        << "\n\nStatuses\n========\nVentilation " << std::setw(7) << (gameData.ventilationNeedsReset ? "WARNING" : "good")
        << "\nLeft  door  " << std::setw(6) << (gameData.doorsClosed[0] ? "closed" : "open")
        << "\nFront vent  " << std::setw(6) << (gameData.doorsClosed[1] ? "closed" : "open")
        << "\nRight door  " << std::setw(6) << (gameData.doorsClosed[2] ? "closed" : "open")
        << "\nRight vent  " << std::setw(6) << (gameData.doorsClosed[3] ? "closed" : "open")
        << "\nFlashlight  " << std::setw(3) << (gameData.flashlight ? "on" : "off")
        << "\nGamestate\n=========\nState: " << std::setw(6) << state << "\n                                 \x1b[1G";
    switch (state) {
        case State::Camera:
            std::cout << "Looking at: " << std::setw(18) << stateData.cd.camera;
            break;

        case State::Office:
            std::cout << "Yaw: " << stateData.od.officeYaw;
            break;

        default:
            std::cout << "TODO";
            break;
    }
    std::cout << "\n\n";
}

void GameState::Init() {
    state = State::Office;
    stateData.cd.camera = Camera::WestHall;
    gameData.doorsClosed[0] = false;
    gameData.doorsClosed[1] = false;
    gameData.doorsClosed[2] = false;
    gameData.doorsClosed[3] = false;
    gameData.time = ClockTime();
    gameData.ventilationNeedsReset = false;
    gameData.flashlight = false;
}

ColorHSL Color::ToHSL() const {
    CNorm col = Normal();

    double cmax;
    int cmaxComp;
    double cmin;

    if (col.r > col.g) {
        if (col.r > col.b) cmaxComp = 0;
        else cmaxComp = 2; // col.r < col.b
    } else { // col.r < col.g
        if (col.g > col.b) cmaxComp = 1;
        else cmaxComp = 2; // col.g < col.b
    }

    switch (cmaxComp) {
        case 0: cmax = col.r; break;
        case 1: cmax = col.g; break;
        case 2: cmax = col.b; break;
    }
    cmin = std::min(col.r, std::min(col.g, col.b));
    double delta = cmax - cmin;

    double h, s, l;

    // Hue
    if (delta == 0.0) h = 0.0;
    else {
        switch (cmaxComp) {
            case 0: // Red
                h = 60.0 * ((col.g - col.b) / delta);
                break;
            case 1: // Green
                h = 60.0 * (((col.b - col.r) / delta) + 2.0);
                break;
            case 2: // Blue
                h = 60.0 * (((col.r - col.g) / delta) + 4.0);
                break;
        }
    }

    // Lum
    l = (cmax + cmin) / 2.0;

    // Sat
    if (delta == 0.0) s = 0;
    else s = delta / (1 - abs(2 * l - 1));

    // Finished
    return { h,s,l };
}
