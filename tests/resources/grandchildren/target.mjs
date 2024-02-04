import { spawn } from "child_process";
for (let i = 0; i < 2; i++) {
  spawn("node", ["../../../tests/resources/grandchildren/child.mjs"], {
    stdio: "inherit",
  });
}
setTimeout(() => {}, 5000);
