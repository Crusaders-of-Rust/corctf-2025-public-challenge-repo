FROM node:22.18-slim AS builder
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build
RUN npm prune --production

FROM node:22.18-slim
WORKDIR /app
COPY --from=builder /app ./
EXPOSE 3000
CMD ["node", "--experimental-strip-types", "server.ts"]
