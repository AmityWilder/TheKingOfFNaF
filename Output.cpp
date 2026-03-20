#include "Output.h"

void SimulateKey(VirtualKey key) {
    INPUT input;

    input.type = INPUT_KEYBOARD;
    input.ki.wScan = 0;
    input.ki.dwExtraInfo = 0;
    input.ki.wVk = (int)key;
    input.ki.dwFlags = 0;
    SendInput(1, &input, sizeof(INPUT));

    Sleep(10);

    input.ki.dwFlags = KEYEVENTF_KEYUP;
    SendInput(1, &input, sizeof(INPUT));
    Sleep(2);
}

POINT GetMousePos() {
    POINT p;
    if (GetCursorPos(&p)) return p;
    else return { 0,0 };
}

void SimulateMouseMove(long x, long y) {
    INPUT input;
    input.type = INPUT_MOUSE;
    input.mi.mouseData = 0;
    input.mi.time = 0; // Pleaseeeee don't mess with this... it makes the monitor go funky...
    input.mi.dx = x;
    input.mi.dy = y;
    input.mi.dwFlags = MOUSEEVENTF_MOVE;
    SendInput(1, &input, sizeof(input));
}

void SimulateMouseMove(POINT p) { SimulateMouseMove(p.x, p.y); }

void SimulateMouseGoto(long x, long y) {
    INPUT input;
    input.type = INPUT_MOUSE;
    input.mi.mouseData = 0;
    input.mi.time = 0; // Pleaseeeee don't mess with this... it makes the monitor go funky...
    input.mi.dx = x * 34;
    input.mi.dy = y * 61;
    input.mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;
    SendInput(1, &input, sizeof(input));
}

void SimulateMouseGoto(POINT p) {
    SimulateMouseGoto(p.x, p.y);
}

void SimulateMouseClick() {
    INPUT input;
    input.type = INPUT_MOUSE;
    input.mi.mouseData = 0;
    input.mi.time = 0;
    input.mi.dx = 0;
    input.mi.dy = 0;
    input.mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
    SendInput(1, &input, sizeof(input));

    Sleep(10);

    input.type = INPUT_MOUSE;
    input.mi.mouseData = 0;
    input.mi.time = 0;
    input.mi.dx = 0;
    input.mi.dy = 0;
    input.mi.dwFlags = MOUSEEVENTF_LEFTUP;
    SendInput(1, &input, sizeof(INPUT));
}

void SimulateMouseClickAt(POINT p) {
    SimulateMouseGoto(p);
    SimulateMouseClick();
}

void OfficeLookLeft(void) {
    if (GAME_STATE.state == State::Office) { // We cannot look left/right in cameras
        SimulateMouseGoto(8, 540);
        Sleep(500); // 5 deciseconds
    }
}

void OfficeLookRight(void) {
    if (GAME_STATE.state == State::Office) { // We cannot look left/right in cameras
        SimulateMouseGoto(1910, 540);
        Sleep(500); // 5 deciseconds
    }
}
