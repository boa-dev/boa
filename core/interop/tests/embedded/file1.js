import { foo } from "./file2.js";
import { foo as foo2 } from "./dir1/file3.js";

export function bar() {
  return foo() + foo2() + 1;
}
