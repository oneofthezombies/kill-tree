import { spawn } from "child_process";
spawn("node", ["../../../tests/resources/grandchild/child.mjs"], {
  stdio: "inherit",
});
setTimeout(() => {}, 5000);
