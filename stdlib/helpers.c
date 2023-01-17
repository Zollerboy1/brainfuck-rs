#include <stdbool.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static inline size_t nextPowerOfTwo(size_t n) {
    if (n <= 1) return 1;

    size_t p = n - 1;
    size_t z = __builtin_clzl(p);
    return 1ul << (64 - z);
}

extern void moveRight(char ** cellsPtr, size_t * cellsCountPtr, size_t * currentCellPtr, size_t amount) {
    size_t cellsCount = *cellsCountPtr;
    if (cellsCount <= (*currentCellPtr += amount)) {
        size_t newCellsCount = nextPowerOfTwo(*currentCellPtr + 1);

        *cellsPtr = (char *)realloc(*cellsPtr, newCellsCount);
        memset(*cellsPtr + cellsCount, 0, newCellsCount - cellsCount);

        *cellsCountPtr = newCellsCount;
    }
}

extern void input(char * cells, size_t currentCell, char ** inputBufferPtr) {
    static char * inputPosition = NULL;
    static size_t length = 0, bufferLength = 0;

    size_t currentLength = length - (inputPosition - *inputBufferPtr);
    if (currentLength == 0) {
        length = getline(inputBufferPtr, &bufferLength, stdin);
        inputPosition = *inputBufferPtr;
    }

    char currentChar = *(inputPosition++);
    cells[currentCell] = currentChar;
}

extern void moveRightUntilZero(char ** cellsPtr, size_t * cellsCountPtr, size_t * currentCellPtr, size_t stepSize) {
    char * cells = *cellsPtr;
    size_t cellsCount = *cellsCountPtr;
    size_t currentCell = *currentCellPtr;

    while (cells[currentCell] != 0) {
        if (cellsCount <= (currentCell += stepSize)) {
            size_t newCellsCount = nextPowerOfTwo(currentCell + 1);

            *cellsPtr = (char *)realloc(*cellsPtr, newCellsCount);
            memset(*cellsPtr + cellsCount, 0, newCellsCount - cellsCount);

            *cellsCountPtr = newCellsCount;
            break;
        }
    }

    *currentCellPtr = currentCell;
}

extern bool moveLeftUntilZero(char * cells, size_t * currentCellPtr, size_t stepSize) {
    size_t currentCell = *currentCellPtr;

    while (cells[currentCell] != 0) {
        if (currentCell < stepSize) return true;
        currentCell -= stepSize;
    }

    *currentCellPtr = currentCell;
    return false;
}

extern void moveValueRight(char ** cellsPtr, size_t * cellsCountPtr, size_t currentCell, size_t amount) {
    char * cells = *cellsPtr;
    char value = cells[currentCell];

    size_t destinationCell = currentCell;
    moveRight(cellsPtr, cellsCountPtr, &destinationCell, amount);

    cells = *cellsPtr;
    cells[currentCell] = 0;
    cells[destinationCell] += value;
}

extern bool moveValueLeft(char * cells, size_t currentCell, size_t amount) {
    char value = cells[currentCell];

    if (currentCell < amount) return true;

    cells[currentCell] = 0;
    cells[currentCell - amount] += value;
    return false;
}
