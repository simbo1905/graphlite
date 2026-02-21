FROM node:22-slim

RUN apt-get update && apt-get install -y chromium && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY serve.mjs .
COPY index.html .
COPY pkg/ pkg/

EXPOSE 8080
CMD ["node", "serve.mjs"]
