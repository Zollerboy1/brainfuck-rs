#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

inline size_t nextPowerOfTwo(size_t n) {
    if (n <= 1) return 1;

    size_t p = n - 1;
    size_t z = __builtin_clzl(p);
    return 1ul << (64 - z);
}

void moveRight(char ** cellsPtr, size_t * cellsCountPtr, size_t * currentCellPtr, size_t amount) {
    size_t cellsCount = *cellsCountPtr;
    if (cellsCount <= (*currentCellPtr += amount)) {
        size_t newCellsCount = nextPowerOfTwo(*currentCellPtr + 1);

        *cellsPtr = (char *)realloc(*cellsPtr, newCellsCount);
        memset(*cellsPtr + cellsCount, 0, newCellsCount - cellsCount);

        *cellsCountPtr = newCellsCount;
    }
}

void moveRightUntilZero(char ** cellsPtr, size_t * cellsCountPtr, size_t * currentCellPtr, size_t amount) {
    char * cells = *cellsPtr;
    size_t cellsCount = *cellsCountPtr;
    size_t currentCell = *currentCellPtr;

    while (cells[currentCell] != 0) {
        if (cellsCount <= (currentCell += amount)) {
            size_t newCellsCount = nextPowerOfTwo(currentCell + 1);

            *cellsPtr = (char *)realloc(*cellsPtr, newCellsCount);
            memset(*cellsPtr + cellsCount, 0, newCellsCount - cellsCount);

            *cellsCountPtr = newCellsCount;
            break;
        }
    }

    *currentCellPtr = currentCell;
}

bool moveLeftUntilZero(char * cells, size_t * currentCellPtr, size_t amount) {
    size_t currentCell = *currentCellPtr;

    while (cells[currentCell] != 0) {
        if (currentCell < amount) return true;
        currentCell -= amount;
    }

    *currentCellPtr = currentCell;
    return false;
}

void input(char * cells, size_t currentCell, char ** inputBufferPtr, size_t * lengthPtr, size_t * bufferLengthPtr, char ** inputPositionPtr) {
    size_t currentLength = *lengthPtr - (*inputPositionPtr - *inputBufferPtr);
    if (currentLength == 0) {
        *lengthPtr = getline(inputBufferPtr, bufferLengthPtr, stdin);
        *inputPositionPtr = *inputBufferPtr;
    }

    char currentChar = *(*inputPositionPtr)++;
    cells[currentCell] = currentChar;
}
