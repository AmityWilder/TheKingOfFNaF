#include "Input.h"

void UpdateScreencap() {
    BitBlt(INTERNAL_HDC, 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT, DESKTOP_HDC, 0, 0, SRCCOPY);

    BITMAPINFOHEADER bmi = { 0 };
    bmi.biSize = sizeof(BITMAPINFOHEADER);
    bmi.biPlanes = 1;
    bmi.biBitCount = 32;
    bmi.biWidth = SCREEN_WIDTH;
    bmi.biHeight = -SCREEN_HEIGHT;
    bmi.biCompression = BI_RGB;
    bmi.biSizeImage = 0; // 3 * ScreenX * ScreenY; (position, not size)

    GetDIBits(DESKTOP_HDC, H_BITMAP, 0, SCREEN_HEIGHT, SCREEN_DATA, (BITMAPINFO*)&bmi, DIB_RGB_COLORS);
}

unsigned long PixelIndex(long x, long y) {
    return 4 * (unsigned)((y * (long)SCREEN_WIDTH) + x);
}

Color GetPixelColor(long x, long y) {
    unsigned long index = PixelIndex(x, y);

    Color output = {
        SCREEN_DATA[index + 2u],    // Red
        SCREEN_DATA[index + 1u],    // Green
        SCREEN_DATA[index],         // Blue
    };

#ifdef _DEBUG
    SetPixel(CONSOLE_HDC, (int)x, (int)y, output);
#endif

    return output;
}

Color GetPixelColor(POINT pos) {
    return GetPixelColor(pos.x, pos.y);
}
