#include "MultiThread.h"

bool THREADS_SHOULD_LOOP = true;
bool FIRST_TIME_SCREEN_UPDATE = false;

void Produce() {
    while (THREADS_SHOULD_LOOP) {
        UpdateScreencap(); // Update our internal copy of what the gamescreen looks like so we can sample its pixels
        if (!FIRST_TIME_SCREEN_UPDATE) FIRST_TIME_SCREEN_UPDATE = true;
        Sleep(2);
    }
}

void Consume() {
    while (!FIRST_TIME_SCREEN_UPDATE) {
        Sleep(6); // Wait before checking again
    }

    while (THREADS_SHOULD_LOOP) {
        RefreshGameData(); // Using the screencap we just generated, update the game data statuses for decision making

        GAME_STATE.DisplayData(); // Output the data for the user to view
        //BitBlt(g_hConsoleDC, 0, 0, g_screenWidth, g_screenHeight, g_hInternal, 0, 0, SRCCOPY);

        ActOnGameData(); // Based upon the game data, perform all actions necessary to return the game to a neutral state

        Sleep(4);
    }
}

void Direct() {
    while (THREADS_SHOULD_LOOP) {
        /// !! SAFETY !!
        Sleep(2); // Give the user time to move the mouse

        POINT p = GetMousePos(); // Check where the mouse is

        if (!(p.x == 0 && p.y == 0) && // I want to reserve 0,0 for the mouse to reset itself without closing the program
            (p.x == 0 || p.y == 0 || p.x >= SCREEN_WIDTH || p.y >= SCREEN_HEIGHT)) // If the mouse is touching any edge (easy to flick your mouse to those positions)
        {
            std::cout << "User has chosen to reclaim control.\nTask ended.\n"; // Let the user know that the program has ended and why

            THREADS_SHOULD_LOOP = false; // This tells the loop to break
        }
    }
}

void CreateHelpers() {
    std::thread producer(Produce); // Spawn a thread for reading the screen pixels
    std::thread consumer(Consume); // Spawn a thread for acting on that data
    Direct(); // Make sure that the user never has control taken away from them

    // Wait for threads to safely finish their respective functions before destructing them
    producer.join();
    consumer.join();
}
