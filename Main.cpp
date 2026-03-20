#include <windows.h>
#include "Globals.h"
#include "CustomTypes.h"
#include "Input.h"
#include "Output.h"
#include "GameActions.h"
#include "MultiThread.h"

int main() {
    // SETUP //

    WND_CONSOLE = GetConsoleWindow(); // Get a console handle
    CONSOLE_HDC = GetDC(WND_CONSOLE); // Get a handle to device hdc

    SCREEN_HEIGHT = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    SCREEN_WIDTH = GetSystemMetrics(SM_CXVIRTUALSCREEN);

    SCREEN_DATA = new BYTE[CHANNELS_PER_COLOR * (size_t)SCREEN_WIDTH * (size_t)SCREEN_HEIGHT];

    DESKTOP_HDC = GetDC(NULL); // get the desktop device context
    INTERNAL_HDC = CreateCompatibleDC(DESKTOP_HDC); // create a device context to use ourselves

    H_BITMAP = CreateCompatibleBitmap(DESKTOP_HDC, SCREEN_WIDTH, SCREEN_HEIGHT);

    SelectObject(INTERNAL_HDC, H_BITMAP); // Get a handle to our bitmap

    // GAME LOOP //

    // TODO: multithread
    while (true) {
        /// !! SAFETY !!
        Sleep(2); // Give the user time to move the mouse

        POINT p = GetMousePos(); // Check where the mouse is

        if (!(p.x == 0 && p.y == 0) && // I want to reserve 0,0 for the mouse to reset itself without closing the program
            (p.x == 0 || p.y == 0 || p.x >= SCREEN_WIDTH || p.y >= SCREEN_HEIGHT)) // If the mouse is touching any edge (easy to flick your mouse to those positions)
        {
            std::cout << "User has chosen to reclaim control.\nTask ended.\n"; // Let the user know that the program has ended and why
            break;
        }

        UpdateScreencap(); // Update our internal copy of what the gamescreen looks like so we can sample its pixels

        RefreshGameData(); // Using the screencap we just generated, update the game data statuses for decision making

        GAME_STATE.DisplayData(); // Output the data for the user to view
        //BitBlt(g_hConsoleDC, 0, 0, g_screenWidth, g_screenHeight, g_hInternal, 0, 0, SRCCOPY);

        ActOnGameData(); // Based upon the game data, perform all actions necessary to return the game to a neutral state
    }

    // WRAP UP //

    DeleteObject(H_BITMAP); // Free the bitmap memory to the OS

    DeleteDC(INTERNAL_HDC); // Destroy our internal display handle

    ReleaseDC(NULL, DESKTOP_HDC); // Free the desktop handle

    delete[] SCREEN_DATA;

    return 0;
}
