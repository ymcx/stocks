#include "src/model/price.h"

Price model_price_new(double close, double high, double low, double open,
                       double volume) {
  Price price = {close, high, low, open, volume};

  return price;
}
