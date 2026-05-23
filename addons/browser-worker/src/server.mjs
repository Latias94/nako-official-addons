import { createServer } from 'node:http';

import { createApp } from './app.mjs';

const port = Number(process.env.PORT ?? '3000');
const host = process.env.HOST ?? '0.0.0.0';

const app = createApp();
const server = createServer(app);

server.listen(port, host, () => {
  process.stdout.write(`nako-browser-worker listening on ${host}:${port}\n`);
});
