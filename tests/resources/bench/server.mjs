import http from "http";

const args = process.argv.slice(2);
const port = args[0];

console.log(`Server is trying to listen on port ${port}`);

const server = http.createServer((req, res) => {
  res.end("Hello, World!");
});

server.listen(port, () => {
  console.log("Server is running on port 3000");
});
