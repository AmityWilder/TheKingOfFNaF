CC = g++

TheKingOfFNaF.exe: Globals.h Globals.cpp CustomTypes.h CustomTypes.cpp GameActions.h GameActions.cpp Input.h Input.cpp InputProcessing.h InputProcessing.cpp Multithread.h Multithread.cpp Output.h Output.cpp Main.cpp
	$(CC) -Wall -Wextra -pedantic -o bin/TheKingOfFNaF.exe $^
