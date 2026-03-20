#include "Globals.h"

HWND WND_CONSOLE = GetConsoleWindow(); // Get a console handle
HDC CONSOLE_HDC = GetDC(WND_CONSOLE); // Get a handle to device hdc

int SCREEN_HEIGHT = GetSystemMetrics(SM_CYVIRTUALSCREEN);
int SCREEN_WIDTH = GetSystemMetrics(SM_CXVIRTUALSCREEN);

BYTE* SCREEN_DATA = new BYTE[CHANNELS_PER_COLOR * (size_t)SCREEN_WIDTH * (size_t)SCREEN_HEIGHT];

GameState GAME_STATE = GameState(); // All the information we have about the state of the game

//HWND g_gameWindow = FindWindow(NULL, TEXT("Ultimate Custom Night"));
//HDC g_gameDC = GetDC(g_gameWindow);
HDC DESKTOP_HDC = GetDC(NULL); // get the desktop device context
HDC INTERNAL_HDC = CreateCompatibleDC(DESKTOP_HDC); // create a device context to use ourselves

HBITMAP H_BITMAP = CreateCompatibleBitmap(DESKTOP_HDC, SCREEN_WIDTH, SCREEN_HEIGHT);

const POINT BTN_POSITIONS[] = {
    pnt::ofc::MASK_POS,
    pnt::cam::RESET_VENT_BTN_POS,
    pnt::cam::CAM_01_POS,
    pnt::cam::CAM_02_POS,
    pnt::cam::CAM_03_POS,
    pnt::cam::CAM_04_POS,
    pnt::cam::CAM_05_POS,
    pnt::cam::CAM_06_POS,
    pnt::cam::CAM_07_POS,
    pnt::cam::CAM_08_POS,
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::CAM_SYS_BTN_Y
	},
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::VENT_SYS_BTN_Y
	},
    {
		pnt::cam::SYS_BTN_X,
		pnt::cam::DUCT_SYS_BTN_Y
	},
    pnt::vnt::SNARE_L_POS,
    pnt::vnt::SNARE_T_POS,
    pnt::vnt::SNARE_R_POS,
    pnt::dct::DUCT_L_BTN_POS,
    pnt::dct::DUCT_R_BTN_POS
};

Button CameraButton(int cam) {
    return Button((int)Button::Cam01 + cam);
}
Button CameraButton(Camera cam) {
    return CameraButton((int)cam);
}

Button SystemButton(int system) {
    return Button((int)Button::CameraSystem + system);
}
Button SystemButton(State system) {
    return SystemButton((int)system);
}

POINT GetButtonPos(Button button) {
    return BTN_POSITIONS[(int)button];
}
