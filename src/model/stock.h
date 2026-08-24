#ifndef MODEL_STOCK_H
#define MODEL_STOCK_H

#include "src/model/price.h"
#include <stddef.h>

typedef struct {
  Price **prices;
  size_t prices_length;
  char *name;
  char *symbol;
} Stock;

Stock *model_stock_new(Price **prices, size_t prices_length, char *name,
                       char *symbol);
void model_stock_free(Stock *stock);

#endif
