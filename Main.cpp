#include "Globals.h"
#include "CustomTypes.h"
#include "Input.h"
#include "Output.h"
#include "GameActions.h"
#include "MultiThread.h"

int main() {
    // SETUP //

    GAME_STATE.Init(); // Set all values in the gamestate to their defaults

    SelectObject(INTERNAL_HDC, H_BITMAP); // I'm not 100% sure what this does but I know it's important

    // GAME LOOP //

    CreateHelpers(); // Creates multiple threads for performing tasks

    // WRAPUP //

    ReleaseDC(NULL, DESKTOP_HDC); // Return the desktop handle

    DeleteObject(H_BITMAP); // Return the bitmap memory to the OS

    DeleteDC(INTERNAL_HDC); // Destroy our internal display handle

    return 0; // Must have a return
}
