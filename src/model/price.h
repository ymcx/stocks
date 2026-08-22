#ifndef MODEL_PRICE_H
#define MODEL_PRICE_H

typedef struct {
  double close;
  double high;
  double low;
  double open;
  double volume;
} Price;

Price *model_price_new(double close, double high, double low, double open,
                       double volume);

#endif
