#ifndef MODEL_STOCK_H
#define MODEL_STOCK_H

#include "src/model/price.h"

typedef struct {
  Price *prices;
  int prices_length;
  char *name;
  char *symbol;
} Stock;

Stock *model_stock_new(Price *prices, int prices_length, char *name,
                       char *symbol);

#endif
