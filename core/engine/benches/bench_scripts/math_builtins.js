(function () {
  let sum = 0;
  for (let i = 0; i < 1000; i++) {
    sum += Math.abs(-i);
    sum += Math.floor(i + 0.5);
    sum += Math.ceil(i - 0.5);
    sum += Math.round(i + 0.3);
    sum += Math.sqrt(i);
    sum += Math.pow(i, 2);
    sum += Math.log(i + 1);
    sum += Math.exp(i / 100);
    sum += Math.log2(i + 1);
    sum += Math.log10(i + 1);
    sum += Math.sin(i);
    sum += Math.cos(i);
    sum += Math.max(i, i + 1);
    sum += Math.min(i, i + 1);
  }
  return sum;
})();
