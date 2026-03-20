#include <windows.h>
#include "Globals.h"
#include "CustomTypes.h"
#include "Input.h"
#include "Output.h"
#include "GameActions.h"
#include "MultiThread.h"

int main() {
    // SETUP //

    SelectObject(INTERNAL_HDC, H_BITMAP); // Get a handle to our bitmap

    // GAME LOOP //

    CreateHelpers(); // Creates multiple threads for performing tasks

    // WRAP UP //

    ReleaseDC(NULL, DESKTOP_HDC); // Free the desktop handle

    DeleteObject(H_BITMAP); // Free the bitmap memory to the OS

    DeleteDC(INTERNAL_HDC); // Destroy our internal display handle

    return 0;
}
