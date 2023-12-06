import { createServer } from "http";
import { readFile, readFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath, parse } from "url";
import { lookup } from "mime-types";

const __filename = fileURLToPath(import.meta.url);

createServer((req, res) => {
  const __dirname = dirname(__filename);
  const path = join(__dirname, "../../dist", parse(req.url, true).path);
  readFile(path, (err, data) => {
    if (err) {
      res.writeHead(200, { "Content-Type": "text/html" });
      res.end(readFileSync(join(__dirname, "../../dist/index.html")));
    } else {
      res.writeHead(200, { "Content-Type": lookup(path) });
      res.end(data);
    }
  });
}).listen(8081, () => {
  console.log("Server running at http://localhost:8081/");
});
