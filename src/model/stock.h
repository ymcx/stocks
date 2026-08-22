#ifndef MODEL_STOCK_H
#define MODEL_STOCK_H

#include "glib.h"
#include "src/model/price.h"

typedef struct {
  Price *prices;
  int prices_length;
  gchar *name;
  gchar *symbol;
} Stock;

Stock *model_stock_new(Price *prices, int prices_length, gchar *name,
                       gchar *symbol) ;

#endif
