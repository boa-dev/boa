/**
 * Calculate a fibonacci number by calling callbacks with intermediate results,
 * switching between Rust and JavaScript.
 * @param {number} a The fibonacci number to calculate.
 * @param {function} callback_a A callback method.
 * @param {function} callback_b A callback method.
 * @returns {number} The {a}th fibonacci number.
 */
export function fibonacci(a, callback_a, callback_b) {
  if (a <= 1) {
    return a;
  }

  // Switch the callbacks around.
  return (
    callback_a(a - 1, callback_b, callback_a) +
    callback_b(a - 2, callback_b, callback_a)
  );
}
