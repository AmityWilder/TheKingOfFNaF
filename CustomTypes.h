#pragma once
#include <Windows.h>
#include <iostream>
#include <stdlib.h>

////////////////////////////////////////////////////
// Here we declare/define the non-primitive types //
////////////////////////////////////////////////////

// Normalized RGB color
struct CNorm {
    double r, g, b;

    // Better for determining how close a color is to another, regardless of the scale. (brightness/darkness)
    constexpr double CDot(CNorm rhs) const {
        return r * rhs.r + g * rhs.g + b * rhs.b;
    }
};

struct ColorHSL {
    double hue; // A degree on the color wheel [0..360]
    double sat; // Percentage of color [0..100]
    double lum; // Percentage of brightness [0..100]
};

// 24-bit RGB color
struct Color {
    unsigned char r, g, b;

    unsigned char Gray() const;
    unsigned char RedDev() const;
    unsigned char GreenDev() const;
    unsigned char BlueDev() const;

    constexpr CNorm Normal() const {
        return {
            (double)r / 255.0,
            (double)g / 255.0,
            (double)b / 255.0,
        };
    }

    constexpr operator COLORREF() const {
        return RGB(r,g,b);
    }

    ColorHSL ToHSL() const;

    constexpr double CDot(Color rhs) const {
        return Normal().CDot(rhs.Normal());
    }
};

class ClockTime
{
private:
    // One hour is 45 seconds. A night is 4 minutes 30 seconds, or 270 seconds -- 2700 deciseconds. This can be expressed in 12 bits as 0b101010001100.
    unsigned short deciseconds;
    int pingsSinceChange;

public:
    ClockTime() :
        deciseconds{ 0u },
        pingsSinceChange{ 0 }
    {};

    ClockTime(unsigned short const& deciseconds) :
        deciseconds{ deciseconds },
        pingsSinceChange{ 0 }
    {};

    unsigned short const& GetDeciseconds() const; // Read-only
    unsigned short GetSeconds() const; // It takes 1 bit more than a char to describe the number of seconds in a night.
    unsigned char GetMinutes() const; // Not sure what we'd need this for, but just in case.
    unsigned char GetHour() const; // What hour of the night we are at

    unsigned short GetWholeHourDeciseconds() const; // Converts hours to deciseconds, for finding how many deciseconds we are through the current hour.
    unsigned short GetDecisecondsSinceHour() const; // Finds how many deciseconds into the current hour we are.

    void UpdateTime(unsigned short const&);
    int const& GetPingsSinceChange();
};

// What gamestate we are in (what we can see on the screen)
enum class State : unsigned char {
    Camera = 0,
    Vent,
    Duct,
    Office,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, State state) {
        switch (state) {
            case State::Office: return stream << "Office";
            case State::Camera: return stream << "Camera";
            case State::Vent: return stream << "Vent";
            case State::Duct: return stream << "Duct";
            default: return stream << "Error";
        }
    }
}

enum class Camera : unsigned char {
    WestHall = 0,
    EastHall,
    Closet,
    Kitchen,
    PirateCove,
    ShowtimeStage,
    PrizeCounter,
    PartsAndServices,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Camera cam) {
        switch (cam) {
            case Camera::EastHall: return stream << "East hall";
            case Camera::Kitchen: return stream << "Kitchen";
            case Camera::PartsAndServices: return stream << "Parts and services";
            case Camera::PirateCove: return stream << "Pirate cove";
            case Camera::PrizeCounter: return stream << "Prize counter";
            case Camera::ShowtimeStage: return stream << "Showtime stage";
            case Camera::WestHall: return stream << "West hall";
            case Camera::Closet: return stream << "Supply closet";
            default: return stream << "Error";
        }
    }
}

enum class Vent : unsigned char {
    Inactive = 0, // Snares reset after being tripped
    WestSnare,
    NorthSnare,
    EastSnare,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Vent vent) {
        switch (vent) {
            case Vent::Inactive: return stream << "Inactive";
            case Vent::WestSnare: return stream << "West snare";
            case Vent::NorthSnare: return stream << "North snare";
            case Vent::EastSnare: return stream << "East snare";
            default: return stream << "Error";
        }
    }
}

enum class Duct : bool {
    West = false,
    East = true,
};

namespace std {
    std::ostream& operator<<(std::ostream& stream, Duct duct) {
        switch (duct) {
            case Duct::West: return stream << "West";
            case Duct::East: return stream << "East";
            default: return stream << "Error";
        }
    }
}

struct GameState {
    State state; // What state we are in (office, checking cameras, ducts, vents)

    union StateData { // The metadata about the state (what part of the office, which camera)
        struct OfficeData {
            double officeYaw; // How far left/right we are looking [-1,1]
        } od;

        struct CamData {
            Camera camera; // Which camera we are looking at
        } cd;

        struct VentData {
            Vent ventSnare; // Which vent snare is active
        } vd;

        struct DuctData {
            Duct closedDuct; // Which duct is currently closed
            POINT audioLure;
        } dd;
    } stateData; // Information about the current state that can tell us how to interpret information

    // This is the type which actually stores the data we have about the gamestate
    struct GameData {
        ClockTime time;
        bool ventilationNeedsReset;

        bool doorsClosed[4]; // In order from left to right
        bool flashlight;
    } gameData;

    void DisplayData();
    void Init();
};
