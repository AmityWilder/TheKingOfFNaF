#pragma once
#include "CustomTypes.h"

/////////////////////////////////////////////////////////////////////////////////////////////
// Here, all of the variables & constants used throughout the project are declared/defined //
/////////////////////////////////////////////////////////////////////////////////////////////

//
// Global variables -- used by everyone, but changeable.
//

// Get a console handle
extern HWND WND_CONSOLE;

// Get a handle to device hdc
extern HDC CONSOLE_HDC;

extern int SCREEN_HEIGHT;
extern int SCREEN_WIDTH;

extern BYTE* SCREEN_DATA;

extern GameState GAME_STATE;

//extern HWND g_gameWindow;
//extern HDC g_gameDC;
extern HDC DESKTOP_HDC; // get the desktop device context
extern HDC INTERNAL_HDC; // create a device context to use ourselves

// create a bitmap
extern HBITMAP H_BITMAP;

//
// Global constants -- These give context to unchanging values
//

// Important positions on the screen
namespace pnt {
    // Clock position
    constexpr POINT CLK_POS = { 1807, 85 };
    constexpr int CLK_10SEC_X = 1832;
    constexpr int CLK_SEC_X = 1849;
    constexpr int CLK_DECISEC_X = 1873;

    constexpr POINT TEMPERATURE_POS = { 1818, 1012 };

    constexpr POINT COINS = { 155, 75 };

    constexpr POINT PWR_POS = { 71, 910 };
    constexpr POINT PWR_USG_POS = { 38, 969 };

    constexpr POINT NOISE_POS = { 38,1020 };

    constexpr POINT openCam = { 1280, 1006 }; // Not really needed since the S key exists...

    // Office
    namespace ofc {
        constexpr POINT MASK_POS = { 682, 1006 };
        constexpr POINT VENT_WARNING_POS = { 1580, 1040 }; // The office version of this constant

        constexpr POINT FOXY = { 801, 710 };
    }

    // Camera
    namespace cam {
        constexpr POINT VENT_WARNING_POS = { 1563, 892 }; // Location for testing vent warning in the cameras
        constexpr POINT resetVent = { 1700, 915 }; // Where the reset vent button is for clicking

        constexpr POINT CAM_01_POS = { 1133, 903 }; // WestHall
        constexpr POINT CAM_02_POS = { 1382, 903 }; // EastHall
        constexpr POINT CAM_03_POS = { 1067, 825 }; // Closet
        constexpr POINT CAM_04_POS = { 1491, 765 }; // Kitchen
        constexpr POINT CAM_05_POS = { 1122, 670 }; // PirateCove
        constexpr POINT CAM_06_POS = { 1422, 590 }; // ShowtimeStage
        constexpr POINT CAM_07_POS = { 1278, 503 }; // PrizeCounter
        constexpr POINT CAM_08_POS = {  988, 495 }; // PartsAndServices

        constexpr int SYS_BTN_X = 1331; // System buttons X position
        constexpr int CAM_SYS_BTN_Y = 153; // Cam system button Y position
        constexpr int VENT_SYS_BTN_Y = 263; // Vent system button Y position
        constexpr int DUCT_SYS_BTN_Y = 373; // Duct system button Y position
    }

    // Vents
    namespace vnt {
        constexpr POINT SNARE_L_POS = { 548, 645 }; // Left snare
        constexpr POINT SNARE_T_POS = { 650, 536 }; // Top snare
        constexpr POINT SNARE_R_POS = { 747, 645 }; // Right snare
    }

    // Ducts
    namespace dct {
        constexpr POINT DUCT_L_POS = {  500, 791 }; // Check left duct
        constexpr POINT DUCT_R_POS = {  777, 791 }; // Check right duct
        constexpr POINT DUCT_L_BTN_POS = {  331, 844 }; // Left duct button
        constexpr POINT DUCT_R_BTN_POS = { 1016, 844 }; // Right duct button
    }
}

// Colors
namespace clr {
    constexpr Color SYS_BTN_COLOR = { 40, 152, 120 };
    constexpr CNorm SYS_BTN_COLOR_NRM = SYS_BTN_COLOR.Normal();
    constexpr Color CAM_BTN_COLOR = { 136, 172, 0 };
    constexpr CNorm CAM_BTN_COLOR_NRM = CAM_BTN_COLOR.Normal();
}

constexpr int CAM_RESP_MS = 300; // Time it takes for the camera to be ready for input

// These enable us to put the buttons in an array and choose from them instead of just using the literal names
// If you're trying to get the position of just the one thing and don't need to do any sort of "switch" thing, please don't use this. It adds additional steps.
enum class Button {
    Mask = 0,
    ResetVent = 1,

    Cam01 = 2, // WestHall
    Cam02 = 3, // EastHall
    Cam03 = 4, // Closet
    Cam04 = 5, // Kitchen
    Cam05 = 6, // PirateCove
    Cam06 = 7, // ShowtimeStage
    Cam07 = 8, // PrizeCounter
    Cam08 = 9, // PartsAndServices

    CameraSystem = 10,
    VentSystem = 11,
    DuctSystem = 12,

    Snare_Left = 13,
    Snare_Top = 14,
    Snare_Right = 15,

    Duct_Left = 16,
    Duct_Right = 17,
};

// Returns the button enum of a camera int
Button CameraButton(int cam);
// Returns the button enum of a camera enum
Button CameraButton(Camera cam);

// Returns the button enum of a system int
Button SystemButton(int system);
// Returns the button enum of a state enum
Button SystemButton(State system);

// This is the list the above enum was referring to
extern const POINT BTN_POSITIONS[18];

// Pick the button position from the list of button positions
POINT GetButtonPos(Button);
