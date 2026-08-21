#ifndef MODELS_PRICE_H
#define MODELS_PRICE_H

typedef struct {
  double close;
  double high;
  double low;
  double open;
  double volume;
} Price;

Price models_price_new(double close, double high, double low, double open,
                       double volume);

#endif
