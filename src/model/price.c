#include "src/model/price.h"
#include <stdlib.h>

Price *model_price_new(double close, double high, double low, double open,
                       double volume) {
  Price *price = malloc(sizeof(Price));
  if (!price) {
    return NULL;
  }

  price->close = close;
  price->high = high;
  price->low = low;
  price->open = open;
  price->volume = volume;

  return price;
}

void model_price_free(Price *price) { free(price); }
