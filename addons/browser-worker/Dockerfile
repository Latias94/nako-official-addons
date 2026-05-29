FROM node:24-bookworm-slim

WORKDIR /app

COPY package*.json ./
RUN npm ci
RUN npx playwright install --with-deps chromium

COPY . .

ENV HOST=0.0.0.0
ENV PORT=3000
ENV CRAWLEE_STORAGE_DIR=/tmp/nako-browser-worker

EXPOSE 3000

CMD ["npm", "start"]
