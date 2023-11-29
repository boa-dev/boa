globalThis.__hack_print_output = "";
globalThis.__boa_no_color = () => true;
globalThis.console = new globalThis.console.Console(
  (s) => (globalThis.__hack_print_output += s),
);
