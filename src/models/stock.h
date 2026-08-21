#ifndef MODELS_STOCK_H
#define MODELS_STOCK_H

#include "glib.h"
#include "src/models/price.h"

typedef struct {
  Price *prices;
  int prices_length;
  gchar *name;
  gchar *symbol;
} Stock;

Stock *models_stock_new(Price *prices, int prices_length, gchar *name,
                       gchar *symbol) ;

#endif
