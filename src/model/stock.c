#include "src/model/stock.h"
#include "glib.h"
#include "src/model/price.h"

Stock *model_stock_new(Price *prices, int prices_length, gchar *name,
                       gchar *symbol) {
  Stock*stock =malloc(sizeof(Stock));
  stock->prices=prices;
  stock->prices_length=prices_length;
  stock->name=name;

  stock->symbol=symbol;

  return stock;
}
