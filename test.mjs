import { spawn } from "child_process";

async function main() {
  const port = 50000;
  const elapseds = [];
  for (let i = 0; i < 1000; i++) {
    console.log(`Iteration ${i}`);
    const targetPort = port + i;
    await new Promise((resolve) => {
      const target = spawn("node", ["server.mjs", targetPort], {
        stdio: "inherit",
      });

      target.on("spawn", () => {
        console.log(`Spawned server ${target.pid}`);
        const start = Date.now();
        const kill_tree = spawn("target/release/tokio", [target.pid], {
          stdio: "inherit",
        });
        kill_tree.on("exit", () => {
          const end = Date.now();
          const elapsed = end - start;
          console.log(`Killed server ${target.pid} in ${elapsed}ms`);
          elapseds.push(elapsed);
          resolve();
        });
      });
    });
  }

  const total = elapseds.reduce((a, b) => a + b, 0);
  const mean = total / elapseds.length;
  console.log(`Total elapsed time: ${total}ms`);
  console.log(`Mean elapsed time: ${mean}ms`);
}

main();

// windows
// blocking
// Total elapsed time: 16689ms
// Mean elapsed time: 16.689ms
// tokio
// Total elapsed time: 17835ms
// Mean elapsed time: 17.835ms

// linux
// blocking
// Total elapsed time: 1325ms
// Mean elapsed time: 1.325ms
// tokio
// Total elapsed time: 6285ms
// Mean elapsed time: 6.285ms

// macos
// blocking
// Total elapsed time: 3724ms
// Mean elapsed time: 3.724ms
// tokio
// Total elapsed time: 4089ms
// Mean elapsed time: 4.089ms
