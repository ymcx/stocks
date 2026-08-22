#include "src/model/price.h"
#include <stdlib.h>

Price *model_price_new(double close, double high, double low, double open,
                       double volume) {
  Price *price = malloc(sizeof(Price));
  price->close = close;
  price->high = high;
  price->low = low;
  price->open = open;
  price->volume = volume;

  return price;
}
